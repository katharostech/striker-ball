use super::*;

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
            .install_plugin(self::PlayFlowPlugin);
    }
}

pub struct BehaviorsPlugin;
impl SessionPlugin for BehaviorsPlugin {
    fn install(self, session: &mut SessionBuilder) {
        session
            .install_plugin(StatePlugin)
            .install_plugin(PlayerPlugin)
            .install_plugin(BallPlugin)
            .install_plugin(PinPlugin)
            .install_plugin(LifetimePlugin)
            .install_plugin(FollowPlugin)
            .install_plugin(CpuPlayerPlugin);
    }
}

pub struct PlayUIPlugin;
impl SessionPlugin for PlayUIPlugin {
    fn install(self, session: &mut SessionBuilder) {
        session.install_plugin({
            let mut fade = Fade::new(3., 0.15, 1., Color::BLACK, egui::Order::Middle);
            fade.restart_at_wait();
            fade
        });
        session.install_plugin(Countdown::new(4.0, 1.2));
        session.install_plugin(ScoreDisplay::new(3.65));
        session.install_plugin(WinnerBanner::default());
        session.install_plugin(MatchDone::default());
    }
}
