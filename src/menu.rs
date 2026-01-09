use super::*;

#[cfg(not(target_arch = "wasm32"))]
mod network;
#[cfg(not(target_arch = "wasm32"))]
pub use network::*;

#[derive(HasSchema, Clone, Copy, Default, PartialEq, Eq)]
pub enum MenuState {
    #[default]
    Splash,
    Settings,
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
pub struct MenuPlugin;
impl SessionPlugin for MenuPlugin {
    fn install(self, session: &mut SessionBuilder) {
        session.init_resource::<MenuState>();
        session.init_resource::<FadeTransition>();

        session.install_plugin(Splash {
            visual: Visual::new_shown(),
            ..Default::default()
        });
        session.install_plugin(SettingsUi::default());
        session.install_plugin(HowToPlay::default());
        session.install_plugin(Fade::new(
            0.5,
            0.15,
            0.5,
            Color::BLACK,
            egui::Order::Tooltip,
        ));
        session.install_plugin(TeamSelect::default());
        session.install_plugin(Pause::default());

        #[cfg(not(target_arch = "wasm32"))]
        {
            session.install_plugin(
                MatchmakerPlugin::new(MATCHMAKER_SERVICE_NAME)
                    .refresh(1.0)
                    .player_count(2),
            );
            session.install_plugin(LanSelect::default());
            session.install_plugin(LanUI::default());
            session.install_plugin(NetworkQuit::default());
        }

        session.add_startup_system(
            |root: Root<Data>, mut storage: ResMut<Storage>, mut audio: ResMut<AudioCenter>| {
                let Settings {
                    sfx_volume,
                    music_volume,
                } = storage.get_or_insert_default::<Settings>();

                audio.set_volume_scales(1.0, music_volume.scale(), sfx_volume.scale());
                audio.play_music_advanced(
                    *root.sound.menu_music,
                    root.sound.menu_music.volume(),
                    true,
                    false,
                    0.0,
                    1.0,
                    true,
                );
            },
        );
        session.add_system_to_stage(First, update_menu);
    }
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
    let settings_output = world.resource_mut::<SettingsUi>().process_ui(world);
    let team_select_output = world.resource_mut::<TeamSelect>().process_ui(world);
    let pause_ouptut = world.resource_mut::<Pause>().process_ui(world);
    // TODO: use the `LanSelect` pattern for rendering and processing all the ui elements
    //...

    let menu_state = *world.resource::<MenuState>();
    match menu_state {
        MenuState::FadeTransition => fade_transition(world),
        MenuState::Splash => splash_update(world),
        MenuState::HowToPlay => how_to_play_update(world),
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
                network_quit_transition(world, output)
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        MenuState::LanSelect => {
            if let Some(output) = world
                .resource_mut::<LanSelect>()
                .process_input(world)
                .or(lan_select)
            {
                lan_select_transition(world, output);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        MenuState::Lan => {
            let output = world
                .resource_mut::<LanUI>()
                .process_input(world)
                .or(lan_ui);
            // TODO: should probably take the `None` possibility
            // on the other ui transitions as well for standardization.
            // That will let the function decide to go dormant without input.
            lan_ui_transition(world, output)
        }
    }
}
pub fn network_quit_transition(world: &World, output: NetworkQuitOutput) {
    match output {
        NetworkQuitOutput::Quit => {
            start_fade(
                world,
                FadeTransition {
                    hide: play_leave,
                    prep: splash_prep,
                    finish: splash_finish,
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
        // TODO: maybe do these on fade_wait so you see the
        // results fading in immediatelly.
        (transition.hide)(ui);
        (transition.prep)(ui);
    }
    if fade.fade_in.just_finished() {
        (transition.finish)(ui);
    }
}
pub fn start_fade(world: &World, transition: FadeTransition) {
    let mut fade = world.resource_mut::<Fade>();
    if !fade.finished() {
        tracing::warn!("fade interupted, restarting.");
    }
    fade.restart();
    *world.resource_mut() = MenuState::FadeTransition;
    *world.resource_mut() = transition;
}
pub fn splash_hide(world: &World) {
    world.resource_mut::<Splash>().visual.hide();
}
pub fn splash_prep(world: &World) {
    world.resource_mut::<Splash>().visual.show();
}
pub fn splash_finish(world: &World) {
    *world.resource_mut() = MenuState::Splash;
}
pub fn team_select_hide(world: &World) {
    world.resource_mut::<TeamSelect>().visible = false;
}
pub fn settings_hide(world: &World) {
    world.resource_mut::<SettingsUi>().visible = false;
}
pub fn settings_prep(world: &World) {
    *world.resource_mut() = SettingsUi {
        visible: true,
        ..Default::default()
    };
    world.resource::<EguiCtx>().clear_animations();
}
pub fn settings_finish(world: &World) {
    *world.resource_mut() = MenuState::Settings;
}
pub fn team_select_prep(world: &World) {
    *world.resource_mut() = TeamSelect {
        visible: true,
        ..Default::default()
    };
    world.resource::<EguiCtx>().clear_animations();
}
pub fn team_select_finish(world: &World) {
    *world.resource_mut() = MenuState::TeamSelect;
}
pub fn how_to_play_hide(world: &World) {
    *world.resource_mut() = HowToPlay::Hidden;
}
pub fn how_to_play_prep(world: &World) {
    *world.resource_mut() = HowToPlay::GameOverview;
    world.resource::<EguiCtx>().clear_animations();
}
pub fn how_to_play_finish(world: &World) {
    *world.resource_mut() = MenuState::HowToPlay;
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

    tracing::info!("fade_out, recreating PLAY session; assignments:{player_signs:?}");

    sessions.create_play(PlayMode::Offline(player_signs));
    *ui.resource_mut() = MenuState::InGame;
    *ui.resource_mut() = Pause::Hidden;
}

pub fn splash_update(ui: &World) {
    let mut splash = ui.resource_mut::<Splash>();
    let inputs = ui.resource::<LocalInputs>();

    let proceed = move |state: SplashState| match state {
        SplashState::Offline => start_fade(
            ui,
            FadeTransition {
                hide: splash_hide,
                prep: team_select_prep,
                finish: team_select_finish,
            },
        ),
        #[cfg(not(target_arch = "wasm32"))]
        SplashState::Lan => {
            start_fade(
                ui,
                FadeTransition {
                    hide: splash_hide,
                    prep: lan_select_prep,
                    finish: lan_select_finish,
                },
            );
        }
        SplashState::HowToPlay => {
            start_fade(
                ui,
                FadeTransition {
                    hide: splash_hide,
                    prep: how_to_play_prep,
                    finish: how_to_play_finish,
                },
            );
        }
        SplashState::Settings => {
            start_fade(
                ui,
                FadeTransition {
                    hide: splash_hide,
                    prep: settings_prep,
                    finish: settings_finish,
                },
            );
        }
    };

    if let Some(interact) = splash.interact {
        proceed(interact);
        return;
    }

    for (_gamepad, input) in inputs.iter() {
        if input.menu_up.just_pressed() {
            splash.cycle_up();
            return;
        }
        if input.menu_down.just_pressed() {
            splash.cycle_down();
            return;
        }
        if input.menu_left.just_pressed() {
            splash.cycle_left();
            return;
        }
        if input.menu_right.just_pressed() {
            splash.cycle_right();
            return;
        }
        if input.menu_select.just_pressed() {
            proceed(splash.state);
            return;
        }
    }
}
pub fn how_to_play_update(ui: &World) {
    let mut howtoplay = ui.resource_mut::<HowToPlay>();
    let inputs = ui.resource::<LocalInputs>();

    for (_gamepad, input) in inputs.iter() {
        if input.menu_back.just_pressed() {
            start_fade(
                ui,
                FadeTransition {
                    hide: how_to_play_hide,
                    prep: splash_prep,
                    finish: splash_finish,
                },
            );
        }
        if input.menu_left.just_pressed() {
            howtoplay.left();
        }
        if input.menu_right.just_pressed() {
            howtoplay.right();
        }
    }
}
pub fn settings_transition(world: &World, _output: SettingsOutput) {
    start_fade(
        world,
        FadeTransition {
            hide: settings_hide,
            prep: splash_prep,
            finish: splash_finish,
        },
    );
}
pub fn team_select_transition(world: &World, output: TeamSelectOutput) {
    match output {
        TeamSelectOutput::PlayersInfo(..) => start_fade(
            world,
            FadeTransition {
                hide: team_select_hide,
                prep: play_offline_prep,
                finish: |_| {},
            },
        ),
        TeamSelectOutput::Exit => start_fade(
            world,
            FadeTransition {
                hide: team_select_hide,
                prep: splash_prep,
                finish: splash_finish,
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
            session.active = true;
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
            session.active = false;
        }
        PauseOutput::Restart => {
            start_fade(
                world,
                FadeTransition {
                    hide: play_reset,
                    prep: play_offline_prep,
                    finish: |_| {},
                },
            );
        }
        PauseOutput::Quit => {
            start_fade(
                world,
                FadeTransition {
                    hide: play_leave,
                    prep: splash_prep,
                    finish: splash_finish,
                },
            );
        }
    }
}
