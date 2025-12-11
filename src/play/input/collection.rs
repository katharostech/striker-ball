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

fn apply_keyboard_event_primary(input: &mut PlayInput, event: &KeyboardEvent) {
    if let KeyboardEvent {
        key_code: Maybe::Set(key),
        button_state,
        ..
    } = event
    {
        match key {
            KeyCode::W => {
                if let ButtonState::Pressed = button_state {
                    input.y = 1.0;
                } else {
                    input.y = 0.0;
                }
            }
            KeyCode::S => {
                if let ButtonState::Pressed = button_state {
                    input.y = -1.0;
                } else {
                    input.y = 0.0;
                }
            }
            KeyCode::A => {
                if let ButtonState::Pressed = button_state {
                    input.x = -1.0;
                } else {
                    input.x = 0.0;
                }
            }
            KeyCode::D => {
                if let ButtonState::Pressed = button_state {
                    input.x = 1.0;
                } else {
                    input.x = 0.0;
                }
            }
            KeyCode::J => {
                input.shoot.apply_bool(button_state.pressed());
            }
            KeyCode::K => {
                input.pass.apply_bool(button_state.pressed());
            }
            _ => {}
        }
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
    /// This is a temporary function that is needed for CPU player input.
    /// I believe this can be replaced with the [`InputCollector`] trait method
    /// once it supports the borrowing of the world and that is considered safe.
    pub fn offline_apply_inputs(&mut self, world: &World) {
        if let SingleSource::CPU(player_slot) = self.p1_source {
            cpu_player::apply_cpu_input(world, player_slot, &mut self.current.p1);
        }
        if let SingleSource::CPU(player_slot) = self.p2_source {
            cpu_player::apply_cpu_input(world, player_slot, &mut self.current.p2);
        }
        self.apply_inputs(
            &world.resource(),
            &world.resource(),
            &world.resource(),
            &world.resource(),
        );
    }
}
impl InputCollector<'_, Mapping, BlankSource, PlayTeamInput> for PlayTeamInputCollector {
    // Called on cpu cycle as opposed to the frame update.
    fn apply_inputs(
        &mut self,
        _mapping: &Mapping,
        mouse: &MouseInputs,
        keyboard: &KeyboardInputs,
        gamepad: &GamepadInputs,
    ) {
        match self.p1_source {
            SingleSource::KeyboardMouse => {
                for event in &keyboard.key_events {
                    apply_keyboard_event_primary(&mut self.current.p1, event)
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
            SingleSource::CPU(..) => {} // processed in offline runner only via `Self::offline_apply_inputs`
        }
        match self.p2_source {
            SingleSource::KeyboardMouse => {
                for event in &keyboard.key_events {
                    apply_keyboard_event_primary(&mut self.current.p2, event)
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
            SingleSource::CPU(..) => {} // processed in offline runner only via `Self::offline_apply_inputs`
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
