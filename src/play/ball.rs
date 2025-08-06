use super::*;

pub mod prelude {
    pub use super::Ball;
}

#[derive(HasSchema, Clone)]
#[repr(C)]
pub struct Ball {
    pub velocity: Vec2,
    pub bounced: bool,
    pub owner: Maybe<Entity>,
    pub dribble_pos: Vec2,
    pub sound_timer: Timer,
}
impl Default for Ball {
    fn default() -> Self {
        Self {
            velocity: Default::default(),
            bounced: Default::default(),
            owner: Maybe::Unset,
            dribble_pos: default(),
            sound_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        }
    }
}

pub fn sprite() -> AnimatedSprite {
    AnimatedSprite {
        frames: vec![0, 1, 2, 3].into(),
        fps: 20.,
        repeat: true,
        ..Default::default()
    }
}

pub fn plugin(session: &mut SessionBuilder) {
    session.add_system_to_stage(Update, update_ball);
}

pub fn update_ball(
    entities: Res<Entities>,
    root: Root<Data>,
    players: Comp<Player>,
    mut paths: CompMut<Path2d>,
    mut audio: ResMut<AudioCenter>,
    mut balls: CompMut<Ball>,
    mut animated_sprites: CompMut<AnimatedSprite>,
    mut transforms: CompMut<Transform>,
) {
    let Constants {
        ball_bounds,
        player_radius,
        ball_radius,
        ball_friction,
        ball_etransfer,
        ball_border_slide,
        dribble_smoothing,
        dribble_smoothing_threshold,
        ..
    } = root.constant;

    let Sounds { ball_spin, ball_bounced, .. } = root.sound;

    for (ball_entity, (ball, animation)) in entities.iter_with((&mut balls, &mut animated_sprites))
    {
        if let Maybe::Set(target) = ball.owner {
            // Dribble
            let player = players.get(target).unwrap();
            let player_pos = transforms.get(target).unwrap().translation.xy();
            let pos = &mut transforms.get_mut(ball_entity).unwrap().translation;

            let target = player.angle * (player_radius + ball_radius);
            let diff = target - ball.dribble_pos;
            let movement = if diff.length() > dribble_smoothing_threshold {
                diff / dribble_smoothing
            } else {
                diff
            };

            ball.dribble_pos += movement;

            pos.x = player_pos.x + ball.dribble_pos.x;
            pos.y = player_pos.y + ball.dribble_pos.y;

            // ball.velocity = movement; TODO: Maybe use this to spin ball while dribbling
            ball.bounced = false;
        } else {
            let pos = &mut transforms.get_mut(ball_entity).unwrap().translation;

            // Friction
            if ball.bounced {
                ball.velocity *= ball_friction;
            }
            if ball.velocity.length() < 0.01 {
                ball.velocity = Vec2::ZERO;
            }

            // Movement
            pos.x += ball.velocity.x;
            pos.y += ball.velocity.y;
        }
        // Animation Speed
        animation.fps = 10.0 * ball.velocity.length();

        // Sound
        ball.sound_timer
            .tick(std::time::Duration::from_secs_f32(ball.velocity.length()));

        if ball.sound_timer.just_finished() {
            audio.play_sound(*ball_spin, ball_spin.volume());
        }

        let pos = &mut transforms.get_mut(ball_entity).unwrap().translation;

        // Bounds
        let bounds = ball_bounds;

        if (bounds.y - pos.y - ball_radius) < 0.0 {
            pos.y = bounds.y - ball_radius;
            ball.velocity.y = -ball.velocity.y;
            ball.velocity.y *= ball_etransfer;
            if ball.owner.is_none() {
                ball.bounced = true;
                audio.play_sound(*ball_bounced, ball_bounced.volume());
            }
        }
        if (-bounds.y - pos.y + ball_radius) > 0.0 {
            pos.y = -bounds.y + ball_radius;
            ball.velocity.y = -ball.velocity.y;
            ball.velocity.y *= ball_etransfer;
            ball.bounced = ball.owner.is_none();
            if ball.owner.is_none() {
                ball.bounced = true;
                audio.play_sound(*ball_bounced, ball_bounced.volume());
            }
        }
        if (bounds.x - pos.x - ball_radius) < 0.0 {
            pos.x = bounds.x - ball_radius;
            ball.velocity.x = -ball.velocity.x;
            ball.velocity.x *= ball_etransfer;
            ball.bounced = ball.owner.is_none();
            if ball.owner.is_none() {
                ball.bounced = true;
                audio.play_sound(*ball_bounced, ball_bounced.volume());
            }
        }
        if (-bounds.x - pos.x + ball_radius) > 0.0 {
            pos.x = -bounds.x + ball_radius;
            ball.velocity.x = -ball.velocity.x;
            ball.velocity.x *= ball_etransfer;
            ball.bounced = ball.owner.is_none();
            if ball.owner.is_none() {
                ball.bounced = true;
                audio.play_sound(*ball_bounced, ball_bounced.volume());
            }
        }

        // Drift to make sure the ball doesn't get stuck on the side.
        if pos.x + ball_radius + ball_radius > bounds.x
            || pos.x - ball_radius - ball_radius < -bounds.x
        {
            let signum = ball.velocity.x.signum();
            let abs = ball.velocity.x.abs();
            let mag = abs.max(ball_border_slide);
            let value = mag * signum;
            ball.velocity.x = value;
            paths.get_mut(ball_entity).unwrap().color = Color::ORANGE;
        } else {
            paths.get_mut(ball_entity).unwrap().color = Color::GREEN;
        }
    }
}
