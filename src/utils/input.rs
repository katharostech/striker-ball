use bones_framework::prelude::*;

/// Stores input data for inputs with an on & off state. Tracks
/// 'on', 'off', 'just pressed', 'just released', and 'held'.
#[derive(HasSchema, Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct PressInput {
    current: bool,
    last: bool,
    /// Tracks how many frames the input has been "held".
    held: u32,
}
impl PressInput {
    pub fn just_pressed(&self) -> bool {
        self.current && !self.last
    }
    pub fn just_released(&self) -> bool {
        !self.current && self.last
    }
    pub fn pressed(&self) -> bool {
        self.current
    }
    pub fn released(&self) -> bool {
        !self.current
    }
    pub fn just_held(&self, frames: u32) -> bool {
        self.held >= frames && self.pressed()
    }
    pub fn held(&self) -> u32 {
        self.held
    }
    pub fn press(&mut self) {
        self.apply_bool(true);
    }
    pub fn release(&mut self) {
        self.apply_bool(false);
    }
    pub fn toggle(&mut self) {
        self.apply_bool(!self.current);
    }
    /// Applies a boolean value to the input for whether or not the button should be pressed.
    /// This can be used multiple times in one frame.
    pub fn apply_bool(&mut self, pressed: bool) {
        self.current = pressed;
        if self.just_pressed() {
            self.held = 0;
        }
    }
    /// Applies an `f32` value to the input, `value == 1.0` meaning the button should be pressed.
    /// /// This can be used multiple times in one frame.
    pub fn apply_value(&mut self, value: f32) {
        self.apply_bool(value == 1.0);
    }
    /// This should be called after the input has been read or updated in a frame.
    pub fn advance(&mut self) {
        self.last = self.current;

        if self.pressed() {
            self.held += 1;
        }
    }
}
impl std::ops::BitOr for PressInput {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            current: self.current.max(rhs.current),
            last: self.last.max(rhs.last),
            ..self
        }
    }
}
