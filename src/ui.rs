use super::*;

pub mod countdown;
pub mod credits;
pub mod fade;
pub mod howtoplay;
pub mod lan_select;
#[cfg(not(target_arch = "wasm32"))]
pub mod lan_ui;
pub mod match_done;
pub mod network_quit;
pub mod pause;
pub mod score_display;
pub mod settings;
pub mod splash;
pub mod team_select;
pub mod winner;

pub use countdown::*;
pub use credits::*;
pub use fade::*;
pub use howtoplay::*;
pub use lan_select::*;
#[cfg(not(target_arch = "wasm32"))]
pub use lan_ui::*;
pub use match_done::*;
pub use network_quit::*;
pub use pause::*;
pub use score_display::*;
pub use settings::*;
pub use splash::*;
pub use team_select::*;
pub use winner::*;

pub struct UiSessionPlugin;
impl SessionPlugin for UiSessionPlugin {
    fn install(self, session: &mut SessionBuilder) {
        session
            .set_priority(session::UI_PRIORITY)
            .install_plugin(DefaultSessionPlugin)
            .install_plugin(UiScalePlugin)
            .install_plugin(MenuPlugin)
            .add_startup_system(set_egui_styles)
            .add_system_to_stage(Update, show_ui);
    }
}
pub fn show_ui(world: &World) {
    fade::show(world);
    splash::show(world);

    if let Some(world) = world.resource_mut::<Sessions>().get_world(session::PLAY) {
        world.run_system(fix_camera_size, ());
        world.resource_mut::<MatchDone>().process_input(world);
        world.resource_mut::<MatchDone>().process_ui(world);
        fade::show(world);
        countdown::show(world);
        score_display::show(world);
        winner::show(world);
    }
}

pub fn set_egui_styles(ctx: Res<EguiCtx>) {
    use egui::*;
    ctx.style_mut(|w| w.visuals.selection.bg_fill = Color32::YELLOW);
    ctx.style_mut(|w| w.visuals.text_cursor.color = Color32::YELLOW);
}

pub struct UiScalePlugin;
impl SessionPlugin for UiScalePlugin {
    fn install(self, session: &mut SessionBuilder) {
        session.insert_resource(EguiSettings::default());
        session.add_system_to_stage(Update, |world: &World, root: Root<Data>| {
            let size = world.resource::<Window>().size;
            world.resource_mut::<EguiSettings>().scale =
                // TODO: Use resource instead of root asset & Move to utils module
                (size.y / root.screen_size.y).min(size.x / root.screen_size.x) as f64;
        });
    }
}

fn fix_camera_size(root: Root<Data>, window: Res<Window>, mut cameras: CompMut<Camera>) {
    for camera in cameras.iter_mut() {
        let size = root.court.size();
        let ratio = size.x / size.y;
        let wratio = window.size.x / window.size.y;
        if wratio > ratio {
            camera.size = CameraSize::FixedHeight(size.y);
        } else {
            camera.size = CameraSize::FixedWidth(size.x);
        }
    }
}

pub fn primary_text(
    text: &str,
    selected: bool,
    asset_server: &AssetServer,
    ui: &mut egui::Ui,
) -> egui::Response {
    use egui::*;

    let root = asset_server.root::<Data>();

    let inner_font = asset_server
        .get(root.font.primary_inner)
        .family_name
        .clone();
    let outer_font = asset_server
        .get(root.font.primary_outer)
        .family_name
        .clone();

    let builder = TextPainter::new(text).size(7.0).pos(ui.cursor().min);

    let rect = builder
        .clone()
        .family(outer_font)
        .color(Color32::BLACK)
        .paint(ui.painter());

    let color = if selected {
        Color32::YELLOW
    } else {
        Color32::WHITE
    };

    builder
        .clone()
        .family(inner_font)
        .color(color)
        .paint(ui.painter());

    ui.allocate_rect(rect, Sense::click())
}
