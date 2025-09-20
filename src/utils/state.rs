//! To use the states just install the [`StatePlugin`] into the session you want the states to be updated in.
//!
//! The [`State`] component contains a [`Ustr`] for the state id and data for the number of frames the current
//! state has been active.
//!
//! Installing the [`StatePlugin`] adds a [`SystemStage`] to your [`Session`] under the label [`StateStage`]
//! that runs before [`CoreStage::PreUpdate`].
//!
//! The systems under [`StateStage`] run several times if needed to ensure that the states finish
//! all their potential transitions in one frame without delays that could cause presumably impossible
//! game scenarios.
//!
//! Since the systems can potentially run multiple times the majority of your mutations should change only
//! [`State::current`] and few other necessary state transition related changes to avoid the transition
//! loop getting stuck in an infinite loop.

use bones_framework::prelude::*;
use tracing::trace;

#[derive(HasSchema, Clone, Default)]
pub struct State {
    /// The ID for the current state.
    pub current: Ustr,
    /// The number of frames that the pre_transition state has been current.
    age: u64,
    /// The time that the pre_transition state has been current.
    duration: std::time::Duration,
    /// The state of [`Self::current`], before the state transition systems ran last.
    /// This is used to check whether or not the state changed after a loop in the [`StateStage`].
    pre_transition: Ustr,
}
impl State {
    pub fn new(name: &str) -> Self {
        let current = ustr(name);
        Self {
            current,
            age: default(),
            duration: default(),
            pre_transition: current,
        }
    }
    /// Gets the number of frames the current state has been active.
    pub fn age(&self) -> u64 {
        if self.pre_transition == self.current {
            self.age
        } else {
            0
        }
    }
    /// Gets the number of frames the current state has been active.
    pub fn duration(&self) -> std::time::Duration {
        if self.pre_transition == self.current {
            self.duration
        } else {
            default()
        }
    }
    /// For use in transition systems. Gets the age of the pre_transition state if the
    /// current state has changed in the [`StateStage`].
    /// If it hasn't changed the function will return [`None`] meaning the current
    /// state is `0` frames old and the pre_transition age is lost.
    pub fn get_previous_age(&self) -> Option<u64> {
        (self.pre_transition != self.current).then_some(self.age)
    }
}

#[derive(Debug)]
pub struct StateStage;

impl StageLabel for StateStage {
    fn name(&self) -> String {
        format!("{self:?}")
    }
    fn id(&self) -> Ulid {
        Ulid(2022686805174362721866480948664103805)
    }
}
pub struct StateStageImpl<Label>
where
    Label: StageLabel + Sync + Send + 'static,
{
    label: Label,
    systems: Vec<StaticSystem<(), ()>>,
}
impl<L> StateStageImpl<L>
where
    L: StageLabel + Sync + Send + 'static,
{
    pub fn new(label: L) -> Self {
        Self {
            label,
            systems: Vec::new(),
        }
    }
}
impl<L> SystemStage for StateStageImpl<L>
where
    L: StageLabel + Sync + Send + 'static,
{
    fn id(&self) -> Ulid {
        self.label.id()
    }
    fn name(&self) -> String {
        self.label.name()
    }
    fn run(&mut self, world: &World) {
        trace!("Starting state transitions");
        loop {
            trace!("Running state transitions");
            for system in &mut self.systems {
                system.run(world, ());
            }

            // Updates and returns whether any states have changed after transitions
            fn update_states(entities: Res<Entities>, mut states: CompMut<State>) -> bool {
                let mut any_changed = false;

                for (_ent, state) in entities.iter_with(&mut states) {
                    if state.pre_transition != state.current {
                        any_changed = true;
                        // TODO: May need to wait until later to reset `state.age` & `state.pre_transition`
                        // since the state could still return to the pre_transition state before the next frame.
                        state.age = 0;
                        state.duration = default();
                        state.pre_transition = state.current;
                    }
                }
                any_changed
            }
            let any_changed = world.run_system(update_states, ());

            if !any_changed {
                trace!("No state changes, done with state transition loop");
                break;
            }
        }
    }
    fn add_system(&mut self, system: StaticSystem<(), ()>) {
        self.systems.push(system.system())
    }
    fn remove_all_systems(&mut self) {
        self.systems = Vec::new();
    }
}

pub struct StatePlugin;
impl SessionPlugin for StatePlugin {
    fn install(self, session: &mut SessionBuilder) {
        session
            .stages
            .insert_stage_before(PreUpdate, StateStageImpl::new(StateStage));
        session.stages.add_system_to_stage(Last, update_state_age);
    }
}
pub fn update_state_age(time: Res<Time>, entities: Res<Entities>, mut states: CompMut<State>) {
    for (_ent, state) in entities.iter_with(&mut states) {
        // TODO: should we warn if these saturate?
        state.duration = state.duration.saturating_add(time.delta());
        state.age = state.age.saturating_add(1);
    }
}
