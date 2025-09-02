use super::*;

pub struct LocalInputGamePlugin;
impl GamePlugin for LocalInputGamePlugin {
    fn install(self, game: &mut Game) {
        game.insert_shared_resource(LocalInputs::default());
        game.systems.add_before_system(LocalInputs::update);
        game.systems.add_after_system(LocalInputs::advance);
    }
}

/// The primary layer of individual input.
#[derive(HasSchema, Clone, Default)]
pub struct LocalInput {
    pub left_stick: Vec2,
    pub right_stick: Vec2,
    pub up: PressInput,
    pub down: PressInput,
    pub left: PressInput,
    pub right: PressInput,
    pub north: PressInput,
    pub south: PressInput,
    pub west: PressInput,
    pub east: PressInput,
    pub start: PressInput,
    pub left_bump: PressInput,
    pub right_bump: PressInput,
    pub left_trigger: PressInput,
    pub right_trigger: PressInput,
}
impl LocalInput {
    pub fn apply_gamepad_input(&mut self, event: &GamepadEvent) {
        /// The distance the stick has to move to press its equivalent 'button'.
        const STROKE: f32 = 0.5;
        match event {
            GamepadEvent::Axis(GamepadAxisEvent { axis, value, .. }) => match axis {
                GamepadAxis::LeftStickX => {
                    self.left_stick.x = *value;
                    self.right.apply_bool(*value > STROKE);
                    self.left.apply_bool(*value < -STROKE);
                }
                GamepadAxis::LeftStickY => {
                    self.left_stick.y = *value;
                    self.up.apply_bool(*value > STROKE);
                    self.down.apply_bool(*value < -STROKE);
                }
                GamepadAxis::RightStickX => self.right_stick.x = *value,
                GamepadAxis::RightStickY => self.right_stick.y = *value,
                GamepadAxis::LeftZ => {}
                GamepadAxis::RightZ => {}
                GamepadAxis::Other(_) => {}
            },
            GamepadEvent::Button(GamepadButtonEvent { button, value, .. }) => match button {
                GamepadButton::DPadUp => self.up.apply_value(*value),
                GamepadButton::DPadDown => self.down.apply_value(*value),
                GamepadButton::DPadLeft => self.left.apply_value(*value),
                GamepadButton::DPadRight => self.right.apply_value(*value),
                GamepadButton::Start => self.start.apply_value(*value),
                GamepadButton::North => self.north.apply_value(*value),
                GamepadButton::South => self.south.apply_value(*value),
                GamepadButton::West => self.west.apply_value(*value),
                GamepadButton::East => self.east.apply_value(*value),
                GamepadButton::LeftTrigger => self.left_bump.apply_value(*value),
                GamepadButton::RightTrigger => self.right_bump.apply_value(*value),
                GamepadButton::LeftTrigger2 => self.left_trigger.apply_value(*value),
                GamepadButton::RightTrigger2 => self.right_trigger.apply_value(*value),
                _ => {}
            },
            _ => {}
        }
    }
    pub fn advance(&mut self) {
        self.up.advance();
        self.down.advance();
        self.left.advance();
        self.right.advance();
        self.north.advance();
        self.south.advance();
        self.west.advance();
        self.east.advance();
        self.start.advance();
        self.left_bump.advance();
        self.right_bump.advance();
        self.left_trigger.advance();
        self.right_trigger.advance();
    }
}

/// The primary layer of collective inputs.
#[derive(HasSchema, Clone, Default, Deref, DerefMut)]
pub struct LocalInputs {
    pub gamepads: SMap<u32, LocalInput>,
}
impl LocalInputs {
    pub fn get_input(&mut self, gamepad_id: u32) -> &LocalInput {
        if !self.gamepads.contains_key(&gamepad_id) {
            self.gamepads.insert(gamepad_id, default());
        }
        self.gamepads.get(&gamepad_id).unwrap()
    }
    pub fn update(game: &mut Game) {
        let LocalInputs { gamepads } = &mut *game.shared_resource_mut::<LocalInputs>().unwrap();
        let gamepad_inputs = game.shared_resource::<GamepadInputs>().unwrap();

        for event in &gamepad_inputs.gamepad_events {
            let id = event.gamepad_id();
            let local_input = if gamepads.contains_key(id) {
                gamepads.get_mut(id).unwrap()
            } else {
                gamepads.insert(*id, default());
                gamepads.get_mut(id).unwrap()
            };
            local_input.apply_gamepad_input(event);
        }
    }
    pub fn advance(game: &mut Game) {
        for (_id, local_input) in &mut game.shared_resource_mut::<LocalInputs>().unwrap().gamepads {
            local_input.advance()
        }
    }
}
