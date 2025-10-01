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
#[derive(HasSchema, Debug, Clone)]
pub struct PlayersInfo {
    pub team_a: TeamInfo,
    pub team_b: TeamInfo,
}
impl Default for PlayersInfo {
    fn default() -> Self {
        Self {
            team_a: TeamInfo::SinglePlayer(PlayerInfo {
                number: 0,
                gamepad: 0,
                dual_stick: true,
                slot: PlayerSlot::A1,
            }),
            team_b: TeamInfo::SinglePlayer(PlayerInfo {
                number: 0,
                gamepad: 0,
                dual_stick: true,
                slot: PlayerSlot::B1,
            }),
        }
    }
}

/// Represents all the info related to a player in the game.
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
    SinglePlayer(PlayerInfo),
    TwoPlayer(PlayerInfo, PlayerInfo),
}
impl Default for TeamInfo {
    fn default() -> Self {
        TeamInfo::TwoPlayer(default(), default())
    }
}
impl TeamInfo {
    pub fn is_dual_stick(&self) -> bool {
        matches!(self, Self::SinglePlayer(..))
    }
    pub fn primary(&self) -> PlayerInfo {
        match self.clone() {
            TeamInfo::SinglePlayer(player_sign) | TeamInfo::TwoPlayer(player_sign, _) => {
                player_sign
            }
        }
    }
    pub fn secondary(&self) -> PlayerInfo {
        match self.clone() {
            TeamInfo::SinglePlayer(player_info) => PlayerInfo {
                slot: player_info.slot.partner(),
                ..player_info
            },
            TeamInfo::TwoPlayer(_, player_sign) => player_sign,
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
            PlayMode::Online {
                socket,
                service_type,
            } => {
                session.runner = lan_session_runner(socket, service_type);
            }
        };
        session.insert_resource(self.mode);
        session.insert_resource(Mapping);
        session.init_resource::<PlayTeamInputs>();
        session.install_plugin(Path2dTogglePlugin);

        session.add_system_to_stage(Update, toggle_debug_lines);
        session.add_system_to_stage(First, fix_camera_size);

        session.add_startup_system(spawn::scene);
        session.add_startup_system(hide_debug_lines);
        session.add_system_to_stage(Last, |mut inputs: ResMut<PlayTeamInputs>| {
            inputs.advance_frame()
        });
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

pub fn lan_session_runner(
    socket: &bones_framework::networking::NetworkMatchSocket,
    service_type: &ServiceType,
) -> Box<dyn SessionRunner> {
    use bones_framework::networking::{GgrsSessionRunner, GgrsSessionRunnerInfo};

    let mut runner = GgrsSessionRunner::<PlayTeamNetworkInputConfig>::new(
        Some(30.0),
        GgrsSessionRunnerInfo::new(socket.ggrs_socket(), Some(7), Some(2), 0),
    );
    match service_type {
        ServiceType::OnePlayer(p1) => {
            runner
                .input_collector
                .set_source(TeamSource::OnePlayer(*p1));
        }
        ServiceType::TwoPlayer(p1, p2) => {
            runner
                .input_collector
                .set_source(TeamSource::TwoPlayer(*p1, *p2));
        }
    }
    Box::new(runner)
}
pub fn offline_session_runner(players_info: PlayersInfo) -> Box<dyn SessionRunner> {
    let PlayersInfo { team_a, team_b } = players_info;
    Box::new(OfflineRunner {
        collectors: [
            PlayTeamInputCollector::new(if team_a.primary().dual_stick {
                TeamSource::OnePlayer(team_a.primary().gamepad)
            } else {
                TeamSource::TwoPlayer(team_a.primary().gamepad, team_a.secondary().gamepad)
            }),
            PlayTeamInputCollector::new(if team_b.primary().dual_stick {
                TeamSource::OnePlayer(team_b.primary().gamepad)
            } else {
                TeamSource::TwoPlayer(team_b.primary().gamepad, team_b.secondary().gamepad)
            }),
        ],
        ..Default::default()
    })
}
