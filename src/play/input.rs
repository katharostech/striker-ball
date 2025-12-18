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
        }
    }
}

#[derive(HasSchema, Clone, Default)]
pub struct Mapping;

/// Just a type filler because I'm not using this for input sourcing.
#[derive(HasSchema, Clone, Default)]
pub struct BlankSource;

//
// Network Input Configuration
//

#[cfg(not(target_arch = "wasm32"))]
use bones_framework::networking::input::*;

#[cfg(not(target_arch = "wasm32"))]
impl PlayerControls<'_, PlayTeamInput> for PlayTeamInputs {
    type InputCollector = PlayTeamInputCollector;
    type ControlMapping = Mapping;
    type ControlSource = BlankSource;

    fn update_controls(&mut self, _collector: &mut Self::InputCollector) {
        panic!("incorrect assumption") // This is currently an unused function I believe, so no need to do.
    }

    fn get_control_source(&self, local_player_idx: usize) -> Option<Self::ControlSource> {
        Some(BlankSource)
    }

    fn get_control(&self, player_idx: usize) -> &PlayTeamInput {
        &self.clients[player_idx]
    }

    fn get_control_mut(&mut self, player_idx: usize) -> &mut PlayTeamInput {
        &mut self.clients[player_idx]
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct PlayTeamNetworkInputConfig;

#[cfg(not(target_arch = "wasm32"))]
impl<'a> NetworkInputConfig<'a> for PlayTeamNetworkInputConfig {
    type Dense = PlayTeamInputDense;
    type Control = PlayTeamInput;
    type PlayerControls = PlayTeamInputs;
    type InputCollector = PlayTeamInputCollector;
}
