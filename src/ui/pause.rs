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
}
impl SessionPlugin for Pause {
    fn install(self, session: &mut SessionBuilder) {
        session.insert_resource(self);
    }
}
pub fn show(world: &World) {
    let pause = world.resource::<Pause>();

    if matches!(*pause, Pause::Hidden | Pause::Disabled) {
        return;
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

                let pos = match *pause {
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
                )
            });
        });
}
