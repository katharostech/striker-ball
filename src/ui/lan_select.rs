use super::*;

pub const MATCHMAKER_SERVICE_NAME_ONEPLAYER: &str = "sb1player";
pub const MATCHMAKER_SERVICE_NAME_TWOPLAYER: &str = "sb2player";

#[derive(HasSchema, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ServiceType {
    OnePlayer(u32),
    TwoPlayer(SingleSource, SingleSource),
}
impl Default for ServiceType {
    fn default() -> Self {
        Self::OnePlayer(0)
    }
}
impl ServiceType {
    pub fn service_name(&self) -> &str {
        match self {
            ServiceType::OnePlayer(_) => MATCHMAKER_SERVICE_NAME_ONEPLAYER,
            ServiceType::TwoPlayer(_, _) => MATCHMAKER_SERVICE_NAME_TWOPLAYER,
        }
    }
}

#[derive(HasSchema, Clone, Default)]
pub struct LanSelect {
    pub visible: bool,
    pub selection: LanSelection,
}

#[derive(HasSchema, Clone, Default, PartialEq, Eq)]
pub enum LanSelection {
    #[default]
    /// Focus is on the one player button
    OnePlayer,
    /// Waiting for a gamepad to bind because
    /// a keyboard selected one player.
    OnePlayerBind,
    /// Focus is on the two player button
    TwoPlayer,
    /// Waiting for second player to bind.
    /// Contains the source of the player that
    /// selected the two player button.
    TwoPlayerBind { player1: SingleSource },
    // ThreePlayer,
}

pub enum LanSelectOutput {
    Exit,
    ServiceType(ServiceType),
}

impl SessionPlugin for LanSelect {
    fn install(self, session: &mut SessionBuilder) {
        session.insert_resource(self);
    }
}
fn foreground() -> egui::LayerId {
    use egui::*;
    LayerId::new(Order::Foreground, Id::new("lan_select_foreground"))
}
impl LanSelect {
    pub fn process_input(&mut self, world: &World) -> Option<LanSelectOutput> {
        let mut output = None;

        let local_inputs = world.resource::<LocalInputs>();

        for (source, input) in local_inputs.iter() {
            if input.menu_select.just_pressed() {
                match self.selection {
                    LanSelection::OnePlayer => match *source {
                        SingleSource::Gamepad(gamepad_id) => {
                            output =
                                LanSelectOutput::ServiceType(ServiceType::OnePlayer(gamepad_id))
                                    .into();
                        }
                        SingleSource::KeyboardMouse => self.selection = LanSelection::OnePlayerBind,
                        SingleSource::CPU(..) => unreachable!(),
                    },
                    LanSelection::OnePlayerBind => match *source {
                        SingleSource::Gamepad(gamepad_id) => {
                            output =
                                LanSelectOutput::ServiceType(ServiceType::OnePlayer(gamepad_id))
                                    .into();
                        }
                        SingleSource::KeyboardMouse => self.selection = LanSelection::OnePlayer,
                        SingleSource::CPU(..) => unreachable!(),
                    },
                    LanSelection::TwoPlayer => {
                        self.selection = LanSelection::TwoPlayerBind { player1: *source };
                    }
                    LanSelection::TwoPlayerBind { player1 } => {
                        if *source == player1 {
                            self.selection = LanSelection::TwoPlayer;
                        } else {
                            output = LanSelectOutput::ServiceType(ServiceType::TwoPlayer(
                                player1, *source,
                            ))
                            .into();
                        }
                    }
                }
            }
            if input.menu_back.just_pressed() {
                match self.selection {
                    LanSelection::OnePlayer | LanSelection::TwoPlayer => {
                        output = LanSelectOutput::Exit.into()
                    }
                    LanSelection::OnePlayerBind => self.selection = LanSelection::OnePlayer,
                    LanSelection::TwoPlayerBind { .. } => self.selection = LanSelection::TwoPlayer,
                }
            }
            if input.menu_left.just_pressed() || input.menu_right.just_pressed() {
                match self.selection {
                    LanSelection::OnePlayer => self.selection = LanSelection::TwoPlayer,
                    LanSelection::TwoPlayer => self.selection = LanSelection::OnePlayer,
                    LanSelection::OnePlayerBind | LanSelection::TwoPlayerBind { .. } => {}
                }
            }
        }
        output
    }
    pub fn process_ui(&mut self, world: &World) -> Option<LanSelectOutput> {
        if !self.visible {
            return None;
        }
        let mut output = None;

        let asset_server = world.resource::<AssetServer>();
        let root = asset_server.root::<Data>();
        let textures = world.resource::<EguiTextures>();
        let ctx = world.resource::<EguiCtx>();

        use egui::*;

        let area = Area::new("splash")
            .anchor(Align2::CENTER_CENTER, [0., 0.])
            .show(&ctx, |ui| {
                ui.image(load::SizedTexture::new(
                    textures.get(root.menu.splash.bg),
                    root.screen_size.to_array(),
                ));
            });
        let mut painter = ctx.layer_painter(foreground());

        painter.set_clip_rect(area.response.rect);

        let inner_font = asset_server
            .get(root.font.primary_inner)
            .family_name
            .clone();
        let outer_font = asset_server
            .get(root.font.primary_outer)
            .family_name
            .clone();

        match self.selection {
            LanSelection::OnePlayer | LanSelection::TwoPlayer => {
                Area::new("lan_select_buttons")
                    .anchor(Align2::CENTER_CENTER, [0., 0.])
                    .order(Order::Foreground)
                    .show(&world.resource::<EguiCtx>(), |ui| {
                        ui.horizontal(|ui| {
                            let irsp = BorderedFrame::new(&root.menu.bframe)
                                .padding(Margin::same(6.0))
                                .show(ui, |ui| {
                                    super::primary_text(
                                        "1 Player",
                                        self.selection == LanSelection::OnePlayer,
                                        &asset_server,
                                        ui,
                                    );
                                });
                            if irsp.response.hovered() {
                                self.selection = LanSelection::OnePlayer;
                            }
                            if ctx.clicked_rect(irsp.response.rect) {
                                self.selection = LanSelection::OnePlayerBind;
                            }
                            let irsp = BorderedFrame::new(&root.menu.bframe)
                                .padding(Margin::same(6.0))
                                .show(ui, |ui| {
                                    super::primary_text(
                                        "2 Player",
                                        self.selection == LanSelection::TwoPlayer,
                                        &asset_server,
                                        ui,
                                    );
                                });
                            if irsp.response.hovered() {
                                self.selection = LanSelection::TwoPlayer;
                            }
                            // `irsp.response.clicked()` doesn't work for some reason (maybe a layering problem)
                            if ctx.clicked_rect(irsp.response.rect) {
                                self.selection = LanSelection::TwoPlayerBind {
                                    player1: SingleSource::KeyboardMouse,
                                };
                            }
                        });
                    });
            }
            LanSelection::OnePlayerBind => {
                let irsp = Area::new("one_player_bind_popup")
                    .anchor(Align2::CENTER_CENTER, [0., 0.])
                    .order(Order::Foreground)
                    .show(&world.resource::<EguiCtx>(), |ui| {
                        BorderedFrame::new(&root.menu.bframe)
                            .padding(Margin::same(50.0))
                            .show(ui, |ui| {
                                let text = "Player 1, Press Select On A Gamepad";
                                let response = ui.label(
                                    RichText::new(text).color(Color32::WHITE).font(FontId {
                                        size: 7.0,
                                        family: FontFamily::Name(inner_font),
                                    }),
                                );
                                TextPainter::new(text)
                                    .size(7.0)
                                    .pos(response.rect.min)
                                    .family(outer_font)
                                    .color(Color32::BLACK)
                                    .paint(ui.painter())
                            })
                    });
                if ctx.clicked_rect(irsp.response.rect) {
                    self.selection = LanSelection::OnePlayer;
                }
            }
            LanSelection::TwoPlayerBind { .. } => {
                let irsp = Area::new("two_player_bind_popup")
                    .anchor(Align2::CENTER_CENTER, [0., 0.])
                    .order(Order::Foreground)
                    .show(&world.resource::<EguiCtx>(), |ui| {
                        BorderedFrame::new(&root.menu.bframe)
                            .padding(Margin::same(50.0))
                            .show(ui, |ui| {
                                let text = "Player 2, Press Select";
                                let response = ui.label(
                                    RichText::new(text).color(Color32::WHITE).font(FontId {
                                        size: 7.0,
                                        family: FontFamily::Name(inner_font),
                                    }),
                                );
                                TextPainter::new(text)
                                    .size(7.0)
                                    .pos(response.rect.min)
                                    .family(outer_font)
                                    .color(Color32::BLACK)
                                    .paint(ui.painter())
                            })
                    });
                if ctx.clicked_rect(irsp.response.rect) {
                    self.selection = LanSelection::TwoPlayer;
                }
            }
        }
        output
    }
}
