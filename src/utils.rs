//! All things in this module are independant of the game.
//!
//! This means it doesn't belong here if it needs `use crate::*;`.
//!
//! Most of the structures here are potential bones_framework additions
//! or will be separate crates in the future.

mod egui;
mod follow;
mod input;
mod lifetime;
mod path2d;
mod state;

pub use egui::*;
pub use follow::*;
pub use input::*;
pub use lifetime::*;
pub use path2d::*;
pub use state::*;

use bones_framework::prelude::*;

// TODO: Use this once bones' HasSchema supports generic types.
// #[derive(HasSchema, Clone, Default, Deref, DerefMut)]
// pub struct UIElement<T> {
//     #[deref]
//     pub visual: Visual,
//     pub element: T,
// }

/// Handles visibility with [`Self::show`] & [`Self::hide`],
/// but lets you request to hide so other elements can override visibility.
///
/// Use [`Self::add_hide`] and [`Self::remove_hide`] to add and remove
/// requests to hide.
///
/// [`Self::shown`] returns whether or not the element should be visible.
#[derive(HasSchema, Clone, Default, Copy)]
pub struct Visual {
    /// Whether or not there is any desire to be visible at all.
    pub show: bool,
    /// The amount of requests to hide. If this is > 0
    /// [`Self::shown`] returns false.
    ///
    /// You can use [`Self::add_hide`] & [`Self::remove_hide`]
    /// to modify this easily.
    pub poll: u32,
}
impl Visual {
    pub fn new_shown() -> Self {
        Visual {
            show: true,
            ..Default::default()
        }
    }
    pub fn new_hidden() -> Self {
        Visual {
            show: false,
            ..Default::default()
        }
    }
    /// Whether or not the associated element should be seen.
    ///
    /// Returns `true` if [`Self::show`] is true and there are
    /// no requests to hide.
    pub fn shown(&self) -> bool {
        self.show && self.poll == 0
    }
    pub fn show(&mut self) {
        self.show = true;
    }
    pub fn hide(&mut self) {
        self.show = false;
    }
    /// Adds 1 to the requests that the visual be hidden.
    pub fn add_hide(&mut self) {
        self.poll += 1;
    }
    /// Removes 1 from the requests that the visual be hidden.
    pub fn remove_hide(&mut self) {
        self.poll -= 1;
    }
}

/// `layers![GROUND, CHARACTER, OVERLAY]` is equivalent to:
/// ```
/// pub const GROUND: f32 = 0.;
/// pub const CHARACTER: f32 = 1.;
/// pub const OVERLAY: f32 = 2.;
/// // etc...
/// ```
#[macro_export]
macro_rules! layers {
    ( $($layers:ident),* $(,)?) => {
        $crate::layers!(@ $($layers),* => 1. );
    };

    (@ $first:ident $(, $others:ident )* $(,)? => $index:expr ) => {
        pub const $first: f32 = $index;
        $crate::layers!(@ $($others),* => $index + 1. );
    };

    (@ => $index:expr ) => {};
}

/// `states![idle, walk, jump]` is equivalent to:
/// ```
/// pub fn idle() -> ustr::Ustr {
///     ustr::ustr("idle")
/// }
/// pub fn walk() -> ustr::Ustr {
///     ustr::ustr("walk")
/// }
/// pub fn jump() -> ustr::Ustr {
///     ustr::ustr("jump")
/// }
///     // etc...
/// ```
#[macro_export]
macro_rules! states {
    ($($id:ident),* $(,)?) => {
        $(
            pub fn $id() -> bones_framework::prelude::Ustr {
                bones_framework::prelude::ustr(stringify!($id))
            }
        )*
    };
}

pub trait SessionAccess {
    fn get_world(&mut self, name: impl Into<Ustr>) -> Option<&World>;
    fn get_session_resource<T: HasSchema>(&mut self, name: impl Into<Ustr>) -> Option<Ref<'_, T>>;
    fn get_session_resource_mut<T: HasSchema>(
        &mut self,
        name: impl Into<Ustr>,
    ) -> Option<RefMut<'_, T>>;
}
impl SessionAccess for Sessions {
    fn get_world(&mut self, name: impl Into<Ustr>) -> Option<&World> {
        self.get_mut(name).map(|session| &session.world)
    }
    fn get_session_resource<T: HasSchema>(&mut self, name: impl Into<Ustr>) -> Option<Ref<'_, T>> {
        self.get_mut(name).map(|session| session.world.resource())
    }
    fn get_session_resource_mut<T: HasSchema>(
        &mut self,
        name: impl Into<Ustr>,
    ) -> Option<RefMut<'_, T>> {
        self.get_mut(name)
            .map(|session| session.world.resource_mut())
    }
}

pub trait GamepadEventExt {
    fn gamepad_id(&self) -> &u32;
}
impl GamepadEventExt for GamepadEvent {
    fn gamepad_id(&self) -> &u32 {
        match self {
            GamepadEvent::Connection(GamepadConnectionEvent { gamepad, .. }) => gamepad,
            GamepadEvent::Button(GamepadButtonEvent { gamepad, .. }) => gamepad,
            GamepadEvent::Axis(GamepadAxisEvent { gamepad, .. }) => gamepad,
        }
    }
}
pub trait WorldExtra {
    #[track_caller]
    fn spawn(&self) -> EntityOps;
    fn entity_ops(&self, entity: Entity) -> EntityOps;
    #[track_caller]
    fn add_command<Args, S>(&self, system: S)
    where
        S: IntoSystem<Args, (), (), Sys = StaticSystem<(), ()>>;
    #[track_caller]
    fn asset_server(&self) -> Ref<AssetServer>;
}
impl WorldExtra for World {
    fn spawn(&self) -> EntityOps {
        EntityOps {
            entity: self.resource_mut::<Entities>().create(),
            world: self,
        }
    }
    fn entity_ops(&self, entity: Entity) -> EntityOps {
        EntityOps {
            entity,
            world: self,
        }
    }
    fn add_command<Args, S>(&self, system: S)
    where
        S: IntoSystem<Args, (), (), Sys = StaticSystem<(), ()>>,
    {
        self.resource_mut::<CommandQueue>().add(system);
    }
    fn asset_server(&self) -> Ref<AssetServer> {
        self.resource::<AssetServer>()
    }
}

pub struct EntityOps<'w> {
    pub entity: Entity,
    pub world: &'w World,
}
impl<'w> EntityOps<'w> {
    pub fn id(&self) -> Entity {
        self.entity
    }
    pub fn insert<C: HasSchema>(&mut self, component: C) -> &mut Self {
        let cell = self.world.components.get_cell();
        cell.borrow_mut().insert(self.entity, component);
        self
    }
    pub fn add(&mut self, f: fn(&mut Self)) -> &mut Self {
        f(self);
        self
    }
}

pub trait TransformExt {
    fn from_z(z: f32) -> Self;
}

impl TransformExt for Transform {
    fn from_z(z: f32) -> Self {
        Transform::from_translation(Vec3 {
            z,
            ..Default::default()
        })
    }
}

pub trait Vec3Ext {
    fn from_z(z: f32) -> Self;
}

impl Vec3Ext for Vec3 {
    fn from_z(z: f32) -> Self {
        Vec3 {
            z,
            ..Default::default()
        }
    }
}
