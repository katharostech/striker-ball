use super::*;

#[derive(HasSchema, Clone)]
pub struct Fade {
    pub fade_out: Timer,
    pub fade_wait: Timer,
    pub fade_in: Timer,
    pub color: Color,
    pub order: egui::Order,
}
impl Default for Fade {
    fn default() -> Self {
        Self::new(3., 0.15, 1., Color::BLACK, egui::Order::Foreground)
    }
}
impl Fade {
    pub fn new(
        secs_out: f32,
        secs_wait: f32,
        secs_in: f32,
        color: Color,
        order: egui::Order,
    ) -> Self {
        let mut fade_out = Timer::from_seconds(secs_out, TimerMode::Once);
        let mut fade_wait = Timer::from_seconds(secs_wait, TimerMode::Once);
        let mut fade_in = Timer::from_seconds(secs_in, TimerMode::Once);
        fade_out.pause();
        fade_wait.pause();
        fade_in.pause();
        Self {
            fade_out,
            fade_wait,
            fade_in,
            color,
            order,
        }
    }
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color.set_r(color.r());
        self.color.set_g(color.r());
        self.color.set_b(color.r());
        self
    }
    pub fn order(&mut self, order: egui::Order) -> &mut Self {
        self.order = order;
        self
    }
    // This is used in the `PLAY` session so that the fade timing is synced in lan
    // matches.
    /// Restarts the `Fade` with the `fade_out` timer already finished.
    pub fn restart_at_wait(&mut self) {
        self.restart();
        self.fade_out.set_elapsed(self.fade_out.duration());
    }
    pub fn restart(&mut self) {
        self.fade_out.reset();
        self.fade_out.unpause();
        self.fade_wait.reset();
        self.fade_wait.pause();
        self.fade_in.reset();
        self.fade_in.pause();
    }
    pub fn finished(&self) -> bool {
        self.fade_out.finished() && self.fade_wait.finished() && self.fade_in.finished()
    }
}
impl SessionPlugin for Fade {
    fn install(self, session: &mut SessionBuilder) {
        session.insert_resource(self);
        session.add_system_to_stage(First, |world: &World, time: Res<Time>| {
            let Fade {
                fade_out,
                fade_wait,
                fade_in,
                color,
                ..
            } = &mut (*world.resource_mut::<Self>());

            fade_out.tick(time.delta());
            fade_wait.tick(time.delta());
            fade_in.tick(time.delta());

            if !fade_out.finished() {
                color.set_a(fade_out.percent());
            } else if !fade_wait.finished() {
                fade_wait.unpause();
                color.set_a(1.0);
            } else {
                fade_in.unpause();
                color.set_a(fade_in.percent_left());
            }
        });
    }
}
pub fn show(world: &World) {
    let Fade { color, order, .. } = *world.resource_mut::<Fade>();

    use egui::*;
    world
        .resource::<EguiCtx>()
        .layer_painter(LayerId::new(order, Id::new("FADE_OVERLAY")))
        .rect_filled(
            Rect::from_min_size(Pos2::ZERO, Vec2::INFINITY),
            Rounding::ZERO,
            color,
        );
}
