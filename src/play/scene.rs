use super::*;

#[cfg(not(target_arch = "wasm32"))]
use bones_framework::networking::NetworkMatchSocket;

#[derive(HasSchema, Clone)]
pub enum PlayMode {
    #[cfg(not(target_arch = "wasm32"))]
    Online {
        socket: NetworkMatchSocket,
        service_type: ServiceType,
    },
    Offline(PlayersInfo),
}
impl Default for PlayMode {
    fn default() -> Self {
        Self::Offline(default())
    }
}
#[derive(HasSchema, Debug, Clone, Default)]
pub struct PlayersInfo {
    pub a1: PlayerInfo,
    pub a2: PlayerInfo,
    pub b1: PlayerInfo,
    pub b2: PlayerInfo,
}

/// This is the player spawn information.
#[derive(HasSchema, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerInfo {
    CPU,
    Network,
    Local {
        /// The user join index for display purposes,
        /// `0` being P1, `1` being P2, and so on.
        number: usize,
        /// The input source of the player.
        // TODO: Check if this field should be separated from this struct since
        // it is not used in the player spawning.
        source: SingleSource,
        /// Whether or not this player is being controlled with
        /// dual stick controls.
        dual_stick: bool,
    },
}
impl Default for PlayerInfo {
    fn default() -> Self {
        // This is the best default for dev testing
        Self::Local {
            number: 0,
            source: SingleSource::Gamepad(0),
            dual_stick: true,
        }
    }
}

/// The minimal requirements for the [`PLAY`] session.
///
/// Includes the runner selection, input resources, and entity spawning.
/// Also handles camera sizing and path2d toggling.
pub struct ScenePlugin {
    pub mode: PlayMode,
}
impl SessionPlugin for ScenePlugin {
    fn install(self, session: &mut SessionBuilder) {
        match &self.mode {
            PlayMode::Offline(players_info) => {
                session.runner = offline_session_runner(players_info.clone());
            }
            #[cfg(not(target_arch = "wasm32"))]
            PlayMode::Online {
                socket,
                service_type,
            } => {
                session.runner = lan_session_runner(socket, service_type);
            }
        };
        session.insert_resource(self.mode);
        session.insert_resource(Mapping); // TODO: This will probably get removed after a bones update.
        session.init_resource::<PlayTeamInputs>();
        session.install_plugin(Path2dTogglePlugin);

        session.add_system_to_stage(Update, toggle_debug_lines);

        session.add_startup_system(spawn::scene);
        session.add_startup_system(hide_debug_lines);
        session.add_system_to_stage(Last, |mut inputs: ResMut<PlayTeamInputs>| {
            inputs.advance_frame()
        });
    }
}

pub fn toggle_debug_lines(inputs: Res<KeyboardInputs>, mut toggles: CompMut<Path2dToggle>) {
    for input in inputs.key_events.iter() {
        if input.button_state == ButtonState::Pressed && input.key_code == Set(KeyCode::F3) {
            for toggle in toggles.iter_mut() {
                toggle.hide = !toggle.hide;
            }
        }
    }
}

pub fn hide_debug_lines(mut toggles: CompMut<Path2dToggle>) {
    for toggle in toggles.iter_mut() {
        toggle.hide = true;
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn lan_session_runner(
    socket: &bones_framework::networking::NetworkMatchSocket,
    service_type: &ServiceType,
) -> Box<dyn SessionRunner> {
    use bones_framework::networking::{GgrsSessionRunner, GgrsSessionRunnerInfo};

    let mut runner = GgrsSessionRunner::<PlayTeamNetworkInputConfig>::new(
        Some(60.0),
        GgrsSessionRunnerInfo::new(socket.ggrs_socket(), Some(7), Some(2), 0),
    );
    match service_type {
        ServiceType::OnePlayer(p1) => {
            runner
                .input_collector
                .set_sources(SingleSource::Gamepad(*p1), SingleSource::Gamepad(*p1));
        }
        ServiceType::TwoPlayer(p1, p2) => {
            runner.input_collector.set_sources(*p1, *p2);
        }
    }
    Box::new(runner)
}
pub fn offline_session_runner(players_info: PlayersInfo) -> Box<dyn SessionRunner> {
    let PlayersInfo { a1, a2, b1, b2 } = players_info;
    Box::new(OfflineRunner {
        collectors: [
            PlayTeamInputCollector::new(
                match a1 {
                    PlayerInfo::CPU => SingleSource::CPU(PlayerSlot::A1),
                    PlayerInfo::Local { source, .. } => source,
                    PlayerInfo::Network => unreachable!(),
                },
                match a2 {
                    PlayerInfo::CPU => SingleSource::CPU(PlayerSlot::A2),
                    PlayerInfo::Local { source, .. } => source,
                    PlayerInfo::Network => unreachable!(),
                },
            ),
            PlayTeamInputCollector::new(
                match b1 {
                    PlayerInfo::CPU => SingleSource::CPU(PlayerSlot::B1),
                    PlayerInfo::Local { source, .. } => source,
                    PlayerInfo::Network => unreachable!(),
                },
                match b2 {
                    PlayerInfo::CPU => SingleSource::CPU(PlayerSlot::B2),
                    PlayerInfo::Local { source, .. } => source,
                    PlayerInfo::Network => unreachable!(),
                },
            ),
        ],
        ..Default::default()
    })
}
