use super::*;

#[derive(HasSchema, Clone)]
pub struct Countdown {
    pub visual: Visual,
    pub timer: Timer,
    pub speed: f32,
    pub sound_marker: f32,
}

impl Countdown {
    pub fn new(seconds: f32, speed: f32) -> Self {
        Self {
            speed,
            timer: Timer::from_seconds(seconds, TimerMode::Once),
            visual: Visual::default(),
            sound_marker: -1.0,
        }
    }
    pub fn restart(&mut self) {
        self.visual.show();
        self.timer.reset();
    }
    pub fn tick(&mut self, delta: std::time::Duration) {
        self.timer.tick(delta.mul_f32(self.speed));
    }
}
impl Default for Countdown {
    fn default() -> Self {
        Self {
            visual: Visual::default(),
            timer: Timer::from_seconds(4.0, TimerMode::Once),
            speed: 1.0,
            sound_marker: -1.0,
        }
    }
}
impl SessionPlugin for Countdown {
    fn install(self, session: &mut SessionBuilder) {
        session.insert_resource(self);
        session.add_system_to_stage(First, |world: &World, time: Res<Time>| {
            world.resource_mut::<Self>().tick(time.delta());
        });
    }
}
pub fn show(world: &World) {
    let mut countdown = world.resource_mut::<Countdown>();

    if !countdown.visual.shown() {
        return;
    }

    let asset_server = world.resource::<AssetServer>();
    let root = asset_server.root::<Data>();
    let ctx = world.resource::<EguiCtx>();
    let textures = world.resource::<EguiTextures>();

    let Sounds {
        countdown_first,
        countdown_final,
        ..
    } = root.sound;

    let Menus {
        numbers,
        go_text,
        countdown_bg,
        ..
    } = root.menu;

    let shadow_offset = Vec2::splat(2.0);

    use egui::*;

    let area = Area::new("countdown")
        .anchor(Align2::CENTER_CENTER, [0., 0.])
        .show(&ctx, |ui| {
            ui.set_width(root.screen_size.x);
            ui.set_height(root.screen_size.y);
        });
    let origin = area.response.rect.center();

    let mut painter =
        ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("countdown_canvas")));

    painter.set_clip_rect(area.response.rect);

    countdown_bg
        .image_painter()
        .align2(Align2::CENTER_CENTER)
        .pos(origin)
        .paint(&painter, &textures);

    if !countdown.timer.finished() {
        let duration = countdown.timer.duration().as_secs_f32();
        let progress = (countdown.timer.percent_left() * duration).ceil() - 1.0;

        if progress > 0.0 {
            if progress != countdown.sound_marker {
                countdown.sound_marker = progress;
                world
                    .resource_mut::<AudioCenter>()
                    .play_sound(*countdown_first, countdown_first.volume());
            }
            let asset = asset_server.get(numbers);
            let width = asset.tile_size.x;
            let height = asset.tile_size.y;
            let shift = 1.0 / asset.rows as f32;

            painter.image(
                textures.get(asset.image),
                Rect::from_center_size(origin + shadow_offset, Vec2::new(width, height)),
                Rect::from_min_max(
                    pos2(0.0, shift * progress),
                    pos2(1.0, shift * progress + shift),
                ),
                Color32::BLACK,
            );

            painter.image(
                textures.get(asset.image),
                Rect::from_center_size(origin, Vec2::new(width, height)),
                Rect::from_min_max(
                    pos2(0.0, shift * progress),
                    pos2(1.0, shift * progress + shift),
                ),
                Color32::WHITE,
            );
        } else {
            if progress != countdown.sound_marker {
                countdown.sound_marker = progress;
                world
                    .resource_mut::<AudioCenter>()
                    .play_sound(*countdown_final, countdown_final.volume());
            }

            painter.image(
                textures.get(*go_text),
                Rect::from_center_size(origin + shadow_offset, go_text.egui_size()),
                default_uv(),
                Color32::BLACK,
            );

            painter.image(
                textures.get(*go_text),
                Rect::from_center_size(origin, go_text.egui_size()),
                default_uv(),
                Color32::WHITE,
            );
        }
    } else {
        countdown.visual.hide();
    }
}
