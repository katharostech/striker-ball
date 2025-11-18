use core::iter::Iterator;

use super::*;
use crate::play::*;
use crate::player::*;

#[derive(HasSchema, Clone, Copy, Default, PartialEq, Eq)]
pub enum PartnerSetting {
    #[default]
    CPU,
    TwinStick,
}
impl PartnerSetting {
    pub fn cycle(&mut self) {
        match self {
            Self::CPU => *self = Self::TwinStick,
            Self::TwinStick => *self = Self::CPU,
        }
    }
}

#[derive(HasSchema, Clone, Default)]
pub enum Join {
    #[default]
    Empty,
    Joined {
        gamepad: u32,
    },
    Hover {
        gamepad: u32,
        slot: PlayerSlot,
    },
    Single {
        gamepad: u32,
        slot: PlayerSlot,
        partner_setting: PartnerSetting,
    },
    Double {
        gamepad: u32,
        slot: PlayerSlot,
        partner_setting: PartnerSetting,
    },
}
impl Join {
    pub fn join(&mut self, gamepad_id: u32) {
        if let Self::Empty = *self {
            *self = Self::Joined {
                gamepad: gamepad_id,
            }
        } else {
            panic!("un-enforced join state ordering");
        }
    }
    pub fn unjoin(&mut self) {
        if let Self::Joined { .. } = *self {
            *self = Self::Empty;
        } else {
            panic!("un-enforced join state ordering");
        }
    }
    pub fn hover(&mut self, slot: PlayerSlot) {
        if let Self::Joined { gamepad } = *self {
            *self = Self::Hover { gamepad, slot }
        } else {
            panic!("un-enforced join state ordering");
        }
    }
    pub fn unhover(&mut self) {
        if let Self::Hover { gamepad, .. } = *self {
            *self = Self::Joined { gamepad }
        } else {
            panic!("un-enforced join state ordering");
        }
    }
    pub fn single(&mut self) {
        if let Self::Hover { gamepad, slot } = *self {
            *self = Self::Single {
                gamepad,
                slot,
                partner_setting: PartnerSetting::default(),
            }
        } else {
            panic!("un-enforced join state ordering");
        }
    }
    pub fn unsingle(&mut self) {
        if let Self::Single { gamepad, slot, .. } = *self {
            *self = Self::Hover { gamepad, slot }
        } else {
            panic!("un-enforced join state ordering");
        }
    }
    pub fn double(&mut self) {
        if let Self::Single {
            gamepad,
            slot,
            partner_setting,
        } = *self
        {
            *self = Self::Double {
                gamepad,
                slot,
                partner_setting,
            }
        } else {
            panic!("un-enforced join state ordering");
        }
    }
    pub fn undouble(&mut self) {
        if let Self::Double {
            gamepad,
            slot,
            partner_setting,
        } = *self
        {
            *self = Self::Single {
                gamepad,
                slot,
                partner_setting,
            }
        } else {
            panic!("un-enforced join state ordering");
        }
    }
    pub fn get_player_slot(&self) -> Option<PlayerSlot> {
        match &self {
            Join::Empty | Join::Joined { .. } => None,
            Join::Hover { slot, .. } | Join::Single { slot, .. } | Join::Double { slot, .. } => {
                Some(*slot)
            }
        }
    }
    pub fn is_gamepad_id(&self, gamepad_id: u32) -> bool {
        matches!(self,
            Join::Joined { gamepad }
            | Join::Hover {gamepad, ..}
            | Join::Single { gamepad, .. }
            | Join::Double { gamepad, .. } if *gamepad == gamepad_id,
        )
    }
    pub fn is_player_id(&self, id: PlayerSlot) -> bool {
        matches!(self, Join::Hover {slot, ..} | Join::Single { slot, .. } | Join::Double { slot, .. } if *slot == id)
    }
    pub fn is_empty(&self) -> bool {
        matches!(self, Join::Empty)
    }
    pub fn is_joined(&self) -> bool {
        matches!(
            self,
            Join::Joined { .. } | Join::Hover { .. } | Join::Single { .. } | Join::Double { .. }
        )
    }
    pub fn is_hovered(&self) -> bool {
        matches!(
            self,
            Join::Hover { .. } | Join::Single { .. } | Join::Double { .. }
        )
    }
    pub fn is_single(&self) -> bool {
        matches!(self, Join::Single { .. } | Join::Double { .. })
    }
    pub fn is_double(&self) -> bool {
        matches!(self, Self::Double { .. })
    }
    pub fn is_dual_stick(&self) -> bool {
        matches!(
            self,
            Self::Single {
                partner_setting: PartnerSetting::TwinStick,
                ..
            } | Self::Double {
                partner_setting: PartnerSetting::TwinStick,
                ..
            }
        )
    }
}

#[derive(HasSchema, Clone, Default)]
pub struct TeamSelect {
    pub visible: bool,
    pub joins: [Join; 4],
}
impl TeamSelect {
    pub fn add_gamepad(&mut self, id: u32) {
        if !self.joins.iter().any(|join| join.is_gamepad_id(id)) {
            for pad in &mut self.joins {
                if !pad.is_joined() {
                    pad.join(id);
                    return;
                }
            }
        }
    }
    pub fn remove_gamepad(&mut self, id: u32) {
        for pad in &mut self.joins {
            if pad.is_gamepad_id(id) {
                *pad = default();
                break;
            }
        }
    }
    pub fn get_join_from_slot(&self, slot: PlayerSlot) -> Option<&Join> {
        self.joins.iter().find(|join| join.is_player_id(slot))
    }
    pub fn get_mut_join_from_slot(&mut self, slot: PlayerSlot) -> Option<&mut Join> {
        self.joins.iter_mut().find(|join| join.is_player_id(slot))
    }
    pub fn get_index_from_gamepad(&self, id: u32) -> Option<usize> {
        for (index, join) in self.joins.iter().enumerate() {
            if join.is_gamepad_id(id) {
                return Some(index);
            }
        }
        None
    }
    pub fn ready_gamepad(&mut self, id: u32) {
        let Some(index) = self.get_index_from_gamepad(id) else {
            return;
        };
        let Some(slot) = self.joins[index].get_player_slot() else {
            return;
        };
        let dual_able = !self.is_player_slot_hovered(slot.partner());

        let join = &mut self.joins[index];

        if join.is_gamepad_id(id) {
            if join.is_hovered() && !join.is_single() {
                join.single();
            } else if join.is_single() && !join.is_double() && dual_able {
                join.double();
            }
        }
    }
    pub fn reverse_gamepad(&mut self, id: u32) {
        for join in &mut self.joins {
            if join.is_gamepad_id(id) {
                if join.is_double() {
                    join.undouble();
                } else if join.is_single() {
                    join.unsingle();
                } else if join.is_hovered() {
                    join.unhover();
                } else if join.is_joined() {
                    join.unjoin();
                }
            }
        }
    }
    pub fn next_slot_a(&self) -> Option<PlayerSlot> {
        let mut a1 = false;
        let mut a2 = false;
        for join in &self.joins {
            if join.is_player_id(PlayerSlot::A1) {
                a1 = true;
                if join.is_double() {
                    a2 = true
                }
            }
            if join.is_player_id(PlayerSlot::A2) {
                a2 = true;
                if join.is_double() {
                    a1 = true
                }
            }
        }
        if !a1 {
            return PlayerSlot::A1.into();
        }
        if !a2 {
            return PlayerSlot::A2.into();
        }
        None
    }
    pub fn next_slot_b(&self) -> Option<PlayerSlot> {
        let mut b1 = false;
        let mut b2 = false;
        for join in &self.joins {
            if join.is_player_id(PlayerSlot::B1) {
                b1 = true;
                if join.is_double() {
                    b2 = true
                }
            }
            if join.is_player_id(PlayerSlot::B2) {
                b2 = true;
                if join.is_double() {
                    b1 = true
                }
            }
        }
        if !b1 {
            return PlayerSlot::B1.into();
        }
        if !b2 {
            return PlayerSlot::B2.into();
        }
        None
    }
    pub fn left_gamepad(&mut self, id: u32) {
        let next_slot_a = self.next_slot_a();
        let cycle = 'cycle: {
            for join in &mut self.joins {
                if join.is_gamepad_id(id) {
                    if let Join::Single { .. } = join {
                        break 'cycle Some(join.get_player_slot().unwrap());
                    } else if let Join::Hover { slot, .. } = join {
                        if slot.team() == Team::B {
                            join.unhover();
                        }
                    } else if let Join::Joined { .. } = join {
                        if let Some(player_id) = next_slot_a {
                            join.hover(player_id);
                        }
                    }
                }
            }
            None
        };
        if let Some(player_slot) = cycle {
            if !self.is_player_slot_hovered(player_slot.partner()) {
                let Some(Join::Single {
                    partner_setting, ..
                }) = self.get_mut_join_from_slot(player_slot)
                else {
                    unreachable!()
                };
                partner_setting.cycle();
            }
        }
    }
    pub fn right_gamepad(&mut self, id: u32) {
        let next_slot_b = self.next_slot_b();
        let cycle = 'cycle: {
            for join in &mut self.joins {
                if join.is_gamepad_id(id) {
                    if let Join::Single { .. } = join {
                        break 'cycle Some(join.get_player_slot().unwrap());
                    } else if let Join::Hover { slot, .. } = join {
                        if slot.team() == Team::A {
                            join.unhover();
                        }
                    } else if let Join::Joined { .. } = join {
                        if let Some(player_id) = next_slot_b {
                            join.hover(player_id);
                        }
                    }
                }
            }
            None
        };
        if let Some(player_slot) = cycle {
            if !self.is_player_slot_hovered(player_slot.partner()) {
                let Some(Join::Single {
                    partner_setting, ..
                }) = self.get_mut_join_from_slot(player_slot)
                else {
                    unreachable!()
                };
                partner_setting.cycle();
            }
        }
    }
    pub fn is_double(&self, id: u32) -> bool {
        for join in &self.joins {
            if join.is_gamepad_id(id) {
                return join.is_double();
            }
        }
        false
    }
    pub fn is_player_slot_empty(&self, slot: PlayerSlot) -> bool {
        !self
            .joins
            .iter()
            .any(|join| join.is_hovered() && join.is_player_id(slot))
    }
    pub fn is_player_slot_dual_stick(&self, id: PlayerSlot) -> bool {
        self.joins
            .iter()
            .any(|join| join.is_player_id(id) && join.is_dual_stick())
    }
    pub fn is_player_slot_double(&self, id: PlayerSlot) -> bool {
        self.joins.iter().any(|join| {
            join.is_player_id(id) && join.is_double()
                || join.is_player_id(id.partner()) && join.is_double()
        })
    }
    pub fn is_player_slot_set(&self, id: PlayerSlot) -> bool {
        self.joins
            .iter()
            .any(|join| join.is_player_id(id) && join.is_single())
    }
    pub fn is_player_slot_hovered(&self, id: PlayerSlot) -> bool {
        self.joins
            .iter()
            .any(|join| join.is_player_id(id) && join.is_hovered())
    }
    pub fn get_player_signs(&self) -> Option<PlayersInfo> {
        if self.joins.iter().all(Join::is_empty) {
            return None;
        }
        let mut builder: HashMap<PlayerSlot, PlayerInfo> = HashMap::default();

        for (number, join) in self.joins.iter().enumerate() {
            match join {
                Join::Single { gamepad, slot, .. } => {
                    if self.get_join_from_slot(slot.partner()).is_none() {
                        return None;
                    } else {
                        builder.insert(
                            *slot,
                            PlayerInfo::Local {
                                number,
                                gamepad: *gamepad,
                                dual_stick: false,
                            },
                        );
                    }
                }
                Join::Double {
                    gamepad,
                    slot,
                    partner_setting,
                } => match partner_setting {
                    PartnerSetting::CPU => {
                        builder.insert(
                            *slot,
                            PlayerInfo::Local {
                                number,
                                gamepad: *gamepad,
                                dual_stick: false,
                            },
                        );
                        builder.insert(slot.partner(), PlayerInfo::CPU);
                    }
                    PartnerSetting::TwinStick => {
                        builder.insert(
                            slot.partner(),
                            PlayerInfo::Local {
                                number,
                                gamepad: *gamepad,
                                dual_stick: true,
                            },
                        );
                        builder.insert(
                            *slot,
                            PlayerInfo::Local {
                                number,
                                gamepad: *gamepad,
                                dual_stick: true,
                            },
                        );
                    }
                },
                // empty joins will be filled with CPUs
                Join::Empty => {}
                // if there is unconfirmed players joined we are not ready yet
                Join::Joined { .. } | Join::Hover { .. } => return None,
            }
        }
        for slot in [
            PlayerSlot::A1,
            PlayerSlot::A2,
            PlayerSlot::B1,
            PlayerSlot::B2,
        ] {
            if !builder.contains_key(&slot) {
                builder.insert(slot, PlayerInfo::CPU);
            }
        }
        Some(PlayersInfo {
            a1: builder.remove(&PlayerSlot::A1).unwrap(),
            a2: builder.remove(&PlayerSlot::A2).unwrap(),
            b1: builder.remove(&PlayerSlot::B1).unwrap(),
            b2: builder.remove(&PlayerSlot::B2).unwrap(),
        })
    }
}
