use super::*;

pub fn apply_cpu_input(world: &World, slot: PlayerSlot, input: &mut PlayInput) {
    let Some(ent_signs) = world.get_resource::<PlayerEntSigns>() else {
        tracing::warn!("PlayerEntSigns doesn't exist; This is alright before the scene fully spawned but shouldn't happen during the game");
        return;
    };
    let entity = ent_signs.get(slot);
    world.run_system(apply_cpu_input_system, (entity, input));
}

pub fn apply_cpu_input_system(
    In((self_e, input)): In<(Entity, &mut PlayInput)>,
    entities: Res<Entities>,
    transforms: Comp<Transform>,
    balls: Comp<Ball>,
    pins: Comp<Pin>,
    teams: Comp<Team>,
    players: Comp<Player>,
    player_ent_signs: Res<PlayerEntSigns>,
    root: Root<Data>,
) {
    let Constants {
        player_bounds,
        player_radius,
        ball_radius,
        ..
    } = root.constant;

    let get_pos = |entity: Entity| transforms.get(entity).unwrap().translation.xy();

    let partner_e = player_ent_signs.get_partner(self_e);
    let self_player = players.get(self_e).unwrap();
    let partner_player = players.get(partner_e).unwrap();
    let attacking_direction = self_player.id.team().direction(); // TODO: add attacking_direction & defending_direction methods to `Team`
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

                if !input.shoot.pressed()
                    && self_player.angle.angle_between(Vec2::X)
                        == direction_to_pin.angle_between(Vec2::X)
                {
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
