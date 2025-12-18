use super::*;

bitfield::bitfield! {
    #[derive(bytemuck::Pod, bytemuck::Zeroable, Default, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct PlayInputDense(u64);
    impl Debug;
    pub shoot, set_shoot: 0;
    pub pass, set_pass: 1;
    pub some_angle, set_some_angle: 2;
    pub from into DenseAngle, angle, set_angle: 32, 3;
}
// TODO: replace with trait after bones exposes the traits for wasm32
impl PlayInput {
    pub fn get_dense_input(&self) -> PlayInputDense {
        let vec2 = Vec2::new(self.x, self.y);
        let angle = (vec2.length() > 0.1).then_some(vec2.angle_between(Vec2::X));

        let mut dense = PlayInputDense::default();
        dense.set_shoot(self.shoot.pressed());
        dense.set_pass(self.pass.pressed());
        dense.set_some_angle(angle.is_some());

        if let Some(angle) = angle {
            dense.set_angle(DenseAngle(angle));
        }
        dense
    }
    pub fn update_from_dense(&mut self, dense: &PlayInputDense) {
        let Vec2 { x, y } = Vec2::from_angle(*dense.angle());
        self.x = x;
        self.y = y;
        self.shoot.apply_bool(dense.shoot());
        self.pass.apply_bool(dense.pass());
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl bones_framework::networking::input::NetworkPlayerControl<PlayInputDense> for PlayInput {
    fn get_dense_input(&self) -> PlayInputDense {
        self.get_dense_input()
    }
    fn update_from_dense(&mut self, dense: &PlayInputDense) {
        self.update_from_dense(dense);
    }
}
impl From<u64> for PlayInputDense {
    fn from(value: u64) -> Self {
        Self(value)
    }
}
impl From<PlayInputDense> for u64 {
    fn from(dense: PlayInputDense) -> Self {
        dense.0
    }
}

bitfield::bitfield! {
    #[derive(bytemuck::Pod, bytemuck::Zeroable, Default, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct PlayTeamInputDense(u64);
    impl Debug;
    pub from into PlayInputDense, p1, set_p1: 31, 0;
    pub from into PlayInputDense, p2, set_p2: 63, 32;
}
// TODO: replace with trait after bones exposes the traits for wasm32
impl PlayTeamInput {
    pub fn get_dense_input(&self) -> PlayTeamInputDense {
        let mut dense = PlayTeamInputDense::default();
        dense.set_p1(self.p1.get_dense_input());
        dense.set_p2(self.p2.get_dense_input());
        dense
    }
    pub fn update_from_dense(&mut self, new_control: &PlayTeamInputDense) {
        self.p1.update_from_dense(&new_control.p1());
        self.p2.update_from_dense(&new_control.p2());
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl bones_framework::networking::input::NetworkPlayerControl<PlayTeamInputDense>
    for PlayTeamInput
{
    fn get_dense_input(&self) -> PlayTeamInputDense {
        self.get_dense_input()
    }
    fn update_from_dense(&mut self, new_control: &PlayTeamInputDense) {
        self.update_from_dense(new_control);
    }
}

#[derive(Default, Deref, DerefMut, Debug)]
pub struct DenseAngle(pub f32);

impl From<u16> for DenseAngle {
    fn from(bits: u16) -> Self {
        DenseAngle(
            move_direction::DenseMoveDirection::from(bits)
                .0
                .angle_between(Vec2::X),
        )
    }
}
impl From<DenseAngle> for u16 {
    fn from(angle: DenseAngle) -> Self {
        u16::from(move_direction::DenseMoveDirection(Vec2::from_angle(
            angle.0,
        )))
    }
}
impl From<u32> for DenseAngle {
    fn from(bits: u32) -> Self {
        let bits_16 = bits as u16;
        bits_16.into()
    }
}
impl From<DenseAngle> for u32 {
    fn from(dir: DenseAngle) -> Self {
        let bits_16 = u16::from(dir);
        bits_16 as u32
    }
}
impl From<u64> for DenseAngle {
    fn from(bits: u64) -> Self {
        let bits_32 = bits as u32;
        bits_32.into()
    }
}
impl From<DenseAngle> for u64 {
    fn from(dir: DenseAngle) -> Self {
        let bits_16 = u32::from(dir);
        bits_16 as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // TODO: This is an insufficient test since default is much more likely to convert properly.
    #[test]
    pub fn dense_conversions() {
        let mut b = PlayInput::default();
        b.update_from_dense(&PlayInput::default().get_dense_input());

        assert_eq!(PlayInput::default(), b);
    }
}

mod move_direction {
    //! Copied from `bones_framework::networking::proto`.

    // use bevy::reflect::Reflect;
    use numquant::{IntRange, Quantized};

    use crate::prelude::*;

    /// A newtype around [`Vec2`] that implements [`From<u16>`] and [`Into<u16>`] as a way to compress
    /// user stick input for use in [`self::input::DenseInput`].
    #[derive(Debug, Deref, DerefMut, Default)]
    pub struct DenseMoveDirection(pub Vec2);

    /// This is the specific [`Quantized`] type that we use to represent movement directions in
    /// [`DenseMoveDirection`]. This encodes magnitude of direction, but sign is encoded separately.
    type MoveDirQuant = Quantized<IntRange<u16, 0b11111, 0, 1>>;

    impl From<u16> for DenseMoveDirection {
        fn from(bits: u16) -> Self {
            // maximum movement value representable, we use 6 bits to represent each movement direction.
            // Most significant is sign, and other 5 encode float value between 0 and
            let bit_length = 6;
            let quantized = 0b011111;
            let sign = 0b100000;
            // The first six bits represent the x movement
            let x_move_bits = bits & quantized;
            let x_move_sign = if bits & sign == 0 { 1.0 } else { -1.0 };
            // The second six bits represents the y movement
            let y_move_bits = (bits >> bit_length) & quantized;
            let y_move_sign = if (bits >> bit_length) & sign == 0 {
                1.0
            } else {
                -1.0
            };

            // Round near-zero values to zero
            let mut x = MoveDirQuant::from_raw(x_move_bits).to_f32();
            x *= x_move_sign;
            if x.abs() < 0.02 {
                x = 0.0;
            }
            let mut y = MoveDirQuant::from_raw(y_move_bits).to_f32();
            y *= y_move_sign;
            if y.abs() < 0.02 {
                y = 0.0;
            }

            DenseMoveDirection(Vec2::new(x, y))
        }
    }

    impl From<DenseMoveDirection> for u16 {
        fn from(dir: DenseMoveDirection) -> Self {
            let x_bits = MoveDirQuant::from_f32(dir.x.abs()).raw();
            let y_bits = MoveDirQuant::from_f32(dir.y.abs()).raw();
            let x_sign_bit = if dir.x.is_sign_positive() {
                0
            } else {
                0b100000
            };
            let y_sign_bit = if dir.y.is_sign_positive() {
                0
            } else {
                0b100000
            };

            (x_bits | x_sign_bit) | ((y_bits | y_sign_bit) << 6)
        }
    }

    impl From<u32> for DenseMoveDirection {
        fn from(bits: u32) -> Self {
            let bits_16 = bits as u16;
            bits_16.into()
        }
    }

    impl From<DenseMoveDirection> for u32 {
        fn from(dir: DenseMoveDirection) -> Self {
            let bits_16 = u16::from(dir);
            bits_16 as u32
        }
    }
}
