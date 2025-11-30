use super::*;

#[derive(HasSchema, Clone, Default)]
#[repr(C)]
pub struct PauseAssets {
    pub menu: SizedImageAsset,
    pub cursor: SizedImageAsset,
    pub continue_pos: Vec2,
    pub restart_pos: Vec2,
    pub team_select_pos: Vec2,
}

impl SessionPlugin for Pause {
    fn install(self, session: &mut SessionBuilder) {
        session.insert_resource(self);
    }
}

pub enum PauseOutput {
    Hide,
    Show,
    Restart,
    Quit,
}

#[derive(HasSchema, Clone, Default, Copy, PartialEq, Eq)]
pub enum Pause {
    #[default]
    Disabled,
    Hidden,
    Continue,
    Restart,
    Quit,
}
impl Pause {
    pub fn cycle(&mut self) {
        match self {
            Pause::Disabled | Pause::Hidden => {}
            Pause::Continue => *self = Pause::Restart,
            Pause::Restart => *self = Pause::Quit,
            Pause::Quit => *self = Pause::Continue,
        }
    }

    pub fn process_input(&mut self, world: &World) -> Option<PauseOutput> {
        let mut output = None;

        if *self == Pause::Disabled {
            return output;
        }
        let local_inputs = world.resource::<LocalInputs>();

        for (_gamepad, input) in local_inputs.iter() {
            if input.menu_down.just_pressed() {
                self.cycle()
            }
            if input.menu_up.just_pressed() {
                self.cycle();
                self.cycle();
            }
            if input.pause.just_pressed() {
                match *self {
                    Pause::Hidden => {
                        *self = Pause::Continue;
                        output = PauseOutput::Show.into();
                    }
                    Pause::Continue | Pause::Restart | Pause::Quit => {
                        *self = Pause::Hidden;
                        output = PauseOutput::Hide.into()
                    }
                    Pause::Disabled => unreachable!(),
                }
            }
            if input.menu_select.just_pressed() {
                match *self {
                    Pause::Continue => {
                        *self = Pause::Hidden;
                        output = PauseOutput::Hide.into();
                    }
                    Pause::Restart => {
                        *self = Pause::Disabled;
                        output = PauseOutput::Restart.into();
                    }
                    Pause::Quit => {
                        *self = Pause::Disabled;
                        output = PauseOutput::Quit.into();
                    }
                    Pause::Hidden | Pause::Disabled => {}
                }
            }
        }
        output
    }
    pub fn process_ui(&mut self, world: &World) -> Option<PauseOutput> {
        let mut output = None;

        if matches!(*self, Pause::Hidden | Pause::Disabled) {
            return output;
        }

        let asset_server = world.resource::<AssetServer>();
        let root = asset_server.root::<Data>();
        let PauseAssets {
            menu,
            cursor,
            continue_pos,
            restart_pos,
            team_select_pos,
        } = root.menu.pause;

        use egui::*;
        Area::new("pause-ui")
            .anchor(Align2::CENTER_CENTER, [0., 0.])
            .order(Order::Foreground)
            .show(&world.resource::<EguiCtx>(), |ui| {
                let textures = world.resource::<EguiTextures>();
                ui.horizontal(|ui| {
                    ui.style_mut().spacing.item_spacing = Vec2::ZERO;
                    let response = ui.image(ImageSource::Texture(load::SizedTexture::new(
                        textures.get(*menu),
                        menu.egui_size(),
                    )));

                    let pos = match *self {
                        Pause::Continue => continue_pos,
                        Pause::Restart => restart_pos,
                        Pause::Quit => team_select_pos,
                        Pause::Disabled | Pause::Hidden => unreachable!(),
                    };
                    ui.painter().image(
                        textures.get(*cursor),
                        Rect::from_min_size(
                            response.rect.min + egui::Vec2::new(pos.x, pos.y),
                            cursor.egui_size(),
                        ),
                        default_uv(),
                        Color32::WHITE,
                    );

                    let button_height = restart_pos.y - continue_pos.y;
                    let continue_rect = Rect::from_min_size(
                        response.rect.min + egui::vec2(0.0, continue_pos.y - button_height / 4.0),
                        egui::vec2(response.rect.width(), button_height),
                    );
                    let team_select_rect = Rect::from_min_size(
                        response.rect.min
                            + egui::vec2(0.0, team_select_pos.y - button_height / 4.0),
                        egui::vec2(response.rect.width(), button_height),
                    );
                    let restart_rect = Rect::from_min_size(
                        response.rect.min + egui::vec2(0.0, restart_pos.y - button_height / 4.0),
                        egui::vec2(response.rect.width(), button_height),
                    );

                    if ui.ctx().hovered_rect(continue_rect) {
                        *self = Pause::Continue;
                    }
                    if ui.ctx().hovered_rect(restart_rect) {
                        *self = Pause::Restart;
                    }
                    if ui.ctx().hovered_rect(team_select_rect) {
                        *self = Pause::Quit;
                    }

                    if ui.ctx().clicked_rect(continue_rect) {
                        *self = Pause::Hidden;
                        output = PauseOutput::Hide.into();
                    }
                    if ui.ctx().clicked_rect(restart_rect) {
                        *self = Pause::Disabled;
                        output = PauseOutput::Restart.into();
                    }
                    if ui.ctx().clicked_rect(team_select_rect) {
                        *self = Pause::Disabled;
                        output = PauseOutput::Quit.into();
                    }
                });
            });
        output
    }
}
