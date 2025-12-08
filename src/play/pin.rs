use super::*;

pub mod prelude {
    pub use super::{Pin, PinPlugin, PinScore};
}

#[derive(HasSchema, Clone, Default)]
pub struct Pin;

#[derive(HasSchema, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PinScore {
    pub a: u8,
    pub b: u8,
}
impl PinScore {
    pub fn inc_a(&mut self) {
        self.a += 1;
    }
    pub fn inc_b(&mut self) {
        self.b += 1;
    }
}

pub struct PinPlugin;
impl SessionPlugin for PinPlugin {
    fn install(self, session: &mut SessionBuilder) {
        session.insert_resource(PinScore::default());
        session.add_system_to_stage(Update, update);
    }
}

pub fn animation_bank() -> AnimationBankSprite {
    AnimationBankSprite {
        current: ustr("still"),
        animations: vec![
            (
                ustr("still"),
                AnimatedSprite {
                    index: 5,
                    frames: vec![].into(),
                    fps: 0.,
                    ..Default::default()
                },
            ),
            (
                ustr("explode"),
                AnimatedSprite {
                    index: 0,
                    frames: vec![0, 1, 2, 3, 4, 5].into(),
                    fps: 16.,
                    repeat: false,
                    ..Default::default()
                },
            ),
        ]
        .into_iter()
        .collect(),
        last_animation: ustr("still"),
    }
}

pub fn update(
    pins: Comp<Pin>,
    teams: Comp<Team>,
    balls: Comp<Ball>,
    transforms: Comp<Transform>,
    entities: Res<Entities>,
    atlases: Comp<AtlasSprite>,
    root: Root<Data>,
    mut audio: ResMut<AudioCenter>,
    mut score: ResMut<PinScore>,
    mut banks: CompMut<AnimationBankSprite>,
    mut commands: Commands,
) {
    let Constants {
        ball_radius,
        pin_radius,
        ..
    } = root.constant;

    let Sounds { pin_explosion, .. } = root.sound;

    for (pin_e, (_pin, team)) in entities.iter_with((&pins, &teams)) {
        let bank = banks.get_mut(pin_e).unwrap();
        if bank.current == ustr("still") {
            let pin_pos = transforms.get(pin_e).unwrap().translation.xy();
            for (ball_e, ball) in entities.iter_with(&balls) {
                if ball.owner.is_none() {
                    let ball_pos = transforms.get(ball_e).unwrap().translation.xy();
                    if ball_pos.distance(pin_pos) <= ball_radius + pin_radius {
                        match team {
                            Team::A => score.inc_b(),
                            Team::B => score.inc_a(),
                        }
                        bank.set_current("explode");
                        audio.play_sound(*pin_explosion, pin_explosion.volume());
                    }
                }
            }
        } else if bank.current == ustr("explode") && atlases.get(pin_e).unwrap().index == 5 {
            commands.add(move |mut entities: ResMut<Entities>| entities.kill(pin_e));
        }
    }
}
