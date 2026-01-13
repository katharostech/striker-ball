use super::*;

#[derive(HasSchema, Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SingleSource {
    #[default]
    KeyboardMouse,
    CPU(PlayerSlot),
    Gamepad(u32),
}

// TODO: This is unused right now, remove if needed.
/// Collection type variants for gathering inputs for two characters on one computer.
#[derive(HasSchema, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TeamSource {
    /// Represents one source for two characters.
    TwinStick(u32),
    /// Represents two sources for two characters.
    TwoPlayer(SingleSource, SingleSource),
}
impl Default for TeamSource {
    fn default() -> Self {
        Self::TwinStick(0)
    }
}
fn apply_keyboard_state_primary(
    input: &mut PlayInput,
    key: &KeyCode,
    keyboard_state: &KeyboardState,
) {
    match key {
        KeyCode::W | KeyCode::S => {
            input.y = keyboard_state.is_pressed(&KeyCode::W) as i8 as f32
                - keyboard_state.is_pressed(&KeyCode::S) as i8 as f32;
        }
        KeyCode::A | KeyCode::D => {
            input.x = keyboard_state.is_pressed(&KeyCode::D) as i8 as f32
                - keyboard_state.is_pressed(&KeyCode::A) as i8 as f32;
        }
        KeyCode::J => {
            input.shoot.apply_bool(keyboard_state.is_pressed(key));
        }
        KeyCode::K => {
            input.pass.apply_bool(keyboard_state.is_pressed(key));
        }
        _ => {}
    }
}
fn apply_mouse_event_primary(input: &mut PlayInput, event: &MouseButtonEvent) {
    let MouseButtonEvent { button, state } = event;
    match button {
        MouseButton::Left => input.shoot.apply_bool(state.pressed()),
        MouseButton::Right => input.pass.apply_bool(state.pressed()),
        MouseButton::Middle | MouseButton::Other(_) => {}
    }
}
fn apply_gamepad_event_primary(input: &mut PlayInput, event: &GamepadEvent, gamepad_id: u32) {
    if *event.gamepad_id() != gamepad_id {
        return;
    }
    match event {
        GamepadEvent::Axis(GamepadAxisEvent { axis, value, .. }) => match axis {
            GamepadAxis::LeftStickX => {
                input.x = *value;
            }
            GamepadAxis::LeftStickY => {
                input.y = *value;
            }
            _ => {}
        },
        GamepadEvent::Button(GamepadButtonEvent { button, value, .. }) => match button {
            GamepadButton::South => {
                input.shoot.apply_value(*value);
            }
            GamepadButton::West => {
                input.pass.apply_value(*value);
            }
            GamepadButton::RightTrigger2 => {
                input.shoot.apply_value(*value);
            }
            GamepadButton::LeftTrigger2 => {
                input.shoot.apply_value(*value);
            }
            GamepadButton::LeftTrigger => {
                input.pass.apply_value(*value);
            }
            GamepadButton::RightTrigger => {
                input.pass.apply_value(*value);
            }
            _ => {}
        },
        _ => {}
    }
}
fn apply_gamepad_event_secondary(input: &mut PlayInput, event: &GamepadEvent, gamepad_id: u32) {
    if *event.gamepad_id() != gamepad_id {
        return;
    }
    match event {
        GamepadEvent::Axis(GamepadAxisEvent { axis, value, .. }) => match axis {
            GamepadAxis::RightStickX => {
                input.x = *value;
            }
            GamepadAxis::RightStickY => {
                input.y = *value;
            }
            _ => {}
        },
        GamepadEvent::Button(GamepadButtonEvent { button, value, .. }) => match button {
            GamepadButton::South => {
                input.shoot.apply_value(*value);
            }
            GamepadButton::West => {
                input.pass.apply_value(*value);
            }
            GamepadButton::RightTrigger2 => {
                input.shoot.apply_value(*value);
            }
            GamepadButton::LeftTrigger2 => {
                input.shoot.apply_value(*value);
            }
            GamepadButton::RightTrigger => {
                input.pass.apply_value(*value);
            }
            _ => {}
        },
        _ => {}
    }
}

#[derive(Default, Debug, PartialEq)]
// NOTE: For Ggrs, the only need for the collector is to give dense input every SessionRunner::step.
pub struct PlayTeamInputCollector {
    p1_source: SingleSource,
    p2_source: SingleSource,
    // TODO: I think current may be the wrong name but I still have to nail down all the input stuff
    // TODO: FIXME: This should only include the state, but `PlayTeamInput` has just_pressed data in it.
    // NOTE: We can only change this by disolving the unnecessary type restrictions on the collector.
    current: PlayTeamInput,
    // TODO: I think I can put individual collectors into this collector.
}
impl PlayTeamInputCollector {
    pub fn new(p1_source: SingleSource, p2_source: SingleSource) -> Self {
        Self {
            p1_source,
            p2_source,
            current: Default::default(),
        }
    }
    pub fn set_sources(&mut self, p1_source: SingleSource, p2_source: SingleSource) {
        self.p1_source = p1_source;
        self.p2_source = p2_source;
    }
}
impl InputCollector<'_, Mapping, BlankSource, PlayTeamInput> for PlayTeamInputCollector {
    // Called on cpu cycle as opposed to the frame update.
    fn apply_inputs(&mut self, world: &World) {
        let keyboard = world.resource::<KeyboardInputs>();
        let keyboard_state = world.resource::<KeyboardState>();
        let gamepad = world.resource::<GamepadInputs>();
        let mouse = world.resource::<MouseInputs>();

        match self.p1_source {
            SingleSource::KeyboardMouse => {
                for event in &keyboard.key_events {
                    let Maybe::Set(key) = &event.key_code else {
                        continue;
                    };
                    apply_keyboard_state_primary(&mut self.current.p1, key, &keyboard_state);
                }
                for event in &mouse.button_events {
                    apply_mouse_event_primary(&mut self.current.p1, event);
                }
            }
            SingleSource::Gamepad(gamepad_id) => {
                for event in &gamepad.gamepad_events {
                    apply_gamepad_event_primary(&mut self.current.p1, event, gamepad_id)
                }
            }
            SingleSource::CPU(player_slot) => {
                self.current
                    .p1
                    .update_from_dense(&cpu_player::get_cpu_input(world, player_slot));
            }
        }
        match self.p2_source {
            SingleSource::KeyboardMouse => {
                for event in &keyboard.key_events {
                    let Maybe::Set(key) = &event.key_code else {
                        continue;
                    };
                    apply_keyboard_state_primary(&mut self.current.p2, key, &keyboard_state);
                }
                for event in &mouse.button_events {
                    apply_mouse_event_primary(&mut self.current.p2, event);
                }
            }
            SingleSource::Gamepad(gamepad_id) => {
                if self.p1_source == self.p2_source {
                    for event in &gamepad.gamepad_events {
                        apply_gamepad_event_secondary(&mut self.current.p2, event, gamepad_id)
                    }
                } else {
                    for event in &gamepad.gamepad_events {
                        apply_gamepad_event_primary(&mut self.current.p2, event, gamepad_id)
                    }
                }
            }
            SingleSource::CPU(player_slot) => {
                self.current
                    .p2
                    .update_from_dense(&cpu_player::get_cpu_input(world, player_slot));
            }
        }
    }
    fn update_just_pressed(&mut self) {
        // Not neccessary as this is called at the same time as `apply_inputs`.
    }
    // Called on frame update as opposed to the cpu cycle.
    // By the time this is called, the dense input is sent across the network.
    fn advance_frame(&mut self) {
        // self.current.p1.pass.advance();
        // self.current.p1.shoot.advance();
        // self.current.p2.pass.advance();
        // self.current.p2.shoot.advance();
        // This can be properly handled on in-game systems.
    }
    // This is only ever used for getting *dense* input, so we should probably return
    // the dense instead. This would reduce some processing as well since we're not
    // necessarily using all of the non-dense input to output dense input.
    //
    // I'm skipping use of the control_source to opt for chosen source directly on
    // the collector.
    fn get_control(&self, _player_idx: usize, _control_source: BlankSource) -> &PlayTeamInput {
        &self.current
    }
}
