use super::*;

#[derive(HasSchema, Clone, Copy, Default, PartialEq, Eq)]
pub enum MenuState {
    #[default]
    Splash,
    Settings,
    Credits,
    HowToPlay,
    FadeTransition,
    TeamSelect,
    InGame,
    #[cfg(not(target_arch = "wasm32"))]
    LanSelect,
    #[cfg(not(target_arch = "wasm32"))]
    Lan,
    #[cfg(not(target_arch = "wasm32"))]
    InNetworkGame,
}

pub fn update_menu(world: &World) {
    // Each ui element has an output returned that we may use if the element
    // controls are active depending on the game state in this scenario.
    #[cfg(not(target_arch = "wasm32"))]
    let lan_select = world.resource_mut::<LanSelect>().process_ui(world);
    #[cfg(not(target_arch = "wasm32"))]
    let lan_ui = world.resource_mut::<LanUI>().process_ui(world);
    #[cfg(not(target_arch = "wasm32"))]
    let network_quit = world.resource_mut::<NetworkQuit>().process_ui(world);

    let splash_output = world.resource_mut::<Splash>().process_ui(world);
    let howtoplay_output = world.resource_mut::<HowToPlay>().process_ui(world);
    let settings_output = world.resource_mut::<SettingsUi>().process_ui(world);
    let credits_output = world.resource_mut::<CreditsUi>().process_ui(world);
    let team_select_output = world.resource_mut::<TeamSelect>().process_ui(world);
    let pause_ouptut = world.resource_mut::<Pause>().process_ui(world);

    let menu_state = *world.resource::<MenuState>();
    match menu_state {
        MenuState::FadeTransition => fade_transition(world),
        MenuState::Splash => {
            if let Some(output) = world
                .resource_mut::<Splash>()
                .process_input(world)
                .or(splash_output)
            {
                splash_transition(world, output)
            }
        }
        MenuState::HowToPlay => {
            if let Some(output) = world
                .resource_mut::<HowToPlay>()
                .process_input(world)
                .or(howtoplay_output)
            {
                how_to_play_transition(world, output)
            }
        }
        MenuState::Credits => {
            if let Some(output) = world
                .resource_mut::<CreditsUi>()
                .process_input(world)
                .or(credits_output)
            {
                credits_transition(world, output)
            }
        }
        MenuState::Settings => {
            if let Some(output) = world
                .resource_mut::<SettingsUi>()
                .process_input(world)
                .or(settings_output)
            {
                settings_transition(world, output)
            }
        }
        MenuState::TeamSelect => {
            if let Some(output) = world
                .resource_mut::<TeamSelect>()
                .process_input(world)
                .or(team_select_output)
            {
                team_select_transition(world, output)
            }
        }
        MenuState::InGame => {
            if let Some(output) = world
                .resource_mut::<Pause>()
                .process_input(world)
                .or(pause_ouptut)
            {
                pause_transition(world, output)
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        MenuState::InNetworkGame => {
            if let Some(output) = world
                .resource_mut::<NetworkQuit>()
                .process_input(world)
                .or(network_quit)
            {
                network::network_quit_transition(world, output)
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        MenuState::LanSelect => {
            if let Some(output) = world
                .resource_mut::<LanSelect>()
                .process_input(world)
                .or(lan_select)
            {
                network::lan_select_transition(world, output);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        MenuState::Lan => {
            let output = world
                .resource_mut::<LanUI>()
                .process_input(world)
                .or(lan_ui);
            network::lan_ui_transition(world, output)
        }
    }
}

#[derive(HasSchema, Clone)]
pub struct FadeTransition {
    /// Makes the associated ui elements invisible while the screen is blank.
    pub hide: fn(&World),
    /// Makes the associated ui elements visible while the screen is blank to show up later.
    pub prep: fn(&World),
    /// Makes the change to the [`MenuState`] resource that gives control over the associated ui elements.
    pub finish: MenuState,
}
impl Default for FadeTransition {
    fn default() -> Self {
        Self {
            hide: |_| {},
            prep: |_| {},
            finish: MenuState::default(),
        }
    }
}

pub fn fade_transition(ui: &World) {
    let fade = ui.resource::<Fade>();
    let transition = ui.resource::<FadeTransition>();

    if fade.fade_wait.just_finished() {
        (transition.hide)(ui);
        (transition.prep)(ui);

        ui.resource::<EguiCtx>().clear_animations();
        ui.resource::<EguiCtx>()
            .data_mut(|w| w.remove::<()>(egui::Id::new("back_button_pressed")));
    }
    if fade.fade_in.just_finished() {
        *ui.resource_mut() = transition.finish;
    }
}

pub fn start_fade(world: &World, transition: FadeTransition) {
    let mut fade = world.resource_mut::<Fade>();
    if !fade.finished() && !fade.elapsed().is_zero() {
        tracing::warn!("fade interupted, restarting.");
    }
    fade.restart();
    *world.resource_mut() = MenuState::FadeTransition;
    *world.resource_mut() = transition;
}

pub fn play_leave(ui: &World) {
    *ui.resource_mut() = Pause::Disabled;

    #[cfg(not(target_arch = "wasm32"))]
    {
        ui.resource_mut::<Matchmaker>().lan_cancel();
        ui.resource_mut::<NetworkQuit>().visible = false;
    }
    let mut sessions = ui.resource_mut::<Sessions>();
    sessions.delete_play();
}

pub fn play_reset(ui: &World) {
    *ui.resource_mut() = Pause::Disabled;

    let mut sessions = ui.resource_mut::<Sessions>();
    sessions
        .get_mut(PLAY)
        .unwrap()
        .world
        .resources
        .insert(ResetWorld {
            reset: true,
            reset_resources: default(),
        });
}

pub fn play_offline_prep(ui: &World) {
    let mut sessions = ui.resource_mut::<Sessions>();
    let player_signs = ui
        .resource::<TeamSelect>()
        .get_player_signs()
        .unwrap_or_else(|| {
            tracing::warn!("gamepad assignments were not made, defaulting to id 0 for all players");
            default()
        });

    tracing::debug!(?player_signs, "creating offline PLAY session");

    sessions.create_play(PlayMode::Offline(player_signs));
    *ui.resource_mut() = MenuState::InGame;
    *ui.resource_mut() = Pause::Hidden;
}

pub fn splash_transition(ui: &World, output: SplashOutput) {
    match output {
        SplashState::Offline => start_fade(
            ui,
            FadeTransition {
                hide: Splash::hide_resource,
                prep: TeamSelect::show_resource,
                finish: MenuState::TeamSelect,
            },
        ),
        #[cfg(not(target_arch = "wasm32"))]
        SplashState::Lan => {
            start_fade(
                ui,
                FadeTransition {
                    hide: Splash::hide_resource,
                    prep: LanSelect::show_resource,
                    finish: MenuState::LanSelect,
                },
            );
        }
        SplashState::HowToPlay => {
            start_fade(
                ui,
                FadeTransition {
                    hide: Splash::hide_resource,
                    prep: HowToPlay::show_resource,
                    finish: MenuState::HowToPlay,
                },
            );
        }
        SplashState::Settings => {
            start_fade(
                ui,
                FadeTransition {
                    hide: Splash::hide_resource,
                    prep: SettingsUi::show_resource,
                    finish: MenuState::Settings,
                },
            );
        }
        SplashState::Credits => {
            start_fade(
                ui,
                FadeTransition {
                    hide: Splash::hide_resource,
                    prep: CreditsUi::show_resource,
                    finish: MenuState::Credits,
                },
            );
        }
    };
}
pub fn how_to_play_transition(world: &World, _output: HowToPlayOutput) {
    start_fade(
        world,
        FadeTransition {
            hide: HowToPlay::hide_resource,
            prep: Splash::show_resource,
            finish: MenuState::Splash,
        },
    );
}
pub fn settings_transition(world: &World, _output: SettingsOutput) {
    start_fade(
        world,
        FadeTransition {
            hide: SettingsUi::hide_resource,
            prep: Splash::show_resource,
            finish: MenuState::Splash,
        },
    );
}
pub fn credits_transition(world: &World, _output: CreditsOutput) {
    start_fade(
        world,
        FadeTransition {
            hide: CreditsUi::hide_resource,
            prep: Splash::show_resource,
            finish: MenuState::Splash,
        },
    );
}
pub fn team_select_transition(world: &World, output: TeamSelectOutput) {
    match output {
        TeamSelectOutput::PlayersInfo(..) => start_fade(
            world,
            FadeTransition {
                hide: TeamSelect::hide_resource,
                prep: play_offline_prep,
                finish: MenuState::InGame,
            },
        ),
        TeamSelectOutput::Exit => start_fade(
            world,
            FadeTransition {
                hide: TeamSelect::hide_resource,
                prep: Splash::show_resource,
                finish: MenuState::Splash,
            },
        ),
    }
}
pub fn pause_transition(world: &World, output: PauseOutput) {
    match output {
        PauseOutput::Hide => {
            let mut sessions = world.resource_mut::<Sessions>();
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
            session.world.resources.remove::<PauseGame>();
        }
        PauseOutput::Show => {
            let mut sessions = world.resource_mut::<Sessions>();
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
            session.world.resources.insert(PauseGame);
        }
        PauseOutput::Restart => {
            start_fade(
                world,
                FadeTransition {
                    hide: play_reset,
                    prep: play_offline_prep,
                    finish: MenuState::InGame,
                },
            );
        }
        PauseOutput::Quit => {
            start_fade(
                world,
                FadeTransition {
                    hide: play_leave,
                    prep: Splash::show_resource,
                    finish: MenuState::Splash,
                },
            );
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub mod network {
    use super::*;

    pub fn lan_ui_hide(world: &World) {
        world.resource_mut::<Matchmaker>().disable_search();
        world.resource_mut::<LanUI>().visible = false;
    }
    pub fn lan_ui_leave(world: &World) {
        world.resource_mut::<Matchmaker>().lan_cancel();
        world.resource_mut::<Matchmaker>().disable_search();
        world.resource_mut::<LanUI>().visible = false;
    }
    pub fn lan_ui_prep(world: &World) {
        world.resource_mut::<Matchmaker>().enable_search();
        world.resource_mut::<LanUI>().visible = true;
    }
    pub fn play_online_prep(ui: &World) {
        let socket = ui.resource::<Matchmaker>().network_match_socket().unwrap();
        let service_type = *ui.resource::<ServiceType>();
        let mut sessions = ui.resource_mut::<Sessions>();

        sessions.create_play(PlayMode::Online {
            socket,
            service_type,
        });
        *ui.resource_mut() = MenuState::InNetworkGame;
    }
    pub fn lan_select_transition(world: &World, output: LanSelectOutput) {
        match output {
            LanSelectOutput::Exit => {
                start_fade(
                    world,
                    FadeTransition {
                        hide: LanSelect::hide_resource,
                        prep: Splash::show_resource,
                        finish: MenuState::Splash,
                    },
                );
            }
            LanSelectOutput::ServiceType(service) => {
                world.resource_mut::<LanUI>().service = service;
                start_fade(
                    world,
                    FadeTransition {
                        hide: LanSelect::show_resource,
                        prep: lan_ui_prep,
                        finish: MenuState::Lan,
                    },
                );
            }
        }
    }
    pub fn lan_ui_transition(world: &World, output: Option<LanUIOutput>) {
        let lan_ui = world.resource::<LanUI>();
        let mut matchmaker = world.resource_mut::<Matchmaker>();

        if matchmaker.network_match_socket().is_some() {
            world.resources.insert(lan_ui.service);
            start_fade(
                world,
                FadeTransition {
                    hide: lan_ui_hide,
                    prep: play_online_prep,
                    finish: MenuState::InNetworkGame,
                },
            );
            return;
        }
        let Some(output) = output else { return };
        match output {
            LanUIOutput::HostCancel => {
                if matchmaker.is_waiting() {
                    matchmaker.lan_cancel();
                } else {
                    matchmaker.lan_host();
                }
            }
            LanUIOutput::Server(i) => {
                if let Some(server) = matchmaker.lan_servers().get(i).cloned() {
                    matchmaker.lan_join(&server);
                }
            }
            LanUIOutput::Exit => start_fade(
                world,
                FadeTransition {
                    hide: lan_ui_leave,
                    prep: Splash::show_resource,
                    finish: MenuState::Splash,
                },
            ),
        }
    }
    pub fn network_quit_transition(world: &World, output: NetworkQuitOutput) {
        match output {
            NetworkQuitOutput::Quit => {
                start_fade(
                    world,
                    FadeTransition {
                        hide: play_leave,
                        prep: Splash::show_resource,
                        finish: MenuState::Splash,
                    },
                );
            }
            NetworkQuitOutput::Show => {
                let mut sessions = world.resource_mut::<Sessions>();
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
            }
            NetworkQuitOutput::Hide => {
                let mut sessions = world.resource_mut::<Sessions>();
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
            }
        }
    }
}
