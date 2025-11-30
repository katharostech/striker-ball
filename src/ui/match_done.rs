use super::*;

#[derive(HasSchema, Clone, Default)]
#[repr(C)]
pub struct MatchDoneAssets {
    pub menu: SizedImageAsset,
    pub cursor: SizedImageAsset,
    pub play_again_pos: Vec2,
    pub team_select_pos: Vec2,
    pub quit_pos: Vec2,
}

#[derive(HasSchema, Clone, Default, Copy, Deref, DerefMut)]
pub struct MatchDone {
    pub visual: Visual,
    #[deref]
    pub state: MatchDoneState,
    pub output: Option<MatchDoneOutput>,
}
#[derive(HasSchema, Clone, Default, Copy)]
pub enum MatchDoneState {
    #[default]
    PlayAgain,
    TeamSelect,
    Quit,
}
impl MatchDone {
    pub fn cycle_up(&mut self) {
        self.state = match self.state {
            MatchDoneState::PlayAgain => MatchDoneState::Quit,
            MatchDoneState::TeamSelect => MatchDoneState::PlayAgain,
            MatchDoneState::Quit => MatchDoneState::TeamSelect,
        }
    }
    pub fn cycle_down(&mut self) {
        self.state = match self.state {
            MatchDoneState::PlayAgain => MatchDoneState::TeamSelect,
            MatchDoneState::TeamSelect => MatchDoneState::Quit,
            MatchDoneState::Quit => MatchDoneState::PlayAgain,
        }
    }
}

impl SessionPlugin for MatchDone {
    fn install(self, session: &mut SessionBuilder) {
        session.insert_resource(self);
    }
}

type MatchDoneOutput = MatchDoneState;

impl MatchDone {
    pub fn process_input(&mut self, world: &World) -> Option<MatchDoneOutput> {
        let mut output = None;

        if !self.visual.shown() {
            // TODO: In general, for the sake of clarity, the outputs,
            // if stored like they are here, should not be set if the `Some` value
            // hasn't been read yet.
            self.output = output;
            return output;
        }
        let inputs = world.resource::<LocalInputs>();

        for (_id, input) in inputs.iter() {
            if input.menu_select.just_pressed() {
                output = match self.state {
                    MatchDoneState::PlayAgain => MatchDoneOutput::PlayAgain.into(),
                    MatchDoneState::TeamSelect => MatchDoneOutput::TeamSelect.into(),
                    MatchDoneState::Quit => MatchDoneOutput::Quit.into(),
                };
                self.visual.hide();
            }
            if input.menu_up.just_pressed() {
                self.cycle_up();
            }
            if input.menu_down.just_pressed() {
                self.cycle_down();
            }
        }
        self.output = output;
        output
    }
    pub fn process_ui(&mut self, world: &World) -> Option<MatchDoneOutput> {
        let mut output = None;

        if !self.visual.shown() {
            self.output = output;
            return output;
        }
        let asset_server = world.resource::<AssetServer>();
        let root = asset_server.root::<Data>();
        let MatchDoneAssets {
            menu,
            cursor,
            play_again_pos,
            team_select_pos,
            quit_pos,
        } = root.menu.match_done;

        use egui::*;
        Area::new("match-done-ui")
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

                    let pos = match self.state {
                        MatchDoneState::PlayAgain => play_again_pos,
                        MatchDoneState::TeamSelect => team_select_pos,
                        MatchDoneState::Quit => quit_pos,
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

                    let button_height = team_select_pos.y - play_again_pos.y;
                    let play_again_rect = Rect::from_min_size(
                        response.rect.min + egui::vec2(0.0, play_again_pos.y - button_height / 4.0),
                        egui::vec2(response.rect.width(), button_height),
                    );
                    let team_select_rect = Rect::from_min_size(
                        response.rect.min
                            + egui::vec2(0.0, team_select_pos.y - button_height / 4.0),
                        egui::vec2(response.rect.width(), button_height),
                    );
                    let quit_rect = Rect::from_min_size(
                        response.rect.min + egui::vec2(0.0, quit_pos.y - button_height / 4.0),
                        egui::vec2(response.rect.width(), button_height),
                    );

                    if ui.ctx().hovered_rect(play_again_rect) {
                        self.state = MatchDoneState::PlayAgain;
                    }
                    if ui.ctx().hovered_rect(team_select_rect) {
                        self.state = MatchDoneState::TeamSelect;
                    }
                    if ui.ctx().hovered_rect(quit_rect) {
                        self.state = MatchDoneState::Quit;
                    }

                    if ui.ctx().clicked_rect(play_again_rect) {
                        self.visual.hide();
                        output = MatchDoneOutput::PlayAgain.into();
                    }
                    if ui.ctx().clicked_rect(team_select_rect) {
                        self.visual.hide();
                        output = MatchDoneOutput::TeamSelect.into();
                    }
                    if ui.ctx().clicked_rect(quit_rect) {
                        self.visual.hide();
                        output = MatchDoneOutput::Quit.into();
                    }
                });
            });
        self.output = output;
        output
    }
}
