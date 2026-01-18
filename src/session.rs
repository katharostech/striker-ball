use super::*;

pub const UI: &str = "ui";
pub const UI_PRIORITY: i32 = 2;

pub const PLAY: &str = "play";
pub const PLAY_PRIORITY: i32 = 1;

pub const TARGET_FPS: f64 = 60.0;
pub const TARGET_STEP: f64 = 1.0 / TARGET_FPS;

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
/// A fixed time step runner.
///
/// This runner is meant to process frames once per 60th of a second. Typically,
/// when `Self::step` is called, the amount of time elapsed in between calls is
/// not going to be exactly the same. This fixed time step runner uses its
/// accumulator to track that difference in elapsed time and either wait to step
/// if the call was too soon, or run an extra step if it was a 60th of a second
/// late. Calls won't ever be exactly on time but this runner helps
/// significantly for maintaining speed.
///
/// If the runner starts running a deficit, this means that the time it took to
/// run its steps so far took longer than a 60th of a second. When that happens,
/// the runner resets the accumulator and skips any remaining steps, slowing the
/// game down more than the target speed.
///
/// Visually, when multiple steps are made to catch up in speed, you will miss
/// some rendered frames. The runner's steps are processed before rendering, so
/// you will still see only the last frame rendered for every "multi-step".
pub struct OfflineRunner {
    pub accumulator: f64,
    pub last_run: Option<Instant>,
    pub disable_local_input: bool,
    pub collectors: [PlayTeamInputCollector; 2],
}
impl SessionRunner for OfflineRunner {
    fn step(&mut self, frame_start: Instant, world: &mut World, stages: &mut SystemStages) {
        pub const STEP: f64 = TARGET_STEP;

        let last_run = self.last_run.unwrap_or(frame_start);
        let delta = (frame_start - last_run).as_secs_f64();
        self.accumulator += delta;

        let steps = self.accumulator / STEP;

        if steps >= 2.0 {
            tracing::debug!(?steps, "multi-step frame");
        }

        for collector in &mut self.collectors {
            collector.apply_inputs(world);
        }

        let loop_start = Instant::now();

        while self.accumulator >= STEP {
            let loop_too_long = loop_start.elapsed().as_secs_f64() > STEP;

            if loop_too_long {
                tracing::warn!("fixed time step took too long. (game will slow)");
                self.accumulator = 0.0;
                break;
            }

            self.accumulator -= STEP;

            world
                .resource_mut::<Time>()
                .advance_exact(std::time::Duration::from_secs_f64(STEP));

            if self.disable_local_input {
                for client in world.resource_mut::<PlayTeamInputs>().clients.iter_mut() {
                    *client = default();
                }
            } else {
                for (i, client) in world
                    .resource_mut::<PlayTeamInputs>()
                    .clients
                    .iter_mut()
                    .enumerate()
                {
                    client.update_from_dense(
                        &self.collectors[i]
                            .get_control(/* both of these are unused */ 0, Default::default())
                            .get_dense_input(),
                    );
                }
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
