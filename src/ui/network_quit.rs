use super::*;

#[derive(HasSchema, Clone, Copy, Default)]
pub struct NetworkQuit {
    pub visible: bool,
    pub state: NetworkQuitState,
    // TODO: Track gamepad that activated ui for controls.
}

impl NetworkQuit {
    fn output_hide(&mut self) -> Option<NetworkQuitOutput> {
        if self.visible {
            self.visible = false;
            self.state = NetworkQuitState::No;
            NetworkQuitOutput::Hide.into()
        } else {
            None
        }
    }
    fn output_show(&mut self) -> Option<NetworkQuitOutput> {
        if !self.visible {
            self.visible = true;
            NetworkQuitOutput::Show.into()
        } else {
            None
        }
    }
    fn output_quit(&mut self) -> Option<NetworkQuitOutput> {
        if self.visible {
            self.visible = false;
            NetworkQuitOutput::Quit.into()
        } else {
            None
        }
    }
}

#[derive(HasSchema, Clone, Copy, Default, PartialEq, Eq)]
pub enum NetworkQuitState {
    #[default]
    No,
    Yes,
}

#[derive(HasSchema, Clone, Copy, Default, PartialEq, Eq)]
pub enum NetworkQuitOutput {
    #[default]
    Quit,
    Show,
    Hide,
}

impl SessionPlugin for NetworkQuit {
    fn install(self, session: &mut SessionBuilder) {
        session.insert_resource(self);
    }
}

impl NetworkQuit {
    pub fn process_input(&mut self, world: &World) -> Option<NetworkQuitOutput> {
        let mut output = None;

        let local_inputs = world.resource::<LocalInputs>();
        let keyboard_inputs = world.resource::<KeyboardInputs>();

        for event in &keyboard_inputs.key_events {
            if let KeyboardEvent {
                key_code: Maybe::Set(KeyCode::Escape),
                button_state: ButtonState::Pressed,
                ..
            } = event
            {
                if self.visible {
                    output = output.or_else(|| self.output_hide());
                } else {
                    output = output.or_else(|| self.output_show());
                }
            }
        }

        for (_gamepad, input) in local_inputs.iter() {
            if input.start.just_pressed() {
                if self.visible {
                    output = output.or_else(|| self.output_hide());
                } else {
                    output = output.or_else(|| self.output_show());
                }
            }
            if input.south.just_pressed() && self.visible {
                match self.state {
                    NetworkQuitState::Yes => output = output.or_else(|| self.output_quit()),
                    NetworkQuitState::No => output = output.or_else(|| self.output_hide()),
                };
            }
            if input.left.just_pressed() || input.right.just_pressed() && self.visible {
                match self.state {
                    NetworkQuitState::No => self.state = NetworkQuitState::Yes,
                    NetworkQuitState::Yes => self.state = NetworkQuitState::No,
                }
            }
        }
        output
    }
    pub fn process_ui(&mut self, world: &World) -> Option<NetworkQuitOutput> {
        let mut output = None;

        if !self.visible {
            return output;
        }

        let asset_server = world.resource::<AssetServer>();
        let root = asset_server.root::<Data>();
        let ctx = &world.resource::<EguiCtx>();

        let inner_font = asset_server
            .get(root.font.primary_inner)
            .family_name
            .clone();
        let outer_font = asset_server
            .get(root.font.primary_outer)
            .family_name
            .clone();

        use egui::*;

        Area::new("network_quitter")
            .anchor(Align2::CENTER_CENTER, [0., 0.])
            .order(Order::Foreground)
            .show(ctx, |ui| {
                BorderedFrame::new(&root.menu.bframe)
                    .padding(Margin::same(10.0))
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            let text = "Do You Want To Quit?";
                            let response =
                                ui.label(RichText::new(text).color(Color32::WHITE).font(FontId {
                                    size: 7.0,
                                    family: FontFamily::Name(inner_font.clone()),
                                }));
                            TextPainter::new(text)
                                .size(7.0)
                                .pos(response.rect.min)
                                .family(outer_font.clone())
                                .color(Color32::BLACK)
                                .paint(ui.painter());
                            ui.horizontal(|ui| {
                                let response = BorderedFrame::new(&root.menu.bframe)
                                    .padding(Margin::same(6.0))
                                    .show(ui, |ui| {
                                        let text = "Yes";
                                        let color = if self.state == NetworkQuitState::Yes {
                                            Color32::YELLOW
                                        } else {
                                            Color32::WHITE
                                        };
                                        let response = ui.label(
                                            RichText::new(text).color(color).font(FontId {
                                                size: 7.0,
                                                family: FontFamily::Name(inner_font.clone()),
                                            }),
                                        );
                                        TextPainter::new(text)
                                            .size(7.0)
                                            .pos(response.rect.min)
                                            .family(outer_font.clone())
                                            .color(Color32::BLACK)
                                            .paint(ui.painter())
                                    });
                                if ctx.clicked_rect(response.response.rect) {
                                    output = self.output_quit();
                                }
                                if response.response.hovered() {
                                    self.state = NetworkQuitState::Yes;
                                }
                                let response = BorderedFrame::new(&root.menu.bframe)
                                    .padding(Margin::same(6.0))
                                    .show(ui, |ui| {
                                        let text = "No";
                                        let color = if self.state == NetworkQuitState::No {
                                            Color32::YELLOW
                                        } else {
                                            Color32::WHITE
                                        };
                                        let response = ui.label(
                                            RichText::new(text).color(color).font(FontId {
                                                size: 7.0,
                                                family: FontFamily::Name(inner_font.clone()),
                                            }),
                                        );
                                        TextPainter::new(text)
                                            .size(7.0)
                                            .pos(response.rect.min)
                                            .family(outer_font.clone())
                                            .color(Color32::BLACK)
                                            .paint(ui.painter())
                                    });
                                if ctx.clicked_rect(response.response.rect) {
                                    output = output.or_else(|| self.output_hide());
                                }
                                if response.response.hovered() {
                                    self.state = NetworkQuitState::No;
                                }
                            });
                        })
                    })
            });
        output
    }
}
