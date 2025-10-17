use super::*;

#[derive(HasSchema, Clone, Copy, Default)]
pub enum PlayState {
    #[default]
    Countdown,
    WaitForScore,
    ScoreDisplay,
    Podium,
    MatchDone,
}

pub struct FlowPlugin;
impl SessionPlugin for FlowPlugin {
    fn install(self, session: &mut SessionBuilder) {
        session.insert_resource(PlayState::default());
        session.insert_resource(Score {
            target: 7,
            ..Default::default()
        });
        #[cfg(not(target_arch = "wasm32"))]
        session.add_single_success_system(handle_disconnections);

        session.add_startup_system(play_music);

        session.add_system_to_stage(First, update_flow);
    }
}

fn update_flow(world: &World) {
    let state = *world.resource::<PlayState>();
    match state {
        PlayState::Countdown => countdown_update(world),
        PlayState::WaitForScore => wait_for_score_update(world),
        PlayState::ScoreDisplay => world.run_system(score_display_update, ()),
        PlayState::Podium => podium_update(world),
        PlayState::MatchDone => match_done_update(world),
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

        #[cfg(not(target_arch = "wasm32"))]
        if play.get_resource::<SyncingInfo>().is_some() {
            // TODO: Add `NetworkMatchDone.show()` and rematch option.
            let mut sessions = play.resource_mut::<Sessions>();
            let ui = sessions.get_world(session::UI).unwrap();
            start_fade(
                ui,
                FadeTransition {
                    hide: play_leave,
                    prep: lan_ui_prep,
                    finish: lan_ui_finish,
                },
            );
        } else {
            play.resource_mut::<MatchDone>().visual.show();
            *play.resource_mut() = PlayState::MatchDone;
        }
        #[cfg(target_arch = "wasm32")]
        {
            play.resource_mut::<MatchDone>().visual.show();
            *play.resource_mut() = PlayState::MatchDone;
        }
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
                hide: play_leave,
                prep: team_select_prep,
                finish: team_select_finish,
            },
        );
    };
    let play_again = || {
        // We can use the ui session here for convenience since
        // this isn't in a network game.
        let mut sessions = play.resource_mut::<Sessions>();
        let ui = sessions.get_world(session::UI).unwrap();
        start_fade(
            ui,
            FadeTransition {
                hide: play_reset,
                prep: play_offline_prep,
                finish: play_offline_finish,
            },
        );
    };
    let to_splash = || {
        let mut sessions = play.resource_mut::<Sessions>();
        let ui = sessions.get_world(session::UI).unwrap();
        start_fade(
            ui,
            FadeTransition {
                hide: play_leave,
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

// TODO: Use PlayerEntSigns on the three functions below.
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

fn play_music(root: Root<Data>, mut audio: ResMut<AudioCenter>) {
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
}

#[cfg(not(target_arch = "wasm32"))]
fn handle_disconnections(play: &World) -> Option<()> {
    use bones_framework::networking::*;
    if let Some(disconnects) = play.get_resource::<DisconnectedPlayers>() {
        if !disconnects.disconnected_players.is_empty() {
            let mut sessions = play.resource_mut::<Sessions>();
            let ui = sessions.get_world(session::UI).unwrap();
            ui.resource_mut::<LanUI>().state = LanUIState::Disconnected;
            start_fade(
                ui,
                FadeTransition {
                    hide: play_leave,
                    prep: lan_ui_prep,
                    finish: lan_ui_finish,
                },
            );
            return Some(());
        }
    }
    None
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
