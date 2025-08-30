use super::*;

pub mod prelude {
    pub use super::*;
}

/// Component for tracking which inputs are related to an entity.
/// This needs to be used for online and offline input collection.
/// In online, the index is directly related to a network player
/// "slot", however in offline we need a way to connect this index
/// to something other than a slot.
#[derive(HasSchema, Default, Clone)]
pub struct Client {
    pub index: usize,
}
/// The uncompressed minimal play session inputs.
#[derive(HasSchema, Clone, Copy, Default, Debug, PartialEq)]
#[repr(C)]
pub struct PlayInput {
    pub x: f32,
    pub y: f32,
    pub shoot: PressInput,
    pub pass: PressInput,
    pub start: PressInput,
}
impl PlayInput {
    pub fn from_local(local: &LocalInput) -> Self {
        Self {
            x: local.left_stick.x,
            y: local.left_stick.y,
            shoot: local.south | local.right_trigger | local.left_trigger | local.east,
            pass: local.west | local.left_bump | local.north,
            start: local.start,
        }
    }
    /// Gather input for the secondary player in case the player is
    /// being controlled by one controller.
    pub fn from_local_dual(local: &LocalInput) -> Self {
        Self {
            x: local.right_stick.x,
            y: local.right_stick.y,
            shoot: local.south | local.right_trigger | local.left_trigger,
            pass: local.west | local.right_bump,
            start: local.start,
        }
    }
}

/// The second layer of compiled input collection.
#[derive(HasSchema, Clone, Default, Deref, DerefMut)]
pub struct PlayInputs {
    pub clients: [PlayInput; 4usize],
}
impl PlayInputs {
    pub fn from_world(world: &World) -> Self {
        let mut clients = [default(); 4];
        let mut local_inputs = world.resource_mut::<LocalInputs>();

        match &*world.resource::<crate::play::PlayMode>() {
            crate::PlayMode::Online { .. } => todo!(),
            crate::PlayMode::Offline(PlayersInfo { team_a, team_b }) => {
                match team_a {
                    TeamInfo::Single(player_sign) => {
                        clients[0] =
                            PlayInput::from_local(local_inputs.get_input(player_sign.gamepad));
                        clients[1] =
                            PlayInput::from_local_dual(local_inputs.get_input(player_sign.gamepad));
                    }
                    TeamInfo::Double(primary, secondary) => {
                        clients[0] = PlayInput::from_local(local_inputs.get_input(primary.gamepad));
                        clients[1] =
                            PlayInput::from_local(local_inputs.get_input(secondary.gamepad));
                    }
                }
                match team_b {
                    TeamInfo::Single(player_sign) => {
                        clients[2] =
                            PlayInput::from_local(local_inputs.get_input(player_sign.gamepad));
                        clients[3] =
                            PlayInput::from_local_dual(local_inputs.get_input(player_sign.gamepad));
                    }
                    TeamInfo::Double(primary, secondary) => {
                        clients[2] = PlayInput::from_local(local_inputs.get_input(primary.gamepad));
                        clients[3] =
                            PlayInput::from_local(local_inputs.get_input(secondary.gamepad));
                    }
                }
            }
        }
        Self { clients }
    }
    pub fn get_control(&self, index: usize) -> &PlayInput {
        &self.clients[index]
    }
}

// #[derive(HasSchema, Clone, Default)]
// pub struct Mapping;
// #[derive(HasSchema, Clone, Default)]
// pub struct Source;

// pub struct SinglesNetworkInputConfig;
// impl<'a> NetworkInputConfig<'a> for SinglesNetworkInputConfig {
//     type Dense = DenseSingle;
//     type Control = Single;
//     type PlayerControls = Inputs;
//     type InputCollector = SingleCollector;
// }

// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, bytemuck::Pod, bytemuck::Zeroable)]
// #[repr(C)]
// pub struct DenseSingle {
//     pub angle: i8,
//     pub bools: u8,
// }
// impl NetworkPlayerControl<DenseSingle> for Single {
//     fn get_dense_input(&self) -> DenseSingle {
//         DenseSingle {
//             angle: Vec2::new(self.x, self.y).angle_between(Vec2::X) as i8,
//             bools: *0u8
//                 .set_bit(0, self.shoot.pressed)
//                 .set_bit(1, self.pass.pressed)
//                 .set_bit(2, self.start.pressed)
//                 .set_bit(3, Vec2::new(self.x, self.y).length() > 0.1),
//         }
//     }

//     fn update_from_dense(&mut self, dense: &DenseSingle) {
//         self.shoot.update(dense.bools.get_bit(0));
//         self.pass.update(dense.bools.get_bit(1));
//         self.start.update(dense.bools.get_bit(2));

//         let Vec2 { x, y } = dense
//             .bools
//             .get_bit(3)
//             .then_some(Vec2::from_angle(dense.angle as f32))
//             .unwrap_or_default();
//         self.x = x;
//         self.y = -y;
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     #[test]
//     pub fn dense_conversions() {
//         let mut b = Single::default();
//         b.update_from_dense(&Single::default().get_dense_input());

//         assert_eq!(Single::default(), b);
//     }
// }

// impl PlayerControls<'_, Single> for Inputs {
//     type InputCollector = SingleCollector;
//     type ControlMapping = Mapping;
//     type ControlSource = Source;

//     fn update_controls(&mut self, collector: &mut Self::InputCollector) {
//         // for (i, single) in self.iter_mut().enumerate() {
//         //     *single = collector.get_control(i, Source).clone();
//         // }
//     }

//     /// Get the [`ControlSource`](PlayerControls::ControlSource) for a player.
//     /// This is only [`Some`] on the local player indices.
//     fn get_control_source(&self, local_player_idx: usize) -> Option<Self::ControlSource> {
//         Some(Source)
//     }
//     /// Get a reference of the controls that will be synced across the network.
//     fn get_control(&self, player_idx: usize) -> &Single {
//         &self[player_idx]
//     }
//     /// Get a mutable reference of the controls that will be synced across the network.
//     /// Used for network updates.
//     fn get_control_mut(&mut self, player_idx: usize) -> &mut Single {
//         &mut self[player_idx]
//     }
// }
