use super::*;

pub mod collection;
pub use collection::*;

pub mod dense;
pub use dense::*;

pub mod prelude {
    pub use super::*;
}

#[derive(HasSchema, Clone, Copy, Default, Debug, PartialEq)]
#[repr(C)]
pub struct PlayInput {
    pub x: f32,
    pub y: f32,
    pub shoot: PressInput,
    pub tackle: PressInput,
    pub pass: PressInput,
}

#[derive(HasSchema, Clone, Copy, Default, Debug, PartialEq)]
#[repr(C)]
pub struct PlayTeamInput {
    pub p1: PlayInput,
    pub p2: PlayInput,
}

#[derive(HasSchema, Clone, Default, Debug, Deref, DerefMut)]
pub struct PlayTeamInputs {
    pub clients: [PlayTeamInput; 2usize],
}
impl PlayTeamInputs {
    /// Uses the player slot to get the exact control for one character.
    pub fn get_character_control(&self, slot: PlayerSlot) -> &PlayInput {
        match slot {
            PlayerSlot::A1 => &self.clients[0].p1,
            PlayerSlot::A2 => &self.clients[0].p2,
            PlayerSlot::B1 => &self.clients[1].p1,
            PlayerSlot::B2 => &self.clients[1].p2,
        }
    }
    // This is how this will be handled in the future; I call this in my own
    // systems as a way to handle just_pressed inputs based on the state
    // that `Self` has gathered from all the collectors.
    pub fn advance_frame(&mut self) {
        for client in &mut self.clients {
            client.p1.shoot.advance();
            client.p2.shoot.advance();
            client.p1.pass.advance();
            client.p2.pass.advance();
            client.p1.tackle.advance();
            client.p2.tackle.advance();
        }
    }
}

impl Controls<'_, PlayTeamInput> for PlayTeamInputs {
    fn get_control(&self, player_idx: usize) -> &PlayTeamInput {
        &self.clients[player_idx]
    }

    fn get_control_mut(&mut self, player_idx: usize) -> &mut PlayTeamInput {
        &mut self.clients[player_idx]
    }
}

pub struct PlayTeamDenseInputConfig;

impl<'a> DenseInputConfig<'a> for PlayTeamDenseInputConfig {
    type Dense = PlayTeamInputDense;
    type Control = PlayTeamInput;
    type Controls = PlayTeamInputs;
    type InputCollector = PlayTeamInputCollector;
}
