// TODO: Make this module striker_ball in-specific

use bones_framework::prelude::*;

// This goes on sessions with the `GgrsSessionRunner`.
pub struct NetworkFrameConfirmedValueLoggerPlugin;
impl SessionPlugin for NetworkFrameConfirmedValueLoggerPlugin {
    fn install(self, session: &mut SessionBuilder) {
        session.init_resource::<ConfirmedValueLogger>();
        session.add_startup_system(|| {
            ConfirmedValueLogger::clear_log_files()
                .unwrap_or_else(|e| tracing::warn!("error clearing log files for session: {e}"));
        });
        // This is striker_ball specific but I'll leave it here as reference.
        // session.add_system_to_stage(
        //     Last,
        //     |mut logger: ResMut<ConfirmedValueLogger>,
        //      info: Option<Res<SyncingInfo>>,
        //      time: Res<Time>,
        //      players: Comp<Player>,
        //      states: Comp<State>,
        //      entities: Res<Entities>,
        //      inputs: Res<PlayTeamInputs>| {
        //         if let Some(SyncingInfo::Online {
        //             local_player_idx,
        //             current_frame,
        //             last_confirmed_frame,
        //             ..
        //         }) = info.as_deref()
        //         {
        //             for (_entity, (player, state)) in entities.iter_with((&players, &states)) {
        //                 logger.update_state(player.id, state.current, *current_frame);
        //             }
        //             logger.update_inputs((*inputs).clone(), *current_frame);
        //             logger.update_delta(time.delta(), *current_frame);
        //             logger.update(*local_player_idx, *current_frame, *last_confirmed_frame);
        //         }
        //     },
        // );
    }
}

#[derive(HasSchema, Clone, Debug)]
pub struct NetworkFrameConfirmedValueLogger {
    pub next_frame: i32,
    pub buffer: HashMap<i32, ConfirmValue>,
}
impl Default for NetworkFrameConfirmedValueLogger {
    fn default() -> Self {
        Self {
            next_frame: 1,
            buffer: Default::default(),
        }
    }
}

// TODO: Try turning this into a type-map for future use.
#[derive(Clone, Debug, Default)]
pub struct NetworkFrameConfirmedValue {
    pub inputs: Option<PlayTeamInputs>,
    pub distances: HashMap<usize, f32>,
    pub states: HashMap<PlayerSlot, Ustr>,
    pub finished: bool,
    pub elapsed: std::time::Duration,
    pub delta: std::time::Duration,
}

use std::fs::*;
use std::io;
use std::io::Write;
use std::path::PathBuf;

impl NetworkFrameConfirmedValueLogger {
    pub fn get_log_filepath(client: usize) -> io::Result<PathBuf> {
        let mut path = std::env::current_exe()?;
        path.pop();
        path.push(format!("striker_ball_logs/client_{client}.txt"));
        Ok(path)
    }
    pub fn get_log_file(client: usize) -> io::Result<File> {
        let path = Self::get_log_filepath(client)?;
        std::fs::create_dir_all(path.parent().unwrap())?;
        OpenOptions::new().append(true).create(true).open(path)
    }
    pub fn log_to_file(client: usize, current_frame: i32, value: ConfirmValue) -> io::Result<()> {
        let mut file = Self::get_log_file(client)?;
        std::writeln!(
            &mut file,
            "frame={current_frame:?}, confirmed_value={value:#?};"
        )
    }
    pub fn clear_log_files() -> io::Result<()> {
        let path = Self::get_log_filepath(0)?;
        std::fs::write(path, "")?;
        let path = Self::get_log_filepath(1)?;
        std::fs::write(path, "")
    }
    pub fn update_inputs(&mut self, inputs: PlayTeamInputs, current_frame: i32) {
        self.buffer.entry(current_frame).or_default().inputs = Some(inputs.clone());
    }
    pub fn update_distance(&mut self, index: usize, distance: f32, current_frame: i32) {
        *self
            .buffer
            .entry(current_frame)
            .or_default()
            .distances
            .entry(index)
            .or_default() = distance;
    }
    pub fn update_state(&mut self, slot: PlayerSlot, state: Ustr, current_frame: i32) {
        *self
            .buffer
            .entry(current_frame)
            .or_default()
            .states
            .entry(slot)
            .or_default() = state;
    }
    pub fn update_finished(&mut self, finished: bool, current_frame: i32) {
        self.buffer.entry(current_frame).or_default().finished = finished;
    }
    pub fn update_elapsed(&mut self, elapsed: std::time::Duration, current_frame: i32) {
        self.buffer.entry(current_frame).or_default().elapsed = elapsed;
    }
    pub fn update_delta(&mut self, delta: std::time::Duration, current_frame: i32) {
        self.buffer.entry(current_frame).or_default().delta = delta;
    }
    pub fn update(&mut self, client: usize, current_frame: i32, last_confirmed_frame: i32) {
        while self.next_frame <= current_frame && current_frame <= last_confirmed_frame {
            let frame = self.next_frame;
            let confirmed_value = self.buffer.remove(&frame).unwrap_or_else(|| {
                panic!("frame discrepancy: log_frame={frame}, current_frame={current_frame}")
            });

            // tracing::info!(?frame, ?confirmed_value);

            Self::log_to_file(client, frame, confirmed_value)
                .unwrap_or_else(|e| tracing::warn!("error logging to file: {e:?}"));

            self.next_frame += 1;
        }
    }
}
