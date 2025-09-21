#![allow(clippy::too_many_arguments)]

pub use bones::*;
pub use bones_framework::prelude as bones;

pub mod menu;
pub use menu::*;

pub mod assets;
pub use assets::*;

pub mod session;
pub use session::*;

pub mod ui;
pub use ui::*;

pub mod play;
pub use play::*;

pub mod input;
pub use input::*;

pub mod utils;
pub use utils::*;

pub mod schema;
pub use schema::*;

#[cfg(not(target_arch = "wasm32"))]
pub mod matchmaking;
pub use matchmaking::*;
