use super::*;

#[derive(HasSchema, Clone, Copy, Default, PartialEq, Eq)]
pub enum MenuState {
    #[default]
    Splash,
    HowToPlay,
    FadeTransition,
    TeamSelect,
    InGame,
}
pub struct MenuPlugin;
impl SessionPlugin for MenuPlugin {
    fn install(self, session: &mut SessionBuilder) {
        session.init_resource::<MenuState>();
        session.init_resource::<FadeTransition>();

        session.install_plugin(Splash::Offline);
        session.install_plugin(HowToPlay::default());
        session.install_plugin(Fade::new(0.7, 0.5, Color::BLACK, egui::Order::Foreground));
        session.install_plugin(TeamSelect::default());
        session.install_plugin(Pause::default());
        session.add_startup_system(|root: Root<Data>, mut audio: ResMut<AudioCenter>| {
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
        session.add_system_to_stage(First, update_menu);
        session.add_system_to_stage(First, update_pause);
    }
}

pub fn update_pause(ui: &World) {
    if *ui.resource::<MenuState>() == MenuState::FadeTransition
        || ui
            .resource_mut::<Sessions>()
            .get_mut(session::PLAY)
            .is_none()
    {
        return;
    };
    let mut pause = ui.resource_mut::<Pause>();
    let local_inputs = ui.resource::<LocalInputs>();

    let unpause = || {
        let mut sessions = ui.resource_mut::<Sessions>();
        let session = sessions.get_mut(session::PLAY).unwrap();
        session
            .world
            .resource_mut::<Countdown>()
            .visual
            .remove_hide();
        session
            .world
            .resource_mut::<MatchDone>()
            .visual
            .remove_hide();
        session
            .world
            .resource_mut::<ScoreDisplay>()
            .visual
            .remove_hide();
        session
            .world
            .resource_mut::<WinnerBanner>()
            .visual
            .remove_hide();
        session.active = true;
    };

    for (_gamepad, input) in local_inputs.iter() {
        if input.down.just_pressed() {
            pause.cycle()
        }
        if input.up.just_pressed() {
            pause.cycle();
            pause.cycle();
        }
        if input.start.just_pressed() {
            match *pause {
                Pause::Hidden => {
                    let mut sessions = ui.resource_mut::<Sessions>();
                    let session = sessions.get_mut(session::PLAY).unwrap();
                    session.world.resource_mut::<Countdown>().visual.add_hide();
                    session.world.resource_mut::<MatchDone>().visual.add_hide();
                    session
                        .world
                        .resource_mut::<ScoreDisplay>()
                        .visual
                        .add_hide();
                    session
                        .world
                        .resource_mut::<WinnerBanner>()
                        .visual
                        .add_hide();
                    session.active = false;
                    *pause = Pause::Continue;
                }
                Pause::Continue | Pause::Restart | Pause::Quit => {
                    unpause();
                    *pause = Pause::Hidden;
                }
            }
        }
        if input.south.just_pressed() {
            match *pause {
                Pause::Continue => {
                    unpause();
                    *pause = Pause::Hidden;
                }
                Pause::Restart => {
                    start_fade(
                        ui,
                        FadeTransition {
                            hide: play_hide,
                            prep: play_prep,
                            finish: play_finish,
                        },
                    );
                    *pause = Pause::Hidden;
                }
                Pause::Quit => {
                    start_fade(
                        ui,
                        FadeTransition {
                            hide: play_hide,
                            prep: splash_prep,
                            finish: splash_finish,
                        },
                    );
                    *pause = Pause::Hidden;
                }
                Pause::Hidden => {}
            }
        }
    }
}

pub fn update_menu(world: &World) {
    let game_state = *world.resource::<MenuState>();
    match game_state {
        MenuState::FadeTransition => fade_transition(world),
        MenuState::Splash => splash_update(world),
        MenuState::HowToPlay => how_to_play_update(world),
        MenuState::TeamSelect => team_select_update(world),
        MenuState::InGame => {}
    }
}

#[derive(HasSchema, Clone)]
pub struct FadeTransition {
    /// Makes the associated ui elements invisible while the screen is blank.
    pub hide: fn(&World),
    /// Makes the associated ui elements visible while the screen is blank to show up later.
    pub prep: fn(&World),
    /// Makes the changes that gives control over the associated ui elements.
    pub finish: fn(&World),
}
impl Default for FadeTransition {
    fn default() -> Self {
        Self {
            hide: |_| {},
            prep: |_| {},
            finish: |_| {},
        }
    }
}
pub fn fade_transition(ui: &World) {
    let fade = ui.resource::<Fade>();
    let transition = ui.resource::<FadeTransition>();

    if fade.fade_out.just_finished() {
        (transition.hide)(ui);
        (transition.prep)(ui);
    }
    if fade.fade_in.just_finished() {
        (transition.finish)(ui);
    }
}
pub fn start_fade(world: &World, transition: FadeTransition) {
    world.resource_mut::<Fade>().restart();
    *world.resource_mut() = MenuState::FadeTransition;
    *world.resource_mut() = transition;
}
pub fn splash_hide(world: &World) {
    *world.resource_mut() = Splash::Hidden;
}
pub fn splash_prep(world: &World) {
    *world.resource_mut() = Splash::Offline;
}
pub fn splash_finish(world: &World) {
    *world.resource_mut() = MenuState::Splash;
}
pub fn team_select_hide(world: &World) {
    world.resource_mut::<TeamSelect>().visible = false;
}
pub fn team_select_prep(world: &World) {
    world.resource_mut::<TeamSelect>().visible = true;
}
pub fn team_select_finish(world: &World) {
    *world.resource_mut() = MenuState::TeamSelect;
}
pub fn how_to_play_hide(world: &World) {
    *world.resource_mut() = HowToPlay::Hidden;
}
pub fn how_to_play_prep(world: &World) {
    *world.resource_mut() = HowToPlay::GameOverview;
}
pub fn how_to_play_finish(world: &World) {
    *world.resource_mut() = MenuState::HowToPlay;
}
pub fn play_hide(ui: &World) {
    let mut sessions = ui.resource_mut::<Sessions>();
    sessions.delete_play();
}
pub fn play_prep(ui: &World) {
    let mut sessions = ui.resource_mut::<Sessions>();
    let player_signs = ui
        .resource::<TeamSelect>()
        .get_player_signs()
        .unwrap_or_else(|| {
            tracing::warn!("gamepad assignments were not made, defaulting to id 0 for all players");
            default()
        });

    tracing::info!("fade_out, recreating PLAY session; assignments:{player_signs:?}");

    sessions.create_play(PlayMode::Offline(player_signs));
}
pub fn play_finish(ui: &World) {
    *ui.resource_mut() = MenuState::InGame;
    let mut sessions = ui.resource_mut::<Sessions>();
    tracing::info!("fade_in, starting countdown");
    sessions
        .get_world(session::PLAY)
        .unwrap()
        .resource_mut::<Countdown>()
        .restart();
}

pub fn splash_update(ui: &World) {
    let mut splash = ui.resource_mut::<Splash>();
    let inputs = ui.resource::<LocalInputs>();

    for (_gamepad, input) in inputs.iter() {
        if input.up.just_pressed() {
            splash.cycle_up();
        }
        if input.down.just_pressed() {
            splash.cycle_down();
        }
        if input.south.just_pressed() {
            match *splash {
                Splash::Offline => {
                    start_fade(
                        ui,
                        FadeTransition {
                            hide: splash_hide,
                            prep: team_select_prep,
                            finish: team_select_finish,
                        },
                    );
                    return;
                }
                Splash::HowToPlay => {
                    start_fade(
                        ui,
                        FadeTransition {
                            hide: splash_hide,
                            prep: how_to_play_prep,
                            finish: how_to_play_finish,
                        },
                    );
                }
                Splash::Hidden => todo!(),
            }
        }
    }
}
pub fn how_to_play_update(ui: &World) {
    let mut howtoplay = ui.resource_mut::<HowToPlay>();

    let inputs = ui.resource::<LocalInputs>();

    for (_gamepad, input) in inputs.iter() {
        if input.west.just_pressed() {
            start_fade(
                ui,
                FadeTransition {
                    hide: how_to_play_hide,
                    prep: splash_prep,
                    finish: splash_finish,
                },
            );
        }
        match *howtoplay {
            HowToPlay::GameOverview => {
                if input.right.just_pressed() {
                    *howtoplay = HowToPlay::SingleStickControls;
                }
            }
            HowToPlay::DualStickControls => {
                if input.left.just_pressed() {
                    *howtoplay = HowToPlay::SingleStickControls;
                }
            }
            HowToPlay::SingleStickControls => {
                if input.left.just_pressed() {
                    *howtoplay = HowToPlay::GameOverview;
                }
                if input.right.just_pressed() {
                    *howtoplay = HowToPlay::DualStickControls;
                }
            }
            HowToPlay::Hidden => {}
        }
    }
}
pub fn team_select_update(ui: &World) {
    let assignments = ui.resource_mut::<TeamSelect>().get_player_signs();
    let local_inputs = ui.resource::<LocalInputs>();
    let asset_server = ui.asset_server();
    let root = asset_server.root::<Data>();

    for (gamepad, input) in local_inputs.iter() {
        if input.start.just_pressed() && assignments.is_some() {
            start_fade(
                ui,
                FadeTransition {
                    hide: team_select_hide,
                    prep: play_prep,
                    finish: play_finish,
                },
            );
            return;
        }
        if input.start.just_pressed()
            || input.north.just_pressed()
            || input.east.just_pressed()
            || input.south.just_pressed()
            || input.west.just_pressed()
            || input.left_bump.just_pressed()
            || input.right_bump.just_pressed()
        {
            ui.resource_mut::<TeamSelect>().add_gamepad(*gamepad);
            ui.resource_mut::<GamepadsRumble>().set_rumble(
                *gamepad,
                GamepadRumbleIntensity::LIGHT_BOTH,
                0.2,
            );
        }
        if input.south.just_pressed() {
            ui.resource_mut::<TeamSelect>().ready_gamepad(*gamepad);
        }
        if input.west.just_pressed() {
            ui.resource_mut::<TeamSelect>().reverse_gamepad(*gamepad);
        }
        if input.west.just_held(root.menu.team_select.back_buffer) {
            start_fade(
                ui,
                FadeTransition {
                    hide: team_select_hide,
                    prep: splash_prep,
                    finish: splash_finish,
                },
            );
        }
        if input.left.just_pressed() {
            ui.resource_mut::<TeamSelect>().left_gamepad(*gamepad);
        }
        if input.right.just_pressed() {
            ui.resource_mut::<TeamSelect>().right_gamepad(*gamepad);
        }
        if input.right_bump.just_held(20) && input.left_bump.just_held(20) {
            start_fade(
                ui,
                FadeTransition {
                    hide: team_select_hide,
                    prep: play_prep,
                    finish: play_finish,
                },
            );
            return;
        }
    }
}
