use super::*;

#[derive(HasSchema, Clone, Default)]
#[repr(C)]
pub struct TeamSelectAssets {
    pub slots: TeamSelectSlots,

    pub a_team_background: SizedImageAsset,
    pub b_team_background: SizedImageAsset,
    pub center_controller_column: SizedImageAsset,
    pub pad_slot_bg: SizedImageAsset,

    pub player1_icon: SizedImageAsset,
    pub player2_icon: SizedImageAsset,
    pub player3_icon: SizedImageAsset,
    pub player4_icon: SizedImageAsset,

    pub controller_icon: SizedImageAsset,
    pub controller_icon_silhouette: SizedImageAsset,

    pub cpu_icon: SizedImageAsset,
    pub partner_select_arrow: SizedImageAsset,

    pub start: SizedImageAsset,
    pub start_blink: SizedImageAsset,

    pub back_btn: Handle<Atlas>,
    pub back_buffer: u32,
}
impl TeamSelectAssets {
    pub fn player_icons(&self) -> [&SizedImageAsset; 4] {
        [
            &self.player1_icon,
            &self.player2_icon,
            &self.player3_icon,
            &self.player4_icon,
        ]
    }
}

#[derive(HasSchema, Clone, Copy, Default)]
#[repr(C)]
pub struct TeamSelectSlots {
    pub pad1: Vec2,
    pub pad2: Vec2,
    pub pad3: Vec2,
    pub pad4: Vec2,

    pub a1: Vec2,
    pub a2: Vec2,
    pub b1: Vec2,
    pub b2: Vec2,
    pub partner_select_offset_left: Vec2,
    pub partner_select_offset_right: Vec2,
    pub cpu_icon_offset: Vec2,
    pub pad_bg_offset: Vec2,
    pub number_icon_offset: Vec2,
    pub ready_text_offset: Vec2,
    pub ready_btn_offset: Vec2,

    pub start_offset: Vec2,
    pub back_btn_offset: Vec2,
}
impl TeamSelectSlots {
    pub fn pad_slots(&self) -> [&Vec2; 4] {
        [&self.pad1, &self.pad2, &self.pad3, &self.pad4]
    }
    pub fn get_player_pos(&self, player_slot: PlayerSlot) -> Vec2 {
        match player_slot {
            PlayerSlot::A1 => self.a1,
            PlayerSlot::A2 => self.a2,
            PlayerSlot::B1 => self.b1,
            PlayerSlot::B2 => self.b2,
        }
    }
    pub fn player_slots(&self) -> [&Vec2; 4] {
        [&self.a1, &self.a2, &self.b1, &self.b2]
    }
}
