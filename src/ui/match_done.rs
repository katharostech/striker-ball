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
pub fn show(world: &World) {
    let match_done = world.resource::<MatchDone>();

    if !match_done.visual.shown() {
        return;
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

                let pos = match match_done.state {
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
                )
            });
        });
}
