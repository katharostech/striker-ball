use bones_framework::prelude::*;

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
