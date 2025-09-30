use super::*;

#[derive(HasSchema, Clone, Default)]
pub struct LanSelect {
    pub visible: bool,
    pub selection: LanSelection,
}

#[derive(HasSchema, Clone, Default, PartialEq, Eq)]
pub enum LanSelection {
    #[default]
    OnePlayer,
    TwoPlayer,
    TwoPlayerBind {
        player1: u32,
    },
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
    pub fn process(&mut self, world: &World) -> Option<LanSelectOutput> {
        if self.visible {
            self.process_input(world).or(self.process_ui(world))
        } else {
            None
        }
    }
    pub fn process_input(&mut self, world: &World) -> Option<LanSelectOutput> {
        let mut output = None;

        let local_inputs = world.resource::<LocalInputs>();

        for (gamepad, input) in local_inputs.iter() {
            if input.south.just_pressed() {
                match self.selection {
                    LanSelection::OnePlayer => {
                        output =
                            LanSelectOutput::ServiceType(ServiceType::OnePlayer(*gamepad)).into();
                    }
                    LanSelection::TwoPlayer => {
                        self.selection = LanSelection::TwoPlayerBind { player1: *gamepad };
                    }
                    LanSelection::TwoPlayerBind { player1 } => {
                        if *gamepad != player1 {
                            output = LanSelectOutput::ServiceType(ServiceType::TwoPlayer(
                                player1, *gamepad,
                            ))
                            .into();
                        } else {
                            self.selection = LanSelection::TwoPlayer;
                        }
                    }
                }
            }
            if input.west.just_pressed() {
                match self.selection {
                    LanSelection::OnePlayer | LanSelection::TwoPlayer => {
                        output = LanSelectOutput::Exit.into();
                    }
                    LanSelection::TwoPlayerBind { .. } => {
                        self.selection = LanSelection::TwoPlayer;
                    }
                }
            }
            if input.left.just_pressed() || input.right.just_pressed() {
                match self.selection {
                    LanSelection::OnePlayer => self.selection = LanSelection::TwoPlayer,
                    LanSelection::TwoPlayer => self.selection = LanSelection::OnePlayer,
                    LanSelection::TwoPlayerBind { .. } => {}
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

        if matches!(
            self.selection,
            LanSelection::OnePlayer | LanSelection::TwoPlayer
        ) {
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
                        // TODO: add keyboard controls
                        // if irsp.response.clicked() {
                        //     output = LanSelectOutput::ServiceType(ServiceType::OnePlayer(keyboard));
                        // }
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
                        // TODO: add keyboard controls
                        // if irsp.response.clicked() {
                        //     self.selection = LanSelection::TwoPlayerBind { player1: () };
                        // }
                    });
                });
        } else {
            Area::new("popup")
                .anchor(Align2::CENTER_CENTER, [0., 0.])
                .order(Order::Foreground)
                .show(&world.resource::<EguiCtx>(), |ui| {
                    BorderedFrame::new(&root.menu.bframe)
                        .padding(Margin::same(50.0))
                        .show(ui, |ui| {
                            let text = "Player 2, Press South";
                            let response =
                                ui.label(RichText::new(text).color(Color32::WHITE).font(FontId {
                                    size: 7.0,
                                    family: FontFamily::Name(inner_font),
                                }));
                            TextPainter::new(text)
                                .size(7.0)
                                .pos(response.rect.min)
                                .family(outer_font)
                                .color(Color32::BLACK)
                                .paint(ui.painter())
                        })
                });
        }
        output
    }
}
