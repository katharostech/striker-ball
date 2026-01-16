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
        source: SingleSource,
    },
    Hover {
        source: SingleSource,
        slot: PlayerSlot,
    },
    Single {
        source: SingleSource,
        slot: PlayerSlot,
        partner_setting: PartnerSetting,
    },
    Double {
        source: SingleSource,
        slot: PlayerSlot,
        partner_setting: PartnerSetting,
    },
}
impl Join {
    pub fn join(&mut self, source: SingleSource) {
        if let Self::Empty = *self {
            *self = Self::Joined { source }
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
        if let Self::Joined { source } = *self {
            *self = Self::Hover { source, slot }
        } else {
            panic!("un-enforced join state ordering");
        }
    }
    pub fn unhover(&mut self) {
        if let Self::Hover { source, .. } = *self {
            *self = Self::Joined { source }
        } else {
            panic!("un-enforced join state ordering");
        }
    }
    pub fn single(&mut self) {
        if let Self::Hover { source, slot } = *self {
            *self = Self::Single {
                source,
                slot,
                partner_setting: PartnerSetting::default(),
            }
        } else {
            panic!("un-enforced join state ordering");
        }
    }
    pub fn unsingle(&mut self) {
        if let Self::Single { source, slot, .. } = *self {
            *self = Self::Hover { source, slot }
        } else {
            panic!("un-enforced join state ordering");
        }
    }
    pub fn double(&mut self) {
        if let Self::Single {
            source,
            slot,
            partner_setting,
        } = *self
        {
            *self = Self::Double {
                source,
                slot,
                partner_setting,
            }
        } else {
            panic!("un-enforced join state ordering");
        }
    }
    pub fn undouble(&mut self) {
        if let Self::Double {
            source,
            slot,
            partner_setting,
        } = *self
        {
            *self = Self::Single {
                source,
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
    pub fn get_source(&self) -> Option<SingleSource> {
        match &self {
            Join::Empty => None,
            Join::Joined { source, .. }
            | Join::Hover { source, .. }
            | Join::Single { source, .. }
            | Join::Double { source, .. } => Some(*source),
        }
    }
    pub fn is_source(&self, source: SingleSource) -> bool {
        matches!(self,
            Join::Joined { source: eq }
            | Join::Hover {source: eq, ..}
            | Join::Single { source: eq, .. }
            | Join::Double { source: eq, .. } if *eq == source,
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
impl ShowHide for TeamSelect {
    fn show(&mut self) {
        *self = Self {
            visible: true,
            ..Default::default()
        };
    }
    fn hide(&mut self) {
        self.visible = false
    }
}
impl TeamSelect {
    pub fn add_source(&mut self, source: SingleSource) {
        if !self.joins.iter().any(|join| join.is_source(source)) {
            for pad in &mut self.joins {
                if !pad.is_joined() {
                    pad.join(source);
                    return;
                }
            }
        }
    }
    pub fn remove_gamepad(&mut self, source: SingleSource) {
        for pad in &mut self.joins {
            if pad.is_source(source) {
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
    pub fn get_index_from_source(&self, source: SingleSource) -> Option<usize> {
        for (index, join) in self.joins.iter().enumerate() {
            if join.is_source(source) {
                return Some(index);
            }
        }
        None
    }
    pub fn contains_source(&self, source: SingleSource) -> bool {
        self.joins.iter().any(|join| join.is_source(source))
    }
    pub fn ready_join(&mut self, source: SingleSource) {
        let Some(index) = self.get_index_from_source(source) else {
            return;
        };
        let Some(slot) = self.joins[index].get_player_slot() else {
            return;
        };
        let dual_able = !self.is_player_slot_hovered(slot.partner());

        let join = &mut self.joins[index];

        if join.is_source(source) {
            if join.is_hovered() && !join.is_single() {
                join.single();
            } else if join.is_single()
                && !join.is_double()
                && dual_able
                && !join.is_source(SingleSource::KeyboardMouse)
            {
                join.double();
            }
        }
    }
    pub fn reverse_join(&mut self, source: SingleSource) {
        for join in &mut self.joins {
            if join.is_source(source) {
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
    pub fn left_join(&mut self, source: SingleSource) {
        let next_slot_a = self.next_slot_a();
        let cycle = 'cycle: {
            for join in &mut self.joins {
                if join.is_source(source) {
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
                    partner_setting,
                    source,
                    ..
                }) = self.get_mut_join_from_slot(player_slot)
                else {
                    unreachable!()
                };
                if *source != SingleSource::KeyboardMouse {
                    partner_setting.cycle();
                }
            }
        }
    }
    pub fn right_join(&mut self, source: SingleSource) {
        let next_slot_b = self.next_slot_b();
        let cycle = 'cycle: {
            for join in &mut self.joins {
                if join.is_source(source) {
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
                    partner_setting,
                    source,
                    ..
                }) = self.get_mut_join_from_slot(player_slot)
                else {
                    unreachable!()
                };
                if *source != SingleSource::KeyboardMouse {
                    partner_setting.cycle();
                }
            }
        }
    }
    pub fn is_double(&self, source: SingleSource) -> bool {
        for join in &self.joins {
            if join.is_source(source) {
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
                Join::Single { source, slot, .. } => {
                    if self.get_join_from_slot(slot.partner()).is_none() {
                        if let SingleSource::KeyboardMouse = source {
                            builder.insert(
                                *slot,
                                PlayerInfo::Local {
                                    number,
                                    source: *source,
                                    dual_stick: false,
                                },
                            );
                            builder.insert(slot.partner(), PlayerInfo::CPU);
                        } else {
                            return None;
                        }
                    } else {
                        builder.insert(
                            *slot,
                            PlayerInfo::Local {
                                number,
                                source: *source,
                                dual_stick: false,
                            },
                        );
                    }
                }
                Join::Double {
                    source,
                    slot,
                    partner_setting,
                } => match partner_setting {
                    PartnerSetting::CPU => {
                        builder.insert(
                            *slot,
                            PlayerInfo::Local {
                                number,
                                source: *source,
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
                                source: *source,
                                dual_stick: true,
                            },
                        );
                        builder.insert(
                            *slot,
                            PlayerInfo::Local {
                                number,
                                source: *source,
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
