use super::*;
use bones_framework::networking::input::*;

bitfield::bitfield! {
    #[derive(bytemuck::Pod, bytemuck::Zeroable, Default, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct PlayInputDense(u32);
    impl Debug;
    pub shoot, set_shoot: 0;
    pub pass, set_pass: 1;
    pub some_angle, set_some_angle: 2;
    pub from into DenseAngle, angle, set_angle: 16, 3;
}

impl NetworkPlayerControl<PlayInputDense> for PlayInput {
    fn get_dense_input(&self) -> PlayInputDense {
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
    fn update_from_dense(&mut self, dense: &PlayInputDense) {
        let Vec2 { x, y } = Vec2::from_angle(*dense.angle());
        self.x = x;
        self.y = y;
        self.shoot.apply_bool(dense.shoot());
        self.pass.apply_bool(dense.pass());
    }
}

impl From<u32> for PlayInputDense {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<PlayInputDense> for u32 {
    fn from(dense: PlayInputDense) -> Self {
        dense.0
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

#[derive(bytemuck::Pod, bytemuck::Zeroable, Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct PlayTeamInputDense {
    pub p1: PlayInputDense,
    pub p2: PlayInputDense,
}
impl NetworkPlayerControl<PlayTeamInputDense> for PlayTeamInput {
    fn get_dense_input(&self) -> PlayTeamInputDense {
        PlayTeamInputDense {
            p1: self.p1.get_dense_input(),
            p2: self.p2.get_dense_input(),
        }
    }
    fn update_from_dense(&mut self, new_control: &PlayTeamInputDense) {
        self.p1.update_from_dense(&new_control.p1);
        self.p2.update_from_dense(&new_control.p2);
    }
}

#[derive(Default, Deref, DerefMut, Debug)]
pub struct DenseAngle(pub f32);

impl From<u16> for DenseAngle {
    fn from(bits: u16) -> Self {
        use bones_framework::networking::proto::DenseMoveDirection;
        DenseAngle(DenseMoveDirection::from(bits).0.angle_between(Vec2::X))
    }
}
impl From<DenseAngle> for u16 {
    fn from(angle: DenseAngle) -> Self {
        use bones_framework::networking::proto::DenseMoveDirection;
        u16::from(DenseMoveDirection(Vec2::from_angle(angle.0)))
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
