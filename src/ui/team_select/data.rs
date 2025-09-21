use core::iter::Iterator;

use super::*;
use crate::play::*;
use crate::player::*;

#[derive(HasSchema, Clone, Default)]
pub enum Join {
    #[default]
    Empty,
    Joined {
        gamepad: u32,
    },
    Set {
        gamepad: u32,
        slot: PlayerSlot,
    },
    Ready {
        gamepad: u32,
        slot: PlayerSlot,
        dual_stick: bool,
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
    pub fn set(&mut self, slot: PlayerSlot) {
        if let Self::Joined { gamepad } = *self {
            *self = Self::Set { gamepad, slot }
        } else {
            panic!("un-enforced join state ordering");
        }
    }
    pub fn unset(&mut self) {
        if let Self::Set { gamepad, .. } = *self {
            *self = Self::Joined { gamepad }
        } else {
            panic!("un-enforced join state ordering");
        }
    }
    pub fn ready(&mut self) {
        if let Self::Set { gamepad, slot } = *self {
            *self = Self::Ready {
                gamepad,
                slot,
                dual_stick: false,
            }
        } else {
            panic!("un-enforced join state ordering");
        }
    }
    pub fn unready(&mut self) {
        if let Self::Ready { gamepad, slot, .. } = *self {
            *self = Self::Set { gamepad, slot }
        } else {
            panic!("un-enforced join state ordering");
        }
    }
    pub fn dual_stick_ready(&mut self) {
        if let Self::Ready { gamepad, slot, .. } = *self {
            *self = Self::Ready {
                gamepad,
                slot,
                dual_stick: true,
            }
        } else {
            panic!("un-enforced join state ordering");
        }
    }
    pub fn un_dual_stick(&mut self) {
        if let Self::Ready { gamepad, slot, .. } = *self {
            *self = Self::Ready {
                gamepad,
                slot,
                dual_stick: false,
            }
        } else {
            panic!("un-enforced join state ordering");
        }
    }
    pub fn get_player_slot(&self) -> Option<PlayerSlot> {
        match &self {
            Join::Empty | Join::Joined { .. } => None,
            Join::Set { slot, .. } | Join::Ready { slot, .. } => Some(*slot),
        }
    }
    pub fn is_gamepad_id(&self, gamepad_id: u32) -> bool {
        matches!(self,
            Join::Joined { gamepad }
            | Join::Set { gamepad, .. }
            | Join::Ready { gamepad, .. } if *gamepad == gamepad_id,
        )
    }
    pub fn is_player_id(&self, id: PlayerSlot) -> bool {
        matches!(self, Join::Set { slot, .. } | Join::Ready { slot, .. } if *slot == id)
    }
    pub fn is_joined(&self) -> bool {
        matches!(
            self,
            Join::Joined { .. } | Join::Set { .. } | Join::Ready { .. }
        )
    }
    pub fn is_set(&self) -> bool {
        matches!(self, Join::Set { .. } | Join::Ready { .. })
    }
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready { .. })
    }
    pub fn is_dual_stick(&self) -> bool {
        matches!(
            self,
            Self::Ready {
                dual_stick: true,
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
        let dual_able = !self.is_player_slot_set(slot.partner());

        let join = &mut self.joins[index];

        if join.is_gamepad_id(id) {
            if join.is_set() && !join.is_ready() {
                join.ready();
            } else if join.is_ready() && dual_able {
                join.dual_stick_ready();
            }
        }
    }
    pub fn reverse_gamepad(&mut self, id: u32) {
        for join in &mut self.joins {
            if join.is_gamepad_id(id) {
                if join.is_dual_stick() {
                    join.un_dual_stick()
                } else if join.is_ready() {
                    join.unready();
                } else if join.is_set() {
                    join.unset();
                } else if join.is_joined() {
                    join.unjoin();
                }
            }
        }
    }
    pub fn dual_ready(&mut self, id: u32) {
        let mut player_id = None;
        let mut index = None;

        for (i, join) in self.joins.iter_mut().enumerate() {
            if join.is_gamepad_id(id) {
                player_id = join.get_player_slot();
                index = Some(i);
            }
        }
        if let Some(player_id) = player_id {
            if !self.is_player_slot_set(player_id.partner()) {
                let join = &mut self.joins[index.unwrap()];
                if join.is_ready() {
                    join.dual_stick_ready()
                }
            }
        }
    }
    pub fn next_slot_a(&self) -> Option<PlayerSlot> {
        let mut a1 = false;
        let mut a2 = false;
        for join in &self.joins {
            if join.is_set() {
                if join.is_player_id(PlayerSlot::A1) {
                    a1 = true;
                    if join.is_dual_stick() {
                        a2 = true
                    }
                }
                if join.is_player_id(PlayerSlot::A2) {
                    a2 = true;
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
            if join.is_set() {
                if join.is_player_id(PlayerSlot::B1) {
                    b1 = true;
                    if join.is_dual_stick() {
                        b2 = true
                    }
                }
                if join.is_player_id(PlayerSlot::B2) {
                    b2 = true;
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
        for join in &mut self.joins {
            if let Join::Set {
                gamepad,
                slot: player_id,
            } = join
            {
                if *gamepad == id && player_id.team() == Team::B {
                    join.unset();
                }
            } else if let Join::Joined { gamepad } = join {
                if *gamepad == id {
                    if let Some(player_id) = next_slot_a {
                        join.set(player_id);
                    }
                }
            }
        }
    }
    pub fn right_gamepad(&mut self, id: u32) {
        let next_slot_b = self.next_slot_b();
        for join in &mut self.joins {
            if let Join::Set {
                gamepad,
                slot: player_id,
            } = join
            {
                if *gamepad == id && player_id.team() == Team::A {
                    join.unset();
                }
            } else if let Join::Joined { gamepad } = join {
                if *gamepad == id {
                    if let Some(player_id) = next_slot_b {
                        join.set(player_id);
                    }
                }
            }
        }
    }
    pub fn is_ready(&self, id: u32) -> bool {
        for join in &self.joins {
            if join.is_gamepad_id(id) {
                return join.is_ready();
            }
        }
        false
    }
    pub fn is_player_slot_dual_stick(&self, id: PlayerSlot) -> bool {
        self.joins
            .iter()
            .any(|join| join.is_player_id(id) && join.is_dual_stick())
    }
    pub fn is_player_id_ready(&self, id: PlayerSlot) -> bool {
        self.joins.iter().any(|join| {
            join.is_player_id(id) && join.is_ready()
                || join.is_player_id(id.partner()) && join.is_dual_stick()
        })
    }
    pub fn is_player_slot_set(&self, id: PlayerSlot) -> bool {
        self.joins
            .iter()
            .any(|join| join.is_player_id(id) && join.is_set())
    }
    pub fn get_player_signs(&self) -> Option<PlayersInfo> {
        let mut builder = PlayerInfoBuilder::default();

        for (number, join) in self.joins.iter().enumerate() {
            if let Join::Ready {
                gamepad,
                slot,
                dual_stick,
            } = *join
            {
                builder.insert(PlayerInfo {
                    number,
                    gamepad,
                    dual_stick,
                    slot,
                });
            }
        }
        builder.finish()
    }
}

#[derive(Default)]
pub struct PlayerInfoBuilder {
    team_a: TeamInfoBuilder,
    team_b: TeamInfoBuilder,
}
impl PlayerInfoBuilder {
    fn insert(&mut self, player: PlayerInfo) {
        let builder = match player.slot.team() {
            Team::A => &mut self.team_a,
            Team::B => &mut self.team_b,
        };
        if player.dual_stick {
            builder.insert_dual_stick(player);
        } else if player.slot.is_primary() {
            builder.insert_primary(player);
        } else {
            builder.insert_secondary(player);
        }
    }
    fn finish(self) -> Option<PlayersInfo> {
        Some(PlayersInfo {
            team_a: self.team_a.finish()?,
            team_b: self.team_b.finish()?,
        })
    }
}

#[derive(Clone, Copy, Default)]
enum TeamInfoBuilder {
    #[default]
    Empty,
    Primary(PlayerInfo),
    Secondary(PlayerInfo),
    SinglePlayer(PlayerInfo),
    TwoPlayer(PlayerInfo, PlayerInfo),
}
impl TeamInfoBuilder {
    fn insert_dual_stick(&mut self, player: PlayerInfo) {
        *self = match *self {
            Self::Empty => Self::SinglePlayer(player),
            Self::Secondary(..)
            | Self::Primary(..)
            | Self::SinglePlayer(..)
            | Self::TwoPlayer(..) => {
                panic!("team slot taken twice")
            }
        }
    }
    fn insert_primary(&mut self, player: PlayerInfo) {
        *self = match *self {
            Self::Empty => Self::Primary(player),
            Self::Secondary(secondary) => Self::TwoPlayer(player, secondary),
            Self::Primary(..) | Self::SinglePlayer(..) | Self::TwoPlayer(..) => {
                panic!("team slot taken twice")
            }
        }
    }
    fn insert_secondary(&mut self, player: PlayerInfo) {
        *self = match *self {
            Self::Empty => Self::Secondary(player),
            Self::Primary(primary) => Self::TwoPlayer(primary, player),
            Self::Secondary(..) | Self::SinglePlayer(..) | Self::TwoPlayer(..) => {
                panic!("team slot taken twice")
            }
        }
    }
    fn finish(self) -> Option<TeamInfo> {
        match self {
            TeamInfoBuilder::Empty
            | TeamInfoBuilder::Primary(..)
            | TeamInfoBuilder::Secondary(..) => None,
            TeamInfoBuilder::SinglePlayer(primary) => TeamInfo::SinglePlayer(primary).into(),
            TeamInfoBuilder::TwoPlayer(primary, secondary) => {
                TeamInfo::TwoPlayer(primary, secondary).into()
            }
        }
    }
}
