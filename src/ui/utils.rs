use super::*;

pub fn show_all(world: &World) {
    let pointer_moved = world
        .resource::<EguiCtx>()
        .input(|r| r.pointer.delta() != egui::Vec2::ZERO);
    if pointer_moved {
        world.resource_mut::<LocalInputs>().pointer_navigation = true;
    }

    fade::show(world);

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

pub fn setup_egui(world: &World, root: Root<Data>, ctx: Res<EguiCtx>) {
    world.resources.insert(EguiSize(root.screen_size));
    use egui::*;
    ctx.style_mut(|w| w.visuals.selection.bg_fill = Color32::YELLOW);
    ctx.style_mut(|w| w.visuals.text_cursor.color = Color32::YELLOW);
}

pub fn fix_camera_size(root: Root<Data>, window: Res<Window>, mut cameras: CompMut<Camera>) {
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

pub trait ShowHide: HasSchema + Sized {
    fn show(&mut self);
    fn hide(&mut self);

    fn show_resource(world: &World) {
        world.resource_mut::<Self>().show();
    }
    fn hide_resource(world: &World) {
        world.resource_mut::<Self>().hide();
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
