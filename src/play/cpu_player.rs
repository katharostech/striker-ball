use super::*;

pub fn get_cpu_input(world: &World, slot: PlayerSlot) -> PlayInputDense {
    if let Some(ent_signs) = world.get_resource::<PlayerEntSigns>() {
        let entity = ent_signs.get(slot);
        world
            .component::<CpuPlayer>()
            .get(entity)
            .unwrap()
            .input
            .get_dense_input()
    } else {
        PlayInputDense::default()
    }
}

#[derive(HasSchema, Clone, Default)]
pub struct CpuPlayerState;

#[derive(HasSchema, Clone, Default)]
pub struct CpuPlayer {
    /// The entity containing the state component for this cpu.
    pub state_e: Entity,
    // NOTE: The inputs are used at the start of the frame,
    // but are decided on after. This means that the cpu's
    // decisions will be delayed by one frame. We don't care
    // to have the cpus that efficient though, we want them
    // to seem like humans.
    /// The desired input of the cpu player.
    pub input: PlayInput,
}

pub struct CpuPlayerPlugin;
impl SessionPlugin for CpuPlayerPlugin {
    fn install(self, session: &mut SessionBuilder) {
        session.add_system_to_stage(PreUpdate, apply_cpu_inputs);
    }
}
pub fn apply_cpu_inputs(world: &World, player_ent_signs: Res<PlayerEntSigns>) {
    for entity in player_ent_signs.entities() {
        world.run_system(apply_cpu_input, entity);
    }
}

pub fn apply_cpu_input(
    In(self_e): In<Entity>,
    entities: Res<Entities>,
    transforms: Comp<Transform>,
    balls: Comp<Ball>,
    pins: Comp<Pin>,
    teams: Comp<Team>,
    players: Comp<Player>,
    player_ent_signs: Res<PlayerEntSigns>,
    root: Root<Data>,
    mut cpu_players: CompMut<CpuPlayer>,
) {
    let Constants {
        player_bounds,
        player_radius,
        ball_radius,
        ..
    } = root.constant;

    let Some(cpu_player) = cpu_players.get_mut(self_e) else {
        return;
    };
    let input = &mut cpu_player.input;

    let get_pos = |entity: Entity| transforms.get(entity).unwrap().translation.xy();

    let partner_e = player_ent_signs.get_partner(self_e);
    let self_player = players.get(self_e).unwrap();
    let partner_player = players.get(partner_e).unwrap();
    let attacking_direction = self_player.id.team().attacking_direction();
    let defending_direction = -attacking_direction;

    let (ball_e, ball) = entities.get_single_with(&balls).unwrap();
    let ball_pos = get_pos(ball_e);

    let partner_pos = transforms.get(partner_e).unwrap().translation.xy();
    let self_pos = transforms.get(self_e).unwrap().translation.xy();

    let direction_of_ball = (ball_pos - self_pos).normalize_or_zero();
    let distance_to_ball = self_pos.distance(ball_pos);
    let direction_of_partner = (partner_pos - self_pos).normalize_or_zero();

    let closest_enemy_pos = {
        let mut closest_enemy_pos: Option<Vec2> = None;
        for enemy_e in player_ent_signs.get_enemies(self_player.id) {
            let new_pos = transforms.get(enemy_e).unwrap().translation.xy();
            if closest_enemy_pos
                .is_some_and(|pos| pos.distance(self_pos) > new_pos.distance(self_pos))
                || closest_enemy_pos.is_none()
            {
                closest_enemy_pos = Some(new_pos);
            }
        }
        closest_enemy_pos.unwrap()
    };

    let offensive_enemy_pos = {
        let mut offensive_enemy_pos: Option<Vec2> = None;
        for enemy_e in player_ent_signs.get_enemies(self_player.id) {
            let new_pos = transforms.get(enemy_e).unwrap().translation.xy();
            if offensive_enemy_pos.is_some_and(|pos| {
                if defending_direction.is_sign_negative() {
                    new_pos.x < pos.x
                } else {
                    new_pos.x > pos.x
                }
            }) || offensive_enemy_pos.is_none()
            {
                offensive_enemy_pos = Some(new_pos);
            }
        }
        offensive_enemy_pos.unwrap()
    };

    let direction_of_closest_enemy = (closest_enemy_pos - self_pos).normalize_or_zero();
    let distance_to_closest_enemy = closest_enemy_pos.distance(self_pos);

    let closest_enemy_pin_pos = {
        let mut closest_pin_pos: Option<Vec2> = None;
        for (_pin_e, (_pin, team, transform)) in entities.iter_with((&pins, &teams, &transforms)) {
            if *team == self_player.id.team() {
                continue;
            }
            let new_pos = transform.translation.xy();
            if closest_pin_pos
                .is_some_and(|pos| pos.distance(self_pos) > new_pos.distance(self_pos))
                || closest_pin_pos.is_none()
            {
                closest_pin_pos = Some(new_pos);
            }
        }
        closest_pin_pos.unwrap_or_default()
    };

    // self is closer or the primary player slot when equal
    let dibs_on = |target_pos: Vec2| {
        target_pos.distance(self_pos) < target_pos.distance(partner_pos)
            || target_pos.distance(self_pos) == target_pos.distance(partner_pos)
                && self_player.id.is_primary()
    };

    let tackle_distance = player_radius * 5.0;

    let partner_is_ahead = attacking_direction.is_sign_positive() && partner_pos.x > self_pos.x
        || attacking_direction.is_sign_negative() && partner_pos.x < self_pos.x;

    let partner_is_tackleable = 'pressured: {
        for enemy_e in player_ent_signs.get_enemies(partner_player.id) {
            if transforms
                .get(enemy_e)
                .unwrap()
                .translation
                .xy()
                .distance(partner_pos)
                < tackle_distance
            {
                break 'pressured true;
            }
        }
        false
    };
    let partner_is_pressured = 'pressured: {
        for enemy_e in player_ent_signs.get_enemies(partner_player.id) {
            if transforms
                .get(enemy_e)
                .unwrap()
                .translation
                .xy()
                .distance(partner_pos)
                < tackle_distance * 2.0
            {
                break 'pressured true;
            }
        }
        false
    };

    let flee_direction_y = {
        let flee_direction_y = -direction_of_closest_enemy.y.signum();
        let distance_to_edge = player_bounds.y - self_pos.y.abs();
        let wall_is_close = distance_to_edge <= player_radius * 3.0;
        let enemy_is_opposite = self_pos.y.is_sign_positive() && closest_enemy_pos.y <= self_pos.y
            || self_pos.y.is_sign_negative() && closest_enemy_pos.y >= self_pos.y;
        let already_escaping = input.y.signum() == -flee_direction_y;
        let closed_in = enemy_is_opposite && wall_is_close || already_escaping;
        if closed_in {
            -flee_direction_y
        } else {
            flee_direction_y
        }
    };

    let mut match_offensive_enemy_y = || {
        let offensive_enemy_y_distance = (offensive_enemy_pos.y - self_pos.y).abs();
        if offensive_enemy_pos.y.abs() > player_bounds.y * 0.75 {
            input.y = 0.0;
        } else if offensive_enemy_y_distance < player_radius * 4.0 {
            // input.y = 0.0; //
        } else {
            input.y = direction_of_closest_enemy.y.signum();
        }
    };

    if let Maybe::Set(owner_e) = ball.owner {
        let owner_pos = get_pos(owner_e);

        if owner_e == self_e {
            input.x = attacking_direction;

            if partner_is_ahead && !partner_is_tackleable {
                if input.pass.pressed() {
                    input.pass.apply_bool(false);
                } else {
                    input.pass.apply_bool(true);
                }
            } else if distance_to_closest_enemy < tackle_distance {
                let direction_to_pin = (closest_enemy_pin_pos - self_pos).normalize_or_zero();
                input.x = direction_to_pin.x;
                input.y = direction_to_pin.y;

                let near_target_angle = (self_player.angle.angle_between(Vec2::X)
                    - direction_to_pin.angle_between(Vec2::X))
                .abs()
                    < 3_f32.to_radians();

                if !input.shoot.pressed() && near_target_angle {
                    input.shoot.apply_bool(true);
                } else {
                    input.shoot.apply_bool(false);
                }
            } else {
                input.y = flee_direction_y;
            }
        } else if owner_e == partner_e {
            if partner_is_pressured {
                input.x = direction_of_partner.x.signum();
                match_offensive_enemy_y();
            } else {
                input.x = attacking_direction;
                input.y = -direction_of_partner.y.signum();
            }
        }
        // enemy owns the ball
        else {
            let dibs = dibs_on(owner_pos);

            if dibs {
                let direction_of_target = (owner_pos - self_pos).normalize_or_zero();
                input.x = direction_of_target.x;
                input.y = direction_of_target.y;

                let self_distance = self_pos.distance(owner_pos);
                if self_distance < tackle_distance {
                    if input.pass.pressed() {
                        input.pass.apply_bool(false);
                    } else {
                        input.pass.apply_bool(true);
                    }
                }
            } else {
                let distance_to_defense =
                    (player_bounds.x * defending_direction - self_pos.x).abs();

                if distance_to_defense < player_radius {
                    input.x = 0.0;
                } else {
                    input.x = defending_direction;
                }
                match_offensive_enemy_y();
            }
        }
    }
    // un-owned ball
    else {
        let dibs = dibs_on(ball_pos);

        if dibs {
            input.x = direction_of_ball.x;
            input.y = direction_of_ball.y;

            if distance_to_ball < tackle_distance
                && (ball_pos.x - self_pos.x).abs() < ball_radius / 2.0
            {
                if input.pass.pressed() {
                    input.pass.apply_bool(false);
                } else {
                    input.pass.apply_bool(true);
                }
            }
        } else {
            input.x = defending_direction;

            match_offensive_enemy_y();
        }
    }
}

pub mod state {
    crate::states![chase, drive, aim, fire, catch, save];
}

// pub struct CpuPlayerPlugin;
// impl SessionPlugin for CpuPlayerPlugin {
//     fn install(self, session: &mut SessionBuilder) {
//         session
//             .install_plugin(CpuStatePlugin::new(
//                 state::chase(),
//                 chase_transition,
//                 chase_update,
//             ))
//             .install_plugin(CpuStatePlugin::new(
//                 state::drive(),
//                 drive_transition,
//                 drive_update,
//             ))
//             .install_plugin(CpuStatePlugin::new(
//                 state::catch(),
//                 catch_transition,
//                 catch_update,
//             ))
//             .install_plugin(CpuStatePlugin::new(
//                 state::save(),
//                 save_transition,
//                 save_update,
//             ))
//             .install_plugin(CpuStatePlugin::new(
//                 state::aim(),
//                 aim_transition,
//                 aim_update,
//             ))
//             .install_plugin(CpuStatePlugin::new(
//                 state::fire(),
//                 fire_transition,
//                 fire_update,
//             ));
//     }
// }

// //
// // Reused
// fn to_drive_transition(
//     In(self_e): In<Entity>,
//     cpu_players: Comp<CpuPlayer>,
//     balls: Comp<Ball>,
//     mut states: CompMut<State>,
// ) {
//     let cpu_player = cpu_players.get(self_e).unwrap();
//     let cpu_state = states.get_mut(cpu_player.state_e).unwrap();
//     let ball = balls.iter().next().unwrap();
//     if ball.owner.option().is_some_and(|owner_e| owner_e == self_e) {
//         cpu_state.current = state::drive();
//     }
// }
// fn to_catch_transition(
//     In(self_e): In<Entity>,
//     cpu_players: Comp<CpuPlayer>,
//     balls: Comp<Ball>,
//     player_ent_signs: Res<PlayerEntSigns>,
//     mut states: CompMut<State>,
// ) {
//     let cpu_player = cpu_players.get(self_e).unwrap();
//     let cpu_state = states.get_mut(cpu_player.state_e).unwrap();
//     let partner_e = player_ent_signs.get_partner(self_e);
//     let ball = balls.iter().next().unwrap();
//     if ball
//         .owner
//         .option()
//         .is_some_and(|owner_e| owner_e == partner_e)
//     {
//         cpu_state.current = state::catch();
//     }
// }
// fn to_chase_transition(
//     In(self_e): In<Entity>,
//     cpu_players: Comp<CpuPlayer>,
//     balls: Comp<Ball>,
//     player_ent_signs: Res<PlayerEntSigns>,
//     mut states: CompMut<State>,
// ) {
//     let cpu_player = cpu_players.get(self_e).unwrap();
//     let cpu_state = states.get_mut(cpu_player.state_e).unwrap();
//     let partner_e = player_ent_signs.get_partner(self_e);
//     let ball = balls.iter().next().unwrap();
//     if ball
//         .owner
//         .option()
//         .is_some_and(|owner_e| owner_e != self_e && owner_e != partner_e)
//     {
//         cpu_state.current = state::chase();
//     }
// }
// fn run_to_ball_update(
//     In(self_e): In<Entity>,
//     entities: Res<Entities>,
//     transforms: Comp<Transform>,
//     balls: Comp<Ball>,
//     mut cpu_players: CompMut<CpuPlayer>,
// ) {
//     let cpu_player = cpu_players.get_mut(self_e).unwrap();
//     let input = &mut cpu_player.input;

//     let (ball_e, ball) = entities.get_single_with(&balls).unwrap();

//     let get_pos = |entity: Entity| transforms.get(entity).unwrap().translation.xy();
//     let self_pos = get_pos(self_e);

//     let target_pos = if let Some(owner_e) = ball.owner.option() {
//         get_pos(owner_e)
//     } else {
//         get_pos(ball_e)
//     };

//     let direction_of_target = (target_pos - self_pos).normalize_or_zero();
//     input.x = direction_of_target.x;
//     input.y = direction_of_target.y;
// }
// fn tackle_within_distance_update(
//     In(self_e): In<Entity>,
//     entities: Res<Entities>,
//     transforms: Comp<Transform>,
//     balls: Comp<Ball>,
//     states: Comp<State>,
//     root: Root<Data>,
//     mut cpu_players: CompMut<CpuPlayer>,
// ) {
//     // let Constants { player_radius, .. } = root.constant;

//     // let cpu_player = cpu_players.get_mut(self_e).unwrap();

//     // let input = &mut cpu_player.input;

//     // let (ball_e, ball) = entities.get_single_with(&balls).unwrap();

//     // let get_pos = |entity: Entity| transforms.get(entity).unwrap().translation.xy();

//     // let self_state = states.get(self_e).unwrap();
//     // let self_pos = get_pos(self_e);

//     // let target_pos = if let Some(owner_e) = ball.owner.option() {
//     //     get_pos(owner_e)
//     // } else {
//     //     get_pos(ball_e)
//     // };

//     // let distance_to_target = target_pos.distance(self_pos);
//     // let tackling_distance = player_radius * 6.0;
//     // let chase_player = ball.owner.option().is_some_and(|e| e != self_e);
//     // let in_tackling_distance = distance_to_target <= tackling_distance;
//     // let ready_to_tackle = self_state.current != player::state::tackle();

//     // if chase_player && in_tackling_distance && ready_to_tackle {
//     //     input.pass.toggle();
//     // } else {
//     //     input.pass.release();
//     // }
// }

// // Chase
// //
// fn chase_transition(In(player_e): In<Entity>, world: &World) {
//     world.run_system(to_drive_transition, player_e);
//     world.run_system(to_catch_transition, player_e);
// }
// fn chase_update(In(player_e): In<Entity>, world: &World) {
//     world.run_system(run_to_ball_update, player_e);
//     world.run_system(tackle_within_distance_update, player_e);
// }

// // Drive
// //
// fn drive_transition(In(player_e): In<Entity>, world: &World) {
//     world.run_system(to_aim_transition, player_e);
//     world.run_system(to_chase_transition, player_e);
//     world.run_system(to_catch_transition, player_e);
// }
// fn drive_update(In(player_e): In<Entity>, world: &World) {
//     world.run_system(drive_update_impl, player_e);
// }
// fn drive_update_impl(
//     In(self_e): In<Entity>,
//     players: Comp<Player>,
//     transforms: Comp<Transform>,
//     mut cpu_players: CompMut<CpuPlayer>,
// ) {
//     let cpu_player = cpu_players.get_mut(self_e).unwrap();
//     let input = &mut cpu_player.input;

//     let self_player = players.get(self_e).unwrap();

//     let attacking_direction = self_player.team().attacking_direction();
//     input.x = attacking_direction;
//     input.y = 0.0;
// }
// fn to_aim_transition(
//     In(self_e): In<Entity>,
//     players: Comp<Player>,
//     entities: Res<Entities>,
//     transforms: Comp<Transform>,
//     pins: Comp<Pin>,
//     teams: Comp<Team>,
//     root: Root<Data>,
//     mut cpu_players: CompMut<CpuPlayer>,
// ) {
//     let Constants {
//         player_radius,
//         player_bounds,
//         ..
//     } = root.constant;

//     let cpu_player = cpu_players.get_mut(self_e).unwrap();
//     let input = &mut cpu_player.input;

//     let self_player = players.get(self_e).unwrap();

//     let get_pos = |entity: Entity| transforms.get(entity).unwrap().translation.xy();
//     let self_pos = get_pos(self_e);

//     let shooting_distance = player_radius * 10.0;
//     let distance_to_border = player_bounds.x - self_pos.x.abs();

//     if distance_to_border <= shooting_distance {
//         let mut furthest_pin_pos: Option<Vec2> = None;

//         for (pin_e, (_pin, pin_team)) in entities.iter_with((&pins, &teams)) {
//             let pin_pos = get_pos(pin_e);
//             if *pin_team != self_player.team()
//                 && (furthest_pin_pos.is_none()
//                     || furthest_pin_pos
//                         .is_some_and(|p| p.distance(self_pos) < pin_pos.distance(self_pos)))
//             {
//                 furthest_pin_pos = Some(pin_pos);
//             }
//         }
//         let Some(furthest_pin_pos) = furthest_pin_pos else {
//             return;
//         };
//         let direction_to_furthest_pin = (furthest_pin_pos - self_pos).normalize_or_zero();

//         input.x = direction_to_furthest_pin.x;
//         input.y = direction_to_furthest_pin.y;
//     }
// }

// //
// // Aim

// fn aim_transition(In(player_e): In<Entity>, world: &World) {
//     world.run_system(to_fire_transition, player_e);
//     world.run_system(to_chase_transition, player_e);
//     world.run_system(to_catch_transition, player_e);
// }
// fn aim_update(In(player_e): In<Entity>, world: &World) {
//     world.run_system(aim_update_impl, player_e);
// }
// fn aim_update_impl(
//     In(self_e): In<Entity>,
//     players: Comp<Player>,
//     entities: Res<Entities>,
//     transforms: Comp<Transform>,
//     pins: Comp<Pin>,
//     teams: Comp<Team>,
//     root: Root<Data>,
//     mut cpu_players: CompMut<CpuPlayer>,
// ) {
//     let Constants {
//         player_radius,
//         player_bounds,
//         ..
//     } = root.constant;

//     let cpu_player = cpu_players.get_mut(self_e).unwrap();
//     let input = &mut cpu_player.input;

//     let self_player = players.get(self_e).unwrap();

//     let get_pos = |entity: Entity| transforms.get(entity).unwrap().translation.xy();
//     let self_pos = get_pos(self_e);

//     let attacking_direction = self_player.team().attacking_direction();
//     input.x = attacking_direction;
//     input.y = 0.0;

//     let shooting_distance = player_radius * 10.0;
//     let distance_to_border = player_bounds.x - self_pos.x.abs();

//     if distance_to_border <= shooting_distance {
//         let mut furthest_pin_pos: Option<Vec2> = None;

//         for (pin_e, (_pin, pin_team)) in entities.iter_with((&pins, &teams)) {
//             let pin_pos = get_pos(pin_e);
//             if *pin_team != self_player.team()
//                 && (furthest_pin_pos.is_none()
//                     || furthest_pin_pos
//                         .is_some_and(|p| p.distance(self_pos) < pin_pos.distance(self_pos)))
//             {
//                 furthest_pin_pos = Some(pin_pos);
//             }
//         }
//         let Some(furthest_pin_pos) = furthest_pin_pos else {
//             return;
//         };
//         let direction_to_furthest_pin = (furthest_pin_pos - self_pos).normalize_or_zero();

//         input.x = direction_to_furthest_pin.x;
//         input.y = direction_to_furthest_pin.y;
//     }
// }
// fn to_fire_transition(
//     In(self_e): In<Entity>,
//     cpu_players: Comp<CpuPlayer>,
//     mut states: CompMut<State>,
// ) {
//     let cpu_player = cpu_players.get(self_e).unwrap();
//     let self_state = states.get(self_e).unwrap();
//     let shooting = self_state.current == player::state::shoot();
//     let cpu_state = states.get_mut(cpu_player.state_e).unwrap();
//     if shooting {
//         cpu_state.current = state::fire();
//     }
// }

// //
// // Fire
// fn fire_transition(In(player_e): In<Entity>, world: &World) {
//     world.run_system(to_chase_transition, player_e);
//     world.run_system(to_catch_transition, player_e);
// }
// fn fire_update(In(player_e): In<Entity>, world: &World) {
//     world.run_system(fire_update_impl, player_e);
// }
// fn fire_update_impl(
//     In(self_e): In<Entity>,
//     players: Comp<Player>,
//     entities: Res<Entities>,
//     pins: Comp<Pin>,
//     transforms: Comp<Transform>,
//     mut cpu_players: CompMut<CpuPlayer>,
// ) {
//     let cpu_player = cpu_players.get_mut(self_e).unwrap();
//     let input = &mut cpu_player.input;

//     let self_player = players.get(self_e).unwrap();

//     let get_pos = |entity: Entity| transforms.get(entity).unwrap().translation.xy();
//     let self_pos = get_pos(self_e);

//     let mut furthest_pin_pos: Option<Vec2> = None;

//     for (pin_e, _pin) in entities.iter_with(&pins) {
//         let pin_pos = get_pos(pin_e);
//         if furthest_pin_pos.is_none()
//             || furthest_pin_pos.is_some_and(|p| p.distance(self_pos) < pin_pos.distance(self_pos))
//         {
//             furthest_pin_pos = Some(pin_pos);
//         }
//     }
//     let Some(furthest_pin_pos) = furthest_pin_pos else {
//         return;
//     };
//     let direction_to_furthest_pin = (furthest_pin_pos - self_pos).normalize_or_zero();

//     let range = (player::SPREAD - 0.01).to_radians();
//     let diff = self_player.action_angle.angle_between(self_player.angle);
//     let shoot_angle_max_reached = diff.is_sign_positive() && diff.abs() >= range
//         || diff.is_sign_negative() && diff.abs() >= range;
//     let shoot_angle_achieved = self_player.angle.normalize_or_zero() == direction_to_furthest_pin;

//     if shoot_angle_max_reached || shoot_angle_achieved {
//         input.shoot.release();
//     }
// }

// // Catch
// //
// fn catch_transition(In(player_e): In<Entity>, world: &World) {
//     world.run_system(catch_out_transition, player_e);
//     world.run_system(to_chase_transition, player_e);
//     world.run_system(to_drive_transition, player_e);
// }
// fn catch_update(In(player_e): In<Entity>, world: &World) {
//     world.run_system(catch_update_impl, player_e);
// }
// fn catch_out_transition(
//     In(self_e): In<Entity>,
//     player_ent_signs: Res<PlayerEntSigns>,
//     transforms: Comp<Transform>,
//     root: Root<Data>,
//     mut cpu_players: CompMut<CpuPlayer>,
//     mut states: CompMut<State>,
// ) {
//     let Constants { player_radius, .. } = root.constant;

//     let cpu_player = cpu_players.get_mut(self_e).unwrap();
//     let cpu_state = states.get_mut(cpu_player.state_e).unwrap();

//     let partner_e = player_ent_signs.get_partner(self_e);

//     let get_pos = |entity: Entity| transforms.get(entity).unwrap().translation.xy();

//     let [enemy_a, enemy_b] = player_ent_signs.get_enemies_with_e(self_e);
//     let enemy_a_pos = get_pos(enemy_a);
//     let enemy_b_pos = get_pos(enemy_b);
//     let partner_pos = get_pos(partner_e);

//     let partner_pressure_a = enemy_a_pos.distance(partner_pos);
//     let partner_pressure_b = enemy_b_pos.distance(partner_pos);

//     let pressured_distance = player_radius * 6.0;
//     let pressured =
//         partner_pressure_a >= pressured_distance || partner_pressure_b >= pressured_distance;

//     if pressured {
//         cpu_state.current = state::save();
//     }
// }
// fn catch_update_impl(
//     In(self_e): In<Entity>,
//     players: Comp<Player>,
//     mut cpu_players: CompMut<CpuPlayer>,
// ) {
//     let cpu_player = cpu_players.get_mut(self_e).unwrap();
//     let input = &mut cpu_player.input;

//     let self_player = players.get(self_e).unwrap();

//     let attacking_direction = self_player.team().attacking_direction();
//     input.x = attacking_direction;
//     input.y = 0.0;
// }

// //
// // Save

// fn save_transition(In(player_e): In<Entity>, world: &World) {
//     world.run_system(to_chase_transition, player_e);
//     world.run_system(to_drive_transition, player_e);
// }
// fn save_update(In(player_e): In<Entity>, world: &World) {
//     world.run_system(run_to_ball_update, player_e);
// }

// pub struct CpuStatePlugin {
//     pub state: Ustr,
//     pub transition: fn(In<Entity>, &World),
//     pub update: fn(In<Entity>, &World),
// }
// impl CpuStatePlugin {
//     pub fn new(
//         state: Ustr,
//         transition: fn(In<Entity>, &World),
//         update: fn(In<Entity>, &World),
//     ) -> Self {
//         Self {
//             state,
//             transition,
//             update,
//         }
//     }
// }
// impl SessionPlugin for CpuStatePlugin {
//     fn install(self, session: &mut SessionBuilder) {
//         session
//             .add_system_to_stage(First, cpu_player_refresh_tick)
//             .add_system_to_stage(
//                 StateStage,
//                 cpu_state_transition(self.state, self.transition),
//             )
//             .add_system_to_stage(PreUpdate, cpu_state_update(self.state, self.update));
//     }
// }

// pub fn cpu_state_transition(
//     state_id: Ustr,
//     transition: fn(In<Entity>, &World),
// ) -> StaticSystem<(), ()> {
//     (move |world: &World| {
//         for player_e in world.resource::<PlayerEntSigns>().entities() {
//             if world
//                 .component::<CpuPlayer>()
//                 .get(player_e)
//                 .is_some_and(|cpu| {
//                     world.component::<State>().get(cpu.state_e).unwrap().current == state_id
//                 })
//             {
//                 world.run_system(transition, player_e);
//             }
//         }
//     })
//     .system()
// }

// pub fn cpu_state_update(state_id: Ustr, update: fn(In<Entity>, &World)) -> StaticSystem<(), ()> {
//     (move |world: &World| {
//         for player_e in world.resource::<PlayerEntSigns>().entities() {
//             if world
//                 .component::<CpuPlayer>()
//                 .get(player_e)
//                 .is_some_and(|cpu| {
//                     world.component::<State>().get(cpu.state_e).unwrap().current == state_id
//                 })
//             {
//                 world.run_system(update, player_e);
//             }
//         }
//     })
//     .system()
// }
