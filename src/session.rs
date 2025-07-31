use super::*;

pub const UI: &str = "ui";
pub const UI_PRIORITY: i32 = 2;

pub const PLAY: &str = "play";
pub const PLAY_PRIORITY: i32 = 1;

// NOTE: session creation may need to have an immediate and delayed command versions for each session
pub trait SessionCreation {
    fn create_play(&mut self, mode: PlayMode);
    fn delete_play(&mut self);
}
impl SessionCreation for Sessions {
    fn create_play(&mut self, mode: PlayMode) {
        self.add_command(Box::new(move |sessions| {
            sessions.create_with(PLAY, PlayPlugin { mode });
        }));
    }
    fn delete_play(&mut self) {
        self.add_command(Box::new(|sessions| {
            sessions.delete(PLAY);
        }));
    }
}

#[derive(Default)]
pub struct OfflineRunner {
    pub accumulator: f64,
    pub last_run: Option<Instant>,
    pub disable_local_input: bool,
}
impl SessionRunner for OfflineRunner {
    fn step(&mut self, frame_start: Instant, world: &mut World, stages: &mut SystemStages) {
        pub const STEP: f64 = 1.0 / 60.;

        let last_run = self.last_run.unwrap_or(frame_start);
        let delta = (frame_start - last_run).as_secs_f64();

        self.accumulator += delta;

        if self.accumulator >= STEP {
            self.accumulator -= STEP;

            world
                .resource_mut::<Time>()
                .advance_exact(std::time::Duration::from_secs_f64(STEP));

            *world.resource_mut::<PlayInputs>() = if self.disable_local_input {
                PlayInputs::default()
            } else {
                PlayInputs::from_world(world)
            };
            stages.run(world);
        }

        self.last_run = Some(frame_start);
    }

    fn restart_session(&mut self) {
        *self = OfflineRunner::default();
    }

    fn disable_local_input(&mut self, disable_input: bool) {
        self.disable_local_input = disable_input;
    }
}
