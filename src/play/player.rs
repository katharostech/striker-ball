use super::*;

pub mod prelude {
    pub use super::{
        AimArrow, AimCone, Player, PlayerShadowSprite, PlayerSlot, PlayerSprite, StickIndicator,
        Team,
    };
}
pub mod state {
    crate::states![
        free, tackle, tackled, grab, ball, shoot, pass, turn, recieve, kick, lose, win, wait
    ];
}

pub const SPREAD: f32 = 45.;

#[derive(HasSchema, Clone, Copy, Default, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Team {
    #[default]
    A,
    B,
}

#[derive(HasSchema, Clone, Default, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PlayerSlot {
    #[default]
    A1,
    A2,
    B1,
    B2,
}
impl PlayerSlot {
    /// The associated offline client index to this slot.
    pub fn index(&self) -> usize {
        match self {
            PlayerSlot::A1 => 0,
            PlayerSlot::A2 => 1,
            PlayerSlot::B1 => 2,
            PlayerSlot::B2 => 3,
        }
    }
    pub fn variants() -> [PlayerSlot; 4] {
        use PlayerSlot::*;
        [A1, A2, B1, B2]
    }
    pub fn partner(&self) -> PlayerSlot {
        use PlayerSlot::*;
        match self {
            A1 => A2,
            A2 => A1,
            B1 => B2,
            B2 => B1,
        }
    }
    pub fn team(&self) -> Team {
        match self {
            PlayerSlot::A1 | PlayerSlot::A2 => Team::A,
            PlayerSlot::B1 | PlayerSlot::B2 => Team::B,
        }
    }
    pub fn is_primary(&self) -> bool {
        matches!(self, Self::A1 | Self::B1)
    }
    pub fn is_secondary(&self) -> bool {
        matches!(self, Self::A2 | Self::B2)
    }
}

#[derive(HasSchema, Clone, Default)]
pub struct AimArrow(pub Entity);
#[derive(HasSchema, Clone, Default)]
pub struct AimCone(pub Entity);
#[derive(HasSchema, Clone, Default)]
pub struct StickIndicator;
#[derive(HasSchema, Clone, Default)]
pub struct PlayerSprite;
#[derive(HasSchema, Clone, Default)]
pub struct PlayerShadowSprite;

/// A few things that could be components are built into this struct
/// for convinience but could easily be extracted to remove the dependency
/// on its behaviors.
#[derive(HasSchema, Clone)]
#[repr(C)]
pub struct Player {
    pub angle: Vec2,
    /// Used for tracking the angle that the player tackled or is shooting from.
    pub action_angle: Vec2,
    // Used to tell which team the player is on and where the player needs to respawn.
    pub id: PlayerSlot,
    pub flip_x: bool,
    pub animation: Ustr,
}
impl Player {
    pub fn new(id: PlayerSlot) -> Self {
        Player {
            angle: match id.team() {
                Team::A => Vec2::X,
                Team::B => -Vec2::X,
            },
            action_angle: Vec2::X,
            id,
            flip_x: false,
            animation: ustr("idle"),
        }
    }
    pub fn team(&self) -> Team {
        self.id.team()
    }
}
impl Default for Player {
    fn default() -> Self {
        Self::new(PlayerSlot::A1)
    }
}
pub fn plugin(session: &mut SessionBuilder) {
    session
        .add_system_to_stage(StateStage, ball_transition)
        .add_system_to_stage(StateStage, free_transition)
        .add_system_to_stage(StateStage, recieve_transition)
        .add_system_to_stage(StateStage, shoot_transition)
        .add_system_to_stage(StateStage, turn_transition)
        .add_system_to_stage(
            StateStage,
            timed_transition::<Player>(state::kick(), state::free(), 30),
        )
        .add_system_to_stage(
            StateStage,
            timed_transition::<Player>(state::tackle(), state::free(), 30),
        )
        .add_system_to_stage(
            StateStage,
            timed_transition::<Player>(state::tackled(), state::free(), 30),
        )
        .add_system_to_stage(
            StateStage,
            timed_transition::<Player>(state::pass(), state::free(), 30),
        )
        .add_system_to_stage(
            StateStage,
            timed_transition::<Player>(state::recieve(), state::free(), 30),
        )
        .add_system_to_stage(PreUpdate, free_update)
        .add_system_to_stage(PreUpdate, ball_update)
        .add_system_to_stage(PreUpdate, shoot_update)
        .add_system_to_stage(PreUpdate, tackle_update)
        .add_system_to_stage(Update, aim_arrows_update)
        .add_system_to_stage(Update, aim_cones_update)
        .add_system_to_stage(PostUpdate, player_graphics)
        .add_system_to_stage(PostUpdate, hide_stick_indicators)
        .add_system_to_stage(PostUpdate, sync_sub_sprites);
}

//
// Transitions
//

// These are the core transition stages.
// Each one corresponds to one unique state and loops through the desired entities.
// It then runs systems on each individual entity that make **only** changes that
// are critical to state.

pub fn timed_transition<T>(from_id: Ustr, to_id: Ustr, duration: u64) -> StaticSystem<(), ()>
where
    T: HasSchema,
{
    (move |entities: Res<Entities>, markers: Comp<T>, mut states: CompMut<State>| {
        for (_e, (_marker, state)) in entities.iter_with((&markers, &mut states)) {
            if state.current == from_id && state.age() >= duration {
                state.current = to_id;
            }
        }
    })
    .system()
}
fn free_transition(world: &World) {
    for player_e in world.resource::<PlayerEntSigns>().entities() {
        if world.component::<State>().get(player_e).unwrap().current == state::free() {
            world.run_system(to_tackle_transition, player_e);
            world.run_system(to_ball_transition, player_e);
        }
    }
}
fn recieve_transition(world: &World) {
    for player_e in world.resource::<PlayerEntSigns>().entities() {
        if world.component::<State>().get(player_e).unwrap().current == state::recieve() {
            world.run_system(to_ball_transition, player_e);
        }
    }
}
fn ball_transition(world: &World) {
    for player_e in world.resource::<PlayerEntSigns>().entities() {
        if world.component::<State>().get(player_e).unwrap().current == state::ball() {
            world.run_system(ball_out_transition, player_e);
            world.run_system(to_tackled_transition, player_e);
        }
    }
}
fn shoot_transition(world: &World) {
    for player_e in world.resource::<PlayerEntSigns>().entities() {
        if world.component::<State>().get(player_e).unwrap().current == state::shoot() {
            world.run_system(shoot_out_transition, player_e);
            world.run_system(to_tackled_transition, player_e);
        }
    }
}
fn turn_transition(world: &World) {
    for player_e in world.resource::<PlayerEntSigns>().entities() {
        if world.component::<State>().get(player_e).unwrap().current == state::turn() {
            world.run_system(turn_out_transition, player_e);
        }
    }
}
fn turn_out_transition(
    In(player_e): In<Entity>,
    entities: Res<Entities>,
    root: Root<Data>,
    players: Comp<Player>,
    mut audio: ResMut<AudioCenter>,
    mut balls: CompMut<Ball>,
    mut states: CompMut<State>,
) {
    let state = states.get_mut(player_e).unwrap();

    if state.age() >= root.constant.turn_frames {
        state.current = state::kick();

        let (_ball_e, ball) = entities.get_single_with(&mut balls).unwrap();
        let player = players.get(player_e).unwrap();

        // TODO: Add warn if the dribble_pos didn't get to the target position by now.

        ball.owner = Maybe::Unset;
        ball.velocity = player.angle * root.constant.kick_power;
        let Sounds { ball_kicked, .. } = root.sound;
        audio.play_sound(*ball_kicked, ball_kicked.volume());
    }
}

fn to_ball_transition(
    In(player_e): In<Entity>,
    entities: Res<Entities>,
    transforms: Comp<Transform>,
    root: Root<Data>,
    mut players: CompMut<Player>,
    mut states: CompMut<State>,
    mut balls: CompMut<Ball>,
) {
    let player = players.get_mut(player_e).unwrap();
    let state = states.get_mut(player_e).unwrap();
    let transform = transforms.get(player_e).unwrap();
    let player_pos = transform.translation.xy();
    let Constants {
        player_radius,
        ball_radius,
        ..
    } = root.constant;

    let (ball_e, ball) = entities.get_single_with(&mut balls).unwrap();
    let ball_pos = transforms.get(ball_e).unwrap().translation.xy();

    if ball.owner.is_none() && ball_pos.distance(player_pos) <= player_radius + ball_radius
        // in case they gain the ball by other means we want to switch to dribbling
        || ball.owner.option().is_some_and(|target| target == player_e)
    {
        player.angle = (ball_pos - player_pos).normalize_or_zero();

        ball.velocity = default();
        ball.owner = Maybe::Set(player_e);
        ball.dribble_pos = player.angle * player_radius;
        state.current = state::ball();
    }
}

fn ball_out_transition(
    In(player_e): In<Entity>,
    inputs: Res<PlayInputs>,
    player_ent_signs: Res<PlayerEntSigns>,
    clients: Comp<Client>,
    transforms: CompMut<Transform>,
    mut players: CompMut<Player>,
    mut states: CompMut<State>,
) {
    let client = clients.get(player_e).unwrap();
    let control = inputs.get_control(client.index);

    // pass
    if control.pass.just_pressed() {
        let partner_e = player_ent_signs.partner(player_e);
        let partner_state = states.get_mut(partner_e).unwrap();

        if partner_state.current == state::free() {
            partner_state.current = state::recieve();

            states.get_mut(player_e).unwrap().current = state::turn();

            let start = transforms.get(player_e).unwrap().translation.xy();
            let end = transforms.get(partner_e).unwrap().translation.xy();
            let direction = (end - start).normalize();

            players.get_mut(player_e).unwrap().angle = direction;
            players.get_mut(partner_e).unwrap().angle = -direction;
        }
    }
    // shoot
    if control.shoot.just_pressed() {
        states.get_mut(player_e).unwrap().current = state::shoot();

        let player = players.get_mut(player_e).unwrap();

        let target = match player.team() {
            Team::A => Vec2::X,
            Team::B => Vec2::NEG_X,
        };
        if target.angle_between(player.angle) > 135_f32.to_radians()
            || target.angle_between(player.angle) < -135_f32.to_radians()
        {
            player.angle = target;
            player.action_angle = target;
        } else if target.angle_between(player.angle) > SPREAD.to_radians() {
            player.action_angle = target.rotate(Vec2::from_angle(SPREAD.to_radians()));
        } else if target.angle_between(player.angle) < -SPREAD.to_radians() {
            player.action_angle = target.rotate(Vec2::from_angle(-SPREAD.to_radians()));
        } else {
            player.action_angle = player.angle;
        }
    }
}

fn to_tackled_transition(
    In(player_e): In<Entity>,
    entities: Res<Entities>,
    player_ent_signs: Res<PlayerEntSigns>,
    transforms: Comp<Transform>,
    players: Comp<Player>,
    root: Root<Data>,
    mut audio: ResMut<AudioCenter>,
    mut balls: CompMut<Ball>,
    mut states: CompMut<State>,
) {
    let (_ball_e, ball) = entities.get_single_with(&mut balls).unwrap();
    let player = players.get(player_e).unwrap();
    let pos = transforms.get(player_e).unwrap().translation.xy();
    let Sounds { player_tackled, .. } = root.sound;

    for tackler_e in player_ent_signs.entities() {
        let tackler = players.get(tackler_e).unwrap();
        let tackler_pos = transforms.get(tackler_e).unwrap().translation.xy();
        let tackler_state = states.get(tackler_e).unwrap().current;

        if tackler_state == state::tackle()
            && tackler.team() != player.team()
            && tackler_pos.distance(pos) <= root.constant.player_radius * 2.
        {
            states.get_mut(player_e).unwrap().current = state::tackled();

            audio.play_sound(*player_tackled, player_tackled.volume());

            if let Maybe::Set(target) = &mut ball.owner {
                if *target == player_e {
                    *target = tackler_e;
                }
            }
        }
    }
}

fn to_tackle_transition(
    In(player_e): In<Entity>,
    inputs: Res<PlayInputs>,
    clients: Comp<Client>,
    root: Root<Data>,
    mut audio: ResMut<AudioCenter>,
    mut players: CompMut<Player>,
    mut states: CompMut<State>,
) {
    let player = players.get_mut(player_e).unwrap();
    let state = states.get_mut(player_e).unwrap();
    let client = clients.get(player_e).unwrap();
    let control = inputs.get_control(client.index);
    let Sounds { player_tackle, .. } = root.sound;

    if control.pass.just_pressed() {
        state.current = state::tackle();
        player.action_angle = player.angle;
        audio.play_sound(*player_tackle, player_tackle.volume());
    }
}

fn shoot_out_transition(
    In(player_e): In<Entity>,
    entities: Res<Entities>,
    inputs: Res<PlayInputs>,
    clients: Comp<Client>,
    root: Root<Data>,
    mut audio: ResMut<AudioCenter>,
    mut players: CompMut<Player>,
    mut states: CompMut<State>,
    mut balls: CompMut<Ball>,
) {
    let player = players.get_mut(player_e).unwrap();
    let state = states.get_mut(player_e).unwrap();
    let client = clients.get(player_e).unwrap();
    let control = inputs.get_control(client.index);

    if !control.shoot.pressed() {
        state.current = state::kick();

        let (_ball_e, ball) = entities.get_single_with(&mut balls).unwrap();
        if let Maybe::Set(target) = ball.owner {
            if target == player_e {
                ball.owner = Maybe::Unset;
                ball.velocity = player.angle * root.constant.kick_power;
            }
        }
        let Sounds { ball_kicked, .. } = root.sound;
        audio.play_sound(*ball_kicked, ball_kicked.volume());
    }
}

//
// Updates
//

// These are the update systems.
// Each one corresponds to one unique state and loops through the desired entities.
// It then runs updates that **do not** change the current state.
fn free_update(world: &World) {
    for player_e in world.resource::<PlayerEntSigns>().entities() {
        if world.component::<State>().get(player_e).unwrap().current == state::free() {
            world.run_system(walk, player_e);
        }
    }
}
fn ball_update(world: &World) {
    for player_e in world.resource::<PlayerEntSigns>().entities() {
        if world.component::<State>().get(player_e).unwrap().current == state::ball() {
            world.run_system(walk, player_e);
        }
    }
}
fn shoot_update(
    player_ent_signs: Res<PlayerEntSigns>,
    inputs: Res<PlayInputs>,
    clients: Comp<Client>,
    states: Comp<State>,
    mut players: CompMut<Player>,
) {
    for player_e in player_ent_signs.entities() {
        if states.get(player_e).unwrap().current != state::shoot() {
            continue;
        }
        let player = players.get_mut(player_e).unwrap();
        let client = clients.get(player_e).unwrap();
        let control = inputs.get_control(client.index);

        let direction = Vec2::new(control.x, control.y);

        if direction.length() > 0.2 {
            let direction = direction.normalize();
            let range_clock = Vec2::new(1.0, 0.05).normalize_or_zero();
            let range_count = Vec2::new(1.0, -0.05).normalize_or_zero();

            if direction.angle_between(player.angle) < range_clock.angle_between(Vec2::X) {
                player.angle = player.angle.rotate(range_clock)
            } else if direction.angle_between(player.angle) > range_count.angle_between(Vec2::X) {
                player.angle = player.angle.rotate(range_count)
            } else {
                player.angle = direction;
            }
        }

        let range = SPREAD.to_radians();
        let diff = player.action_angle.angle_between(player.angle);

        // Make sure the player can't point outside the aim cone.
        if diff.is_sign_positive() && diff.abs() > range {
            player.angle = player.action_angle.rotate(Vec2::from_angle(range));
        } else if diff.is_sign_negative() && diff.abs() > range {
            player.angle = player.action_angle.rotate(Vec2::from_angle(-range));
        }
    }
}
fn tackle_update(world: &World) {
    for player_e in world.resource::<PlayerEntSigns>().entities() {
        if world.component::<State>().get(player_e).unwrap().current != state::tackle() {
            continue;
        }
        {
            let asset_server = world.asset_server();
            let Constants {
                tackle_friction,
                tackle_speed,
                player_radius,
                ball_radius,
                ..
            } = asset_server.root::<Data>().constant;

            let entities = world.resource::<Entities>();
            let states = world.component::<State>();

            let mut players = world.component_mut::<Player>();
            let mut transforms = world.component_mut::<Transform>();

            let mut balls = world.component_mut::<Ball>();

            let player = players.get_mut(player_e).unwrap();
            let state = states.get(player_e).unwrap();
            let force = (tackle_speed - state.age() as f32 * tackle_friction).max(0.0);
            let movement = player.action_angle * force;
            {
                let transform = transforms.get_mut(player_e).unwrap();
                transform.translation.x += movement.x;
                transform.translation.y += movement.y;

                let player_pos = transform.translation.xy();

                let (ball_e, ball) = entities.get_single_with(&mut balls).unwrap();
                let ball_pos = transforms.get(ball_e).unwrap().translation.xy();

                if ball.owner.is_none()
                    && ball_pos.distance(player_pos) <= player_radius + ball_radius
                    || ball.owner.option().is_some_and(|target| target == player_e)
                {
                    // Don't point at the ball like normal.
                    // player.angle = (ball_pos - player_pos).normalize_or_zero();

                    ball.velocity = default();
                    ball.owner = Maybe::Set(player_e);
                    ball.dribble_pos = player.angle * player_radius;
                }
            }
        }
        world.run_system(update_bounds_collisions, player_e);
        world.run_system(update_player_collisions, player_e);
    }
}

fn walk(In(player_e): In<Entity>, world: &World) {
    {
        let asset_server = world.asset_server();
        let root = asset_server.root::<Data>();
        let inputs = world.resource_mut::<PlayInputs>();
        let clients = world.component_mut::<Client>();
        let mut players = world.component_mut::<Player>();
        let mut states = world.component_mut::<State>();
        let mut transforms = world.component_mut::<Transform>();

        let player = players.get_mut(player_e).unwrap();
        let state = states.get_mut(player_e).unwrap();
        let transform = transforms.get_mut(player_e).unwrap();
        let client = clients.get(player_e).unwrap();

        let speed = match state.current {
            id if id == state::free() => root.constant.run_speed,
            id if id == state::ball() => root.constant.dribble_speed,
            _ => return,
        };
        let control = inputs.get_control(client.index);
        let direction = Vec2::new(control.x, control.y);

        if direction.length() > 0.2 {
            player.animation = ustr("walk");
            player.angle = direction.normalize_or_zero();

            transform.translation.x += player.angle.x * speed;
            transform.translation.y += player.angle.y * speed;
        } else {
            player.animation = ustr("idle");
        }
    }
    world.run_system(update_bounds_collisions, player_e);
    world.run_system(update_player_collisions, player_e);
}

fn update_player_collisions(
    In(player_e): In<Entity>,
    player_ent_signs: Res<PlayerEntSigns>,
    root: Root<Data>,
    mut transforms: CompMut<Transform>,
) {
    let player_positions: [(Entity, Vec2); 4] = player_ent_signs
        .entities()
        .map(|entity| (entity, transforms.get(entity).unwrap().translation.xy()));

    let transform = transforms.get_mut(player_e).unwrap();

    for (entity, pos) in player_positions {
        // filter out the player's own position
        if entity == player_e {
            continue;
        }
        // collision detection
        let touch = root.constant.player_radius * 2.;
        let distance = transform.translation.xy().distance(pos);
        if distance < touch {
            let overlap = touch - distance;
            let angle = (transform.translation.xy() - pos).normalize_or_zero();
            let net = angle * overlap;
            transform.translation.x += net.x;
            transform.translation.y += net.y;
        }
    }
}

fn update_bounds_collisions(
    In(player_e): In<Entity>,
    root: Root<Data>,
    mut transforms: CompMut<Transform>,
) {
    let transform = transforms.get_mut(player_e).unwrap();

    let Constants {
        player_bounds,
        player_radius,
        ..
    } = root.constant;

    let bounds = player_bounds;
    let x_padding = player_radius;

    if bounds.x - x_padding < transform.translation.x {
        transform.translation.x = bounds.x - x_padding
    }
    if -bounds.x + x_padding > transform.translation.x {
        transform.translation.x = -bounds.x + x_padding
    }
    let y_padding = player_radius;

    if bounds.y - y_padding < transform.translation.y {
        transform.translation.y = bounds.y - y_padding
    }
    if -bounds.y + y_padding > transform.translation.y {
        transform.translation.y = -bounds.y + y_padding
    }
}

//
// Graphics Updates
//
fn aim_arrows_update(
    entities: Res<Entities>,
    aim_cones: Comp<AimArrow>,
    players: Comp<Player>,
    states: Comp<State>,
    mut sprites: CompMut<Sprite>,
    mut transforms: CompMut<Transform>,
) {
    for (entity, (AimArrow(player_e), sprite)) in entities.iter_with((&aim_cones, &mut sprites)) {
        let state = states.get(*player_e).unwrap();

        if state.current == state::shoot() {
            let player = players.get(*player_e).unwrap();
            let target = *transforms.get(*player_e).unwrap();
            let aim_cone = transforms.get_mut(entity).unwrap();

            aim_cone.translation.x = target.translation.x;
            aim_cone.translation.y = target.translation.y;
            aim_cone.rotation = Quat::from_rotation_z(-player.angle.angle_between(Vec2::X));

            sprite.color = Color::WHITE
        } else {
            sprite.color = Color::NONE
        }
    }
}
fn aim_cones_update(
    entities: Res<Entities>,
    aim_cones: Comp<AimCone>,
    players: Comp<Player>,
    states: Comp<State>,
    mut sprites: CompMut<Sprite>,
    mut transforms: CompMut<Transform>,
) {
    for (entity, (AimCone(player_e), sprite)) in entities.iter_with((&aim_cones, &mut sprites)) {
        let state = states.get(*player_e).unwrap();

        if state.current == state::shoot() {
            let player = players.get(*player_e).unwrap();
            let target = *transforms.get(*player_e).unwrap();
            let aim_cone = transforms.get_mut(entity).unwrap();

            aim_cone.translation.x = target.translation.x;
            aim_cone.translation.y = target.translation.y;
            aim_cone.rotation = Quat::from_rotation_z(
                -player
                    .action_angle
                    .rotate(Vec2::from_angle(-45f32.to_radians()))
                    .angle_between(Vec2::X),
            );

            sprite.color = Color::WHITE
        } else {
            sprite.color = Color::NONE
        }
    }
}

/// To offset the sprites from the character transform, a sprite is attached to the transform
/// as a seperate entity and its animations are synced in this system.
fn sync_sub_sprites(
    entities: Res<Entities>,
    players: Comp<Player>,
    bindings: Comp<Follow>,
    player_sprites: Comp<PlayerSprite>,
    mut banks: CompMut<AnimationBankSprite>,
    mut atlases: CompMut<AtlasSprite>,
) {
    for (_entity, (_player_sprite, follow, atlas, bank)) in
        entities.iter_with((&player_sprites, &bindings, &mut atlases, &mut banks))
    {
        let player = players.get(follow.target()).unwrap();
        atlas.flip_x = player.flip_x;
        bank.set_current(player.animation);
    }
}

fn hide_stick_indicators(
    entities: Res<Entities>,
    states: Comp<State>,
    players: Comp<Player>,
    indicators: Comp<StickIndicator>,
    follows: Comp<Follow>,
    mut sprites: CompMut<Sprite>,
) {
    for (_e, (_i, follow, sprite)) in entities.iter_with((&indicators, &follows, &mut sprites)) {
        if !players.contains(follow.target()) {
            continue;
        }
        let Some(state) = states.get(follow.target()) else {
            continue;
        };
        let state = state.current;

        if state == state::shoot() || state == state::win() || state == state::lose() {
            sprite.color = Color::NONE;
        } else {
            sprite.color = Color::WHITE;
        }
    }
}

fn player_graphics(
    entities: Res<Entities>,
    states: Comp<State>,
    root: Root<Data>,
    mut path2ds: CompMut<Path2d>,
    mut players: CompMut<Player>,
) {
    for (_player_e, (player, path, state)) in
        entities.iter_with((&mut players, &mut path2ds, &states))
    {
        if player.angle.x > 0.0 {
            player.flip_x = false
        } else if player.angle.x < 0.0 {
            player.flip_x = true
        }
        *path.points.get_mut(10).unwrap() = player.angle * root.constant.player_radius;

        match state.current {
            s if s == state::free() => path.color = path2d::color::FREE,
            s if s == state::tackle() => {
                player.animation = ustr("tackle");
                path.color = path2d::color::SHOOT
            }
            s if s == state::tackled() => {
                player.animation = ustr("tackled");
                path.color = path2d::color::SHOOT
            }
            s if s == state::grab() => {
                player.animation = ustr("grab");
                path.color = path2d::color::KICK_GRAB
            }
            s if s == state::ball() => path.color = path2d::color::DRIBBLE,
            s if s == state::shoot() => {
                player.animation = ustr("shoot");
                path.color = path2d::color::SHOOT
            }
            s if s == state::pass() => {
                player.animation = ustr("kick");
                path.color = path2d::color::KICK_GRAB
            }
            s if s == state::kick() => {
                player.animation = ustr("kick");
                path.color = path2d::color::KICK_GRAB
            }
            s if s == state::recieve() => {
                player.animation = ustr("idle");
                path.color = path2d::color::KICK_GRAB
            }
            s if s == state::lose() => {
                player.animation = ustr("tackled");
                path.color = path2d::color::FREE
            }
            s if s == state::win() => {
                player.animation = ustr("winning");
                path.color = path2d::color::FREE
            }
            s if s == state::wait() => {
                player.animation = ustr("idle");
                path.color = path2d::color::FREE
            }
            _ => {}
        }
    }
}
