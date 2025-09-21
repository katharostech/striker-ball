use super::*;

pub mod countdown;
pub mod fade;
pub mod howtoplay;
pub mod lan_ui;
pub mod match_done;
pub mod pause;
pub mod score_display;
pub mod splash;
pub mod team_select;
pub mod winner;

pub use countdown::*;
pub use fade::*;
pub use howtoplay::*;
pub use lan_ui::*;
pub use match_done::*;
pub use pause::*;
pub use score_display::*;
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
            .add_system_to_stage(Update, show_ui);
    }
}
pub fn show_ui(world: &World) {
    fade::show(world);
    splash::show(world);
    team_select::show(world);
    pause::show(world);
    howtoplay::show(world);
    lan_ui::show(world);

    if let Some(world) = world.resource_mut::<Sessions>().get_world(session::PLAY) {
        fade::show(world);
        countdown::show(world);
        score_display::show(world);
        match_done::show(world);
        winner::show(world);
    }
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
