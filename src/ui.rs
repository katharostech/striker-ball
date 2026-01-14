use super::*;

pub mod plugin;
pub use plugin::*;

pub mod countdown;
pub mod credits;
pub mod fade;
pub mod howtoplay;
pub mod lan_select;
#[cfg(not(target_arch = "wasm32"))]
pub mod lan_ui;
pub mod match_done;
pub mod network_quit;
pub mod pause;
pub mod score_display;
pub mod settings;
pub mod splash;
pub mod team_select;
pub mod winner;

pub use countdown::*;
pub use credits::*;
pub use fade::*;
pub use howtoplay::*;
pub use lan_select::*;
#[cfg(not(target_arch = "wasm32"))]
pub use lan_ui::*;
pub use match_done::*;
pub use network_quit::*;
pub use pause::*;
pub use score_display::*;
pub use settings::*;
pub use splash::*;
pub use team_select::*;
pub use winner::*;
