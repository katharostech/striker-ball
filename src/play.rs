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

#[cfg(not(target_arch = "wasm32"))]
type NetworkMatchSocket = bones_framework::networking::NetworkMatchSocket;
#[cfg(target_arch = "wasm32")]
type NetworkMatchSocket = ();

#[derive(HasSchema, Clone)]
pub enum PlayMode {
    Online {
        clientpad: u32,
        socket: NetworkMatchSocket,
    },
    Offline(PlayersInfo),
}
impl Default for PlayMode {
    fn default() -> Self {
        Self::Offline(default())
    }
}
#[derive(HasSchema, Debug, Clone)]
pub struct PlayersInfo {
    pub team_a: TeamInfo,
    pub team_b: TeamInfo,
}
impl Default for PlayersInfo {
    fn default() -> Self {
        Self {
            team_a: TeamInfo::Single(PlayerInfo {
                number: 0,
                gamepad: 0,
                dual_stick: true,
                slot: PlayerSlot::A1,
            }),
            team_b: TeamInfo::Single(PlayerInfo {
                number: 0,
                gamepad: 0,
                dual_stick: true,
                slot: PlayerSlot::B1,
            }),
        }
    }
}
/// Represents all the info related to a character in the game.
#[derive(HasSchema, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlayerInfo {
    /// The user join index,
    /// `0` being P1, `1` being P2,
    /// and so on.
    pub number: usize,
    /// The associated gamepad id of the player.
    pub gamepad: u32,
    /// Whether or not this player is being controlled with
    /// dual stick controls.
    pub dual_stick: bool,
    /// The exact character slot.
    pub slot: PlayerSlot,
}
#[derive(HasSchema, Debug, Clone)]
pub enum TeamInfo {
    Single(PlayerInfo),
    Double(PlayerInfo, PlayerInfo),
}
impl Default for TeamInfo {
    fn default() -> Self {
        TeamInfo::Double(default(), default())
    }
}
impl TeamInfo {
    pub fn is_dual_stick(&self) -> bool {
        matches!(self, Self::Single(..))
    }
    pub fn primary(&self) -> PlayerInfo {
        match self.clone() {
            TeamInfo::Single(player_sign) | TeamInfo::Double(player_sign, _) => player_sign,
        }
    }
    pub fn secondary(&self) -> PlayerInfo {
        match self.clone() {
            TeamInfo::Single(player_info) => PlayerInfo {
                slot: player_info.slot.partner(),
                ..player_info
            },
            TeamInfo::Double(_, player_sign) => player_sign,
        }
    }
}
#[derive(HasSchema, Clone)]
#[schema(no_default)]
pub struct PlayerEntSigns {
    pub a1: Entity,
    pub a2: Entity,
    pub b1: Entity,
    pub b2: Entity,
}
impl PlayerEntSigns {
    pub fn partner(&self, entity: Entity) -> Entity {
        if entity == self.a1 {
            self.a2
        } else if entity == self.a2 {
            self.a1
        } else if entity == self.b1 {
            self.b2
        } else if entity == self.b2 {
            self.b1
        } else {
            panic!("controller is not assigned to a player")
        }
    }
    pub fn entities(&self) -> [Entity; 4] {
        [self.a1, self.a2, self.b1, self.b2]
    }
}

#[derive(HasSchema, Clone, Copy, Default)]
pub enum PlayState {
    #[default]
    Countdown,
    WaitForScore,
    ScoreDisplay,
    Podium,
    MatchDone,
}

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

pub struct ScenePlugin {
    pub mode: PlayMode,
}
impl SessionPlugin for ScenePlugin {
    fn install(self, session: &mut SessionBuilder) {
        match &self.mode {
            PlayMode::Offline { .. } => {
                session.runner = Box::new(OfflineRunner::default());
            }
            PlayMode::Online { .. } => {
                unimplemented!();
            }
        };
        session.insert_resource(self.mode);
        session.init_resource::<PlayInputs>();

        session.install_plugin(Path2dToggle::hidden());
        session.add_system_to_stage(First, fix_camera_size);
        session.add_system_to_stage(Update, toggle_debug_lines);

        session.add_startup_system(spawn::scene);
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

pub struct FlowPlugin;
impl SessionPlugin for FlowPlugin {
    fn install(self, session: &mut SessionBuilder) {
        session.insert_resource(PlayState::default());
        session.insert_resource(Score {
            target: 7,
            ..Default::default()
        });
        session.add_startup_system(|root: Root<Data>, mut audio: ResMut<AudioCenter>| {
            if let Some(kira::sound::PlaybackState::Playing) = audio.music_state() {
                return;
            }
            audio.play_music_advanced(
                *root.sound.menu_music,
                root.sound.menu_music.volume(),
                true,
                false,
                0.0,
                1.0,
                true,
            );
        });
        session.add_system_to_stage(First, |world: &World| {
            let state = *world.resource::<PlayState>();
            match state {
                PlayState::Countdown => countdown_update(world),
                PlayState::WaitForScore => wait_for_score_update(world),
                PlayState::ScoreDisplay => world.run_system(score_display_update, ()),
                PlayState::Podium => podium_update(world),
                PlayState::MatchDone => match_done_update(world),
            }
        });
    }
}
#[derive(HasSchema, Clone, Default)]
pub struct Score {
    pub target: u8,
    pub current: PinScore,
    pub previous: PinScore,
}
impl Score {
    pub fn update_current(&mut self, score: PinScore) {
        self.current = score;
    }
    pub fn update_previous(&mut self) {
        self.previous = self.current;
    }
    pub fn scorer(&self) -> Option<Team> {
        if self.current.a != self.previous.a {
            return Some(Team::A);
        }
        if self.current.b != self.previous.b {
            return Some(Team::B);
        }
        None
    }
    pub fn winner(&self) -> Option<Team> {
        if self.current.b == self.target {
            return Some(Team::B);
        }
        if self.current.a == self.target {
            return Some(Team::A);
        }
        None
    }
}

pub fn countdown_update(play: &World) {
    if play.resource_mut::<Countdown>().timer.finished() {
        play.run_system(set_player_states_free, ());

        *play.resource_mut::<PlayState>() = PlayState::WaitForScore;
    }
}
pub fn wait_for_score_update(play: &World) {
    let pin_score = *play.resource::<PinScore>();
    let mut score = play.resource_mut::<Score>();

    let mut fade = play.resource_mut::<Fade>();
    let mut score_display = play.resource_mut::<ScoreDisplay>();

    // Update current to detect changes
    score.update_current(pin_score);

    if let Some(scorer) = score.scorer() {
        match scorer {
            Team::A => play.run_system(set_player_states_scored_a, ()),
            Team::B => play.run_system(set_player_states_scored_b, ()),
        }
        score_display.restart();
        fade.restart();
        *play.resource_mut() = PlayState::ScoreDisplay;
    }
}
pub fn score_display_update(
    root: Root<Data>,
    fade: Res<Fade>,
    entities: Res<Entities>,
    pin_score: Res<PinScore>,
    mut audio: ResMut<AudioCenter>,
    mut balls: CompMut<Ball>,
    mut transforms: CompMut<Transform>,
    mut players: CompMut<Player>,
    mut state: CompMut<State>,
    mut countdown: ResMut<Countdown>,
    mut winner: ResMut<WinnerBanner>,
    mut play_state: ResMut<PlayState>,
    mut score: ResMut<Score>,
) {
    if fade.fade_out.just_finished() {
        tracing::info!("fade out for round restart, reseting positions");

        // The score may have changed while we were displaying so we update
        // for a potential win.
        score.update_current(*pin_score);

        for (_player_e, (player, state, transform)) in
            entities.iter_with((&mut players, &mut state, &mut transforms))
        {
            *transform = new_player_transform(player.id, &root);

            if score.winner().is_none() {
                state.current = player::state::wait();
            }
        }
        for (_ball_e, (ball, transform)) in entities.iter_with((&mut balls, &mut transforms)) {
            ball.velocity = default();
            transform.translation.y = 0.0;
            transform.translation.x = match score.scorer().unwrap() {
                Team::A => root.screen_size.x / 10.,
                Team::B => root.screen_size.x / -10.,
            };
        }
    }
    if fade.fade_in.just_finished() {
        tracing::info!("fade in for round restart");
        if let Some(team) = score.winner() {
            tracing::info!("winner found, showing winner");
            winner.team = team;
            winner.visual.show();
            winner.timer = Timer::from_seconds(3., TimerMode::Once);
            audio.play_sound(*root.sound.winner, root.sound.winner.volume());
            audio.stop_music(false);
            *play_state = PlayState::Podium;
        } else {
            tracing::info!("no winner, starting countdown");
            countdown.restart();
            *play_state = PlayState::Countdown;
        }
        // We're done reading until the next score.
        score.update_previous();
    }
}

fn podium_update(play: &World) {
    let mut winner = play.resource_mut::<WinnerBanner>();

    if winner.timer.just_finished() {
        tracing::info!("showing match done ui");
        winner.visual.hide();
        play.resource_mut::<MatchDone>().visual.show();
        *play.resource_mut() = PlayState::MatchDone;
    }
}

fn match_done_update(play: &World) {
    let match_done = *play.resource::<MatchDone>();
    if !match_done.visual.shown() {
        return;
    };

    let to_team_select = || {
        let mut sessions = play.resource_mut::<Sessions>();
        let ui = sessions.get_world(session::UI).unwrap();
        start_fade(
            ui,
            FadeTransition {
                hide: play_hide,
                prep: team_select_prep,
                finish: team_select_finish,
            },
        );
    };
    let play_again = || {
        let mut sessions = play.resource_mut::<Sessions>();
        let ui = sessions.get_world(session::UI).unwrap();
        start_fade(
            ui,
            FadeTransition {
                hide: play_hide,
                prep: play_prep,
                finish: play_finish,
            },
        );
    };
    let to_splash = || {
        let mut sessions = play.resource_mut::<Sessions>();
        let ui = sessions.get_world(session::UI).unwrap();
        start_fade(
            ui,
            FadeTransition {
                hide: play_hide,
                prep: splash_prep,
                finish: splash_finish,
            },
        );
    };

    let inputs = play.resource::<LocalInputs>();

    for (_id, input) in inputs.iter() {
        if input.south.just_pressed() {
            match match_done.state {
                MatchDoneState::TeamSelect => to_team_select(),
                MatchDoneState::PlayAgain => play_again(),
                MatchDoneState::Quit => to_splash(),
            }
            play.resource_mut::<MatchDone>().visual.hide();
        }
        if input.up.just_pressed() {
            play.resource_mut::<MatchDone>().cycle_up();
        }
        if input.down.just_pressed() {
            play.resource_mut::<MatchDone>().cycle_down();
        }
    }
}

pub fn set_player_states_scored_a(
    entities: Res<Entities>,
    players: Comp<Player>,
    mut states: CompMut<State>,
) {
    for (_player_e, (player, state)) in entities.iter_with((&players, &mut states)) {
        match player.team() {
            Team::A => state.current = player::state::win(),
            Team::B => state.current = player::state::lose(),
        }
    }
}
pub fn set_player_states_scored_b(
    entities: Res<Entities>,
    players: Comp<Player>,
    mut states: CompMut<State>,
) {
    for (_player_e, (player, state)) in entities.iter_with((&players, &mut states)) {
        match player.team() {
            Team::A => state.current = player::state::lose(),
            Team::B => state.current = player::state::win(),
        }
    }
}
pub fn set_player_states_free(
    entities: Res<Entities>,
    players: Comp<Player>,
    mut states: CompMut<State>,
) {
    tracing::info!("freeing players");
    for (_player_e, (_player, state)) in entities.iter_with((&players, &mut states)) {
        state.current = player::state::free()
    }
}

pub fn toggle_debug_lines(inputs: Res<KeyboardInputs>, mut path2ds: ResMut<Path2dToggle>) {
    for input in inputs.key_events.iter() {
        if input.button_state == ButtonState::Pressed && input.key_code == Set(KeyCode::F3) {
            path2ds.hide = !path2ds.hide;
        }
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
