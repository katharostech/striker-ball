#![allow(clippy::too_many_arguments)]
use bones_bevy_renderer::BonesBevyRenderer;
use bones_framework::networking::input::*;
use bones_framework::networking::*;
use striker_ball::*;

mod matchmaking;
use matchmaking::MatchmakingMenu;

const fn namespace() -> (&'static str, &'static str, &'static str) {
    ("striker_ball", "example", "lan")
}

fn main() {
    setup_logs!(namespace());

    crate::register_schemas();

    let mut game = Game::new();

    game.install_plugin(DefaultGamePlugin);
    game.install_plugin(LocalInputGamePlugin);
    game.init_shared_resource::<AssetServer>();
    game.sessions.create_with("ui", LanExamplePlugin);

    BonesBevyRenderer::new(game)
        .namespace(namespace())
        .app()
        .run();
}

pub struct LanExamplePlugin;
impl SessionPlugin for LanExamplePlugin {
    fn install(self, session: &mut SessionBuilder) {
        session
            .install_plugin(DefaultSessionPlugin)
            .install_plugin(MatchmakingMenu(Visual::new_shown()))
            .install_plugin(Matchmaker::new("striker_ball").refresh(1.0).player_count(2))
            .add_system_to_stage(Update, ui_update);
    }
}

fn ui_update(world: &World) {
    let socket = world.resource::<Matchmaker>().network_match_socket();
    if let Some(socket) = socket {
        let mut sessions = world.resource_mut::<Sessions>();
        if sessions.get("lan").is_none() {
            sessions.create_with("lan", LanSyncPlugin { socket });
        }
    } else {
        world.run_system(MatchmakingMenu::show, ());
    }
}

#[derive(HasSchema, Clone, Default, Deref, DerefMut)]
pub struct Shapes([Vec2; 2]);

pub struct LanSyncPlugin {
    pub socket: NetworkMatchSocket,
}
impl SessionPlugin for LanSyncPlugin {
    fn install(self, session: &mut SessionBuilder) {
        session.insert_resource(Shapes::default());

        session.insert_resource(ExampleMapping);
        session.insert_resource(ExampleInputs {
            local_player_idx: self.socket.player_idx(),
            control_source: ExampleSource::Keyboard,
            clients: default(),
        });
        session.runner = Box::new(GgrsSessionRunner::<ExampleNetworkInputConfig>::new(
            Some(30.0),
            GgrsSessionRunnerInfo::new(self.socket.ggrs_socket(), Some(7), Some(2), 0),
        ));
        session.insert_resource(self.socket);
        session.add_system_to_stage(Update, lan_sync_update);
    }
}

fn lan_sync_update(
    input: Res<ExampleInputs>,
    ctx: Res<EguiCtx>,
    syncing_info: Res<SyncingInfo>,
    mut shapes: ResMut<Shapes>,
) {
    use egui::*;
    CentralPanel::default().show(&ctx, |ui| {
        for (i, input) in input.clients.iter().enumerate() {
            let shape = &mut shapes[i];
            shape.x += input.right as u8 as f32 * 10.0;
            shape.x -= input.left as u8 as f32 * 10.0;
            shape.y -= input.up as u8 as f32 * 10.0;
            shape.y += input.down as u8 as f32 * 10.0;
            let color = if syncing_info.disconnected_players().contains(&i) {
                Color32::RED
            } else {
                Color32::GREEN
            };
            ui.painter().circle(
                shape.to_array().into(),
                10.,
                Color32::BLACK,
                Stroke::new(10., color),
            );
        }
    });
}

pub struct ExampleNetworkInputConfig;
impl<'a> NetworkInputConfig<'a> for ExampleNetworkInputConfig {
    // TODO: Add docs to these types to bring users through
    // what types they need to create and what they need to
    // implement to get started.
    type Dense = ExampleInputDense;
    type Control = ExampleInput;
    type PlayerControls = ExampleInputs;
    type InputCollector = ExampleInputCollector;
}

#[derive(HasSchema, Clone, Default)]
pub struct ExampleMapping;

#[derive(Debug, Clone, Copy, Default, HasSchema, Hash, Eq, PartialEq)]
#[repr(C, u8)]
pub enum ExampleSource {
    #[default]
    Keyboard,
    Gamepad(u32),
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct ExampleInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}
impl NetworkPlayerControl<ExampleInputDense> for ExampleInput {
    fn get_dense_input(&self) -> ExampleInputDense {
        let mut dense = ExampleInputDense::default();
        dense.set_up(self.up);
        dense.set_down(self.down);
        dense.set_left(self.left);
        dense.set_right(self.right);
        dense
    }
    fn update_from_dense(&mut self, dense: &ExampleInputDense) {
        *self = Self {
            up: dense.up(),
            down: dense.down(),
            left: dense.left(),
            right: dense.right(),
        };
    }
}
#[cfg(not(target_arch = "wasm32"))]
bitfield::bitfield! {
    #[derive(bytemuck::Pod, bytemuck::Zeroable, Default, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct ExampleInputDense(u8);
    impl Debug;
    pub up, set_up: 0;
    pub down, set_down: 1;
    pub left, set_left: 2;
    pub right, set_right: 3;
}

#[derive(HasSchema, Clone, Default, Debug, PartialEq)]
pub struct ExampleInputs {
    pub clients: [ExampleInput; 2],
    pub local_player_idx: u32,
    pub control_source: ExampleSource,
}
impl PlayerControls<'_, ExampleInput> for ExampleInputs {
    type InputCollector = ExampleInputCollector;
    type ControlMapping = ExampleMapping;
    type ControlSource = ExampleSource;

    fn update_controls(&mut self, collector: &mut Self::InputCollector) {
        panic!("incorrect assumption") // This is currently an unused function I believe, so no need to do.
    }

    fn get_control_source(&self, local_player_idx: usize) -> Option<Self::ControlSource> {
        (self.local_player_idx == local_player_idx as u32).then_some(self.control_source)
    }

    fn get_control(&self, player_idx: usize) -> &ExampleInput {
        &self.clients[player_idx]
    }

    fn get_control_mut(&mut self, player_idx: usize) -> &mut ExampleInput {
        &mut self.clients[player_idx]
    }
}

#[derive(Debug, PartialEq)]
pub struct ExampleInputCollector {
    current: HashMap<ExampleSource, ExampleInput>,
    last: HashMap<ExampleSource, ExampleInput>,
    just: HashMap<ExampleSource, ExampleInput>,
}
impl Default for ExampleInputCollector {
    fn default() -> Self {
        let mut map = HashMap::default();
        map.insert(ExampleSource::Keyboard, default());
        Self {
            current: map.clone(),
            last: map.clone(),
            just: map.clone(),
        }
    }
}
impl InputCollector<'_, ExampleMapping, ExampleSource, ExampleInput> for ExampleInputCollector {
    // SHOULDBE: Called on cpu cycle as opposed to the frame update.
    fn apply_inputs(
        &mut self,
        mapping: &ExampleMapping,
        keyboard: &KeyboardInputs,
        gamepad: &GamepadInputs,
    ) {
        /// The distance the stick has to move to press its equivalent 'button'.
        const STROKE: f32 = 0.5;

        for event in &gamepad.gamepad_events {
            let Some(input) = self
                .current
                .get_mut(&ExampleSource::Gamepad(*event.gamepad_id()))
            else {
                continue;
            };
            #[allow(clippy::single_match)] // TODO: remove and add more inputs for example
            match event {
                GamepadEvent::Axis(GamepadAxisEvent { axis, value, .. }) => match axis {
                    GamepadAxis::LeftStickX => {
                        input.right = *value > STROKE;
                        input.left = *value < -STROKE;
                    }
                    GamepadAxis::LeftStickY => {
                        input.up = *value > STROKE;
                        input.down = *value < -STROKE;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        for event in &keyboard.key_events {
            let Some(input) = self.current.get_mut(&ExampleSource::Keyboard) else {
                continue;
            };
            let Maybe::Set(key) = event.key_code else {
                continue;
            };
            match key {
                KeyCode::W => input.up = event.button_state.pressed(),
                KeyCode::S => input.down = event.button_state.pressed(),
                KeyCode::A => input.left = event.button_state.pressed(),
                KeyCode::D => input.right = event.button_state.pressed(),
                _ => {}
            }
        }
    }
    // SHOULDBE: Called on frame update as opposed to the cpu cycle.
    fn update_just_pressed(&mut self) {
        for (source, just) in &mut self.just {
            let current = self.current.entry(*source).or_default();
            let last = self.last.entry(*source).or_default();
            just.up = current.up && !last.up;
            just.down = current.down && !last.down;
            just.left = current.left && !last.left;
            just.right = current.right && !last.right;
        }
    }

    fn advance_frame(&mut self) {
        self.last = self.current.clone();
    }

    fn get_control(&self, player_idx: usize, control_source: ExampleSource) -> &ExampleInput {
        self.current.get(&control_source).unwrap()
    }
}
