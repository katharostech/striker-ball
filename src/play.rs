use super::*;

pub mod layers;
pub mod path2d;

pub mod input;
pub use input::prelude::*;
pub mod player;
pub use player::prelude::*;
pub mod pin;
pub use pin::prelude::*;
pub mod ball;
pub use ball::prelude::*;
pub mod spawn;
pub use spawn::prelude::*;
pub mod flow;
pub use flow::*;
pub mod scene;
pub use scene::*;

/// This should be the complete installation for the play session.
#[derive(Default)]
pub struct PlayPlugin {
    pub mode: PlayMode,
}
impl SessionPlugin for PlayPlugin {
    fn install(self, session: &mut SessionBuilder) {
        session
            .set_priority(session::PLAY_PRIORITY)
            .install_plugin(DefaultSessionPlugin)
            .install_plugin(self::ScenePlugin { mode: self.mode })
            .install_plugin(self::BehaviorsPlugin)
            .install_plugin(self::PlayUIPlugin)
            .install_plugin(self::FlowPlugin);
    }
}

pub struct BehaviorsPlugin;
impl SessionPlugin for BehaviorsPlugin {
    fn install(self, session: &mut SessionBuilder) {
        session
            .install_plugin(StatePlugin)
            .install_plugin(player::plugin)
            .install_plugin(ball::plugin)
            .install_plugin(pin::plugin)
            .install_plugin(LifetimePlugin)
            .install_plugin(FollowPlugin);
    }
}

pub struct PlayUIPlugin;
impl SessionPlugin for PlayUIPlugin {
    fn install(self, session: &mut SessionBuilder) {
        session.install_plugin(Fade::new(3., 1., Color::BLACK, egui::Order::Middle));
        session.install_plugin(Countdown::new(4.0, 1.2));
        session.install_plugin(ScoreDisplay::new(3.65));
        session.install_plugin(WinnerBanner::default());
        session.install_plugin(MatchDone::default());
    }
}
