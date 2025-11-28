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
    pub menu_up: PressInput,
    pub menu_down: PressInput,
    pub menu_left: PressInput,
    pub menu_right: PressInput,
    pub menu_select: PressInput,
    pub menu_back: PressInput,

    pub pause: PressInput,

    pub left_stick: Vec2,
    pub right_stick: Vec2,
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
                    self.menu_right.apply_bool(*value > STROKE);
                    self.menu_left.apply_bool(*value < -STROKE);
                }
                GamepadAxis::LeftStickY => {
                    self.left_stick.y = *value;
                    self.menu_up.apply_bool(*value > STROKE);
                    self.menu_down.apply_bool(*value < -STROKE);
                }
                GamepadAxis::RightStickX => self.right_stick.x = *value,
                GamepadAxis::RightStickY => self.right_stick.y = *value,
                GamepadAxis::LeftZ => {}
                GamepadAxis::RightZ => {}
                GamepadAxis::Other(_) => {}
            },
            GamepadEvent::Button(GamepadButtonEvent { button, value, .. }) => match button {
                GamepadButton::DPadUp => self.menu_up.apply_value(*value),
                GamepadButton::DPadDown => self.menu_down.apply_value(*value),
                GamepadButton::DPadLeft => self.menu_left.apply_value(*value),
                GamepadButton::DPadRight => self.menu_right.apply_value(*value),
                GamepadButton::Start => {
                    self.start.apply_value(*value);
                    self.pause.apply_value(*value);
                }
                GamepadButton::North => self.north.apply_value(*value),
                GamepadButton::South => {
                    self.south.apply_value(*value);
                    self.menu_select.apply_value(*value);
                }
                GamepadButton::West => {
                    self.west.apply_value(*value);
                    self.menu_back.apply_value(*value);
                }
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
    pub fn apply_keyboard_input(&mut self, event: &KeyboardEvent) {
        let KeyboardEvent {
            key_code: Maybe::Set(key),
            button_state,
            ..
        } = event
        else {
            return;
        };
        match key {
            KeyCode::W => self.menu_up.apply_bool(button_state.pressed()),
            KeyCode::S => self.menu_down.apply_bool(button_state.pressed()),
            KeyCode::A => self.menu_left.apply_bool(button_state.pressed()),
            KeyCode::D => self.menu_right.apply_bool(button_state.pressed()),
            KeyCode::Escape => {
                self.menu_back.apply_bool(button_state.pressed());
                self.pause.apply_bool(button_state.pressed());
            }
            KeyCode::Space => self.menu_select.apply_bool(button_state.pressed()),
            KeyCode::Return => self.start.apply_bool(button_state.pressed()),
            _ => {}
        }
    }
    pub fn advance(&mut self) {
        let Self {
            menu_up,
            menu_down,
            menu_left,
            menu_right,
            menu_select,
            menu_back,
            pause,
            left_stick: _,
            right_stick: _,
            north,
            south,
            west,
            east,
            start,
            left_bump,
            right_bump,
            left_trigger,
            right_trigger,
        } = self;

        menu_up.advance();
        menu_down.advance();
        menu_left.advance();
        menu_right.advance();
        north.advance();
        south.advance();
        west.advance();
        east.advance();
        start.advance();
        left_bump.advance();
        right_bump.advance();
        left_trigger.advance();
        right_trigger.advance();
        menu_select.advance();
        menu_back.advance();
        pause.advance();
    }
}

/// The primary layer of collective inputs.
#[derive(HasSchema, Clone, Default, Deref, DerefMut)]
pub struct LocalInputs {
    pub sources: HashMap<SingleSource, LocalInput>,
}
impl LocalInputs {
    pub fn get_input(&mut self, source: SingleSource) -> &LocalInput {
        if !self.sources.contains_key(&source) {
            self.sources.insert(source, default());
        }
        self.sources.get(&source).unwrap()
    }
    pub fn update(game: &mut Game) {
        let LocalInputs { sources } = &mut *game.shared_resource_mut::<LocalInputs>().unwrap();
        let gamepad_inputs = game.shared_resource::<GamepadInputs>().unwrap();
        let keyboard_inputs = game.shared_resource::<KeyboardInputs>().unwrap();

        for event in &gamepad_inputs.gamepad_events {
            let id = &SingleSource::Gamepad(*event.gamepad_id());
            let local_input = if sources.contains_key(id) {
                sources.get_mut(id).unwrap()
            } else {
                sources.insert(*id, default());
                sources.get_mut(id).unwrap()
            };
            local_input.apply_gamepad_input(event);
        }
        for event in &keyboard_inputs.key_events {
            let id = &SingleSource::KeyboardMouse;
            let local_input = if sources.contains_key(id) {
                sources.get_mut(id).unwrap()
            } else {
                sources.insert(*id, default());
                sources.get_mut(id).unwrap()
            };
            local_input.apply_keyboard_input(event);
        }
    }
    pub fn advance(game: &mut Game) {
        for (_id, local_input) in &mut game.shared_resource_mut::<LocalInputs>().unwrap().sources {
            local_input.advance()
        }
    }
}
