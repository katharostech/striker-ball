use super::*;

pub mod prelude {
    pub use super::new_player_transform;
    pub use super::pins as spawn_pins;
    pub use super::player as spawn_player;
    pub use super::scene as spawn_scene;
}

pub fn scene(world: &World) {
    let asset_server = world.asset_server();
    let root = asset_server.root::<Data>();
    let screen = root.court.size();

    // Camera
    world
        .spawn()
        .insert(Camera {
            size: CameraSize::FixedWidth(screen.x),
            ..Default::default()
        })
        .insert(Transform::from_z(layers::CAMERA));

    // Court
    world
        .spawn()
        .insert(path2d::bounds(&root))
        .insert(Path2dToggle::new(path2d::color::BOUNDS))
        .insert(Sprite {
            image: *root.court,
            ..Default::default()
        })
        .insert(Transform::from_z(layers::COURT));

    // Ball
    world
        .spawn()
        .insert(Ball {
            sound_timer: Timer::from_seconds(root.sound.ball_spin_buffer, TimerMode::Repeating),
            ..Default::default()
        })
        .insert(path2d::ball(&root))
        .insert(Path2dToggle::new(path2d::color::BALL))
        .insert(AtlasSprite::new(root.sprite.ball))
        .insert(ball::sprite())
        .insert(Transform::from_translation(Vec3::new(0., 0., layers::BALL)));

    // Players
    let ent_signs = match &*world.resource::<PlayMode>() {
        PlayMode::Online {
            socket,
            service_type,
        } => {
            let team = match socket.player_idx() {
                0 => Team::A,
                1 => Team::B,
                _ => panic!("index out of player count bounds"),
            };
            let is_team_a = team == Team::A;
            let is_team_b = team == Team::B;
            match service_type {
                ServiceType::OnePlayer(..) => PlayerEntSigns {
                    a1: self::player(world, is_team_a.then_some(0), is_team_a, PlayerSlot::A1),
                    a2: self::player(world, is_team_a.then_some(0), is_team_a, PlayerSlot::A2),
                    b1: self::player(world, is_team_b.then_some(0), is_team_b, PlayerSlot::B1),
                    b2: self::player(world, is_team_b.then_some(0), is_team_b, PlayerSlot::B2),
                },
                ServiceType::TwoPlayer(..) => PlayerEntSigns {
                    a1: self::player(world, is_team_a.then_some(0), false, PlayerSlot::A1),
                    a2: self::player(world, is_team_a.then_some(1), false, PlayerSlot::A2),
                    b1: self::player(world, is_team_b.then_some(0), false, PlayerSlot::B1),
                    b2: self::player(world, is_team_b.then_some(1), false, PlayerSlot::B2),
                },
            }
        }
        PlayMode::Offline(PlayersInfo { team_a, team_b }) => PlayerEntSigns {
            a1: self::player(
                world,
                team_a.primary().number.into(),
                team_a.primary().dual_stick,
                PlayerSlot::A1,
            ),
            a2: self::player(
                world,
                team_a.secondary().number.into(),
                team_a.secondary().dual_stick,
                PlayerSlot::A2,
            ),
            b1: self::player(
                world,
                team_b.primary().number.into(),
                team_b.primary().dual_stick,
                PlayerSlot::B1,
            ),
            b2: self::player(
                world,
                team_b.secondary().number.into(),
                team_b.secondary().dual_stick,
                PlayerSlot::B2,
            ),
        },
    };
    world.resources.insert(ent_signs);

    // Pins
    world.run_system(self::pins, ());
}

pub fn pins(world: &World, root: Root<Data>) {
    let Constants {
        pin_count,
        pin_padding,
        pin_radius,
        ..
    } = root.constant;
    let Sprites { a_pin, b_pin, .. } = root.sprite;
    let screen = root.court.size();
    let screen_bounds = screen / 2.;
    let shift = (screen.y - pin_padding.y * 2.) / pin_count as f32;

    for n in 0..pin_count {
        let x_padding = pin_radius + pin_padding.x;
        let y = -screen_bounds.y + pin_padding.y + pin_radius * 2. + (shift * n as f32);

        world
            .spawn()
            .insert(Pin)
            .insert(Team::A)
            .insert(AtlasSprite::new(a_pin))
            .insert(pin::animation_bank())
            .insert(path2d::pin(&root))
            .insert(Path2dToggle::new(path2d::color::PIN))
            .insert(Transform::from_translation(Vec3::new(
                -screen_bounds.x + x_padding,
                y,
                layers::PIN,
            )))
            .id();

        world
            .spawn()
            .insert(Pin)
            .insert(Team::B)
            .insert(AtlasSprite::new(b_pin))
            .insert(pin::animation_bank())
            .insert(path2d::pin(&root))
            .insert(Path2dToggle::new(path2d::color::PIN))
            .insert(Transform::from_translation(Vec3::new(
                screen_bounds.x - x_padding,
                y,
                layers::PIN,
            )))
            .id();
    }
}

pub fn new_player_transform(player_id: PlayerSlot, root: &Data) -> Transform {
    let bounds = root.constant.player_bounds;
    let mut pos = Vec2::new(bounds.x / 2., bounds.y / 2.);

    match player_id {
        PlayerSlot::A1 => pos *= vec2(-1., 1.),
        PlayerSlot::A2 => pos *= vec2(-1., -1.),
        PlayerSlot::B1 => (),
        PlayerSlot::B2 => pos *= vec2(1., -1.),
    }

    Transform::from_translation(Vec3::new(pos.x, pos.y, layers::HITO))
}

pub fn player(world: &World, number: Option<usize>, dual_stick: bool, slot: PlayerSlot) -> Entity {
    let asset_server = world.asset_server();
    let root = asset_server.root::<Data>();
    let transform = new_player_transform(slot, &root);
    let team = slot.team();
    let animations = asset_server.get(root.sprite.player_animations);

    let Sprites {
        player_a,
        player_b,
        player_a2,
        player_b2,
        aim_cone,
        aim_arrow,
        lstick_indicator,
        rstick_indicator,
        ..
    } = root.sprite;

    let Constants { player_radius, .. } = root.constant;

    let mut player = world.spawn();

    player
        .insert(transform)
        .insert(Player::new(slot))
        .insert(State::new("wait"))
        .insert(path2d::player(&root))
        .insert(Path2dToggle::new(path2d::color::FREE));

    let sprite_offset =
        (asset_server.get::<Atlas>(player_a).value().tile_size.y / 2.) - player_radius * 2.;

    world
        .spawn()
        .insert(PlayerSprite)
        .insert(AtlasSprite::new(match slot {
            PlayerSlot::A1 => player_a,
            PlayerSlot::A2 => player_a2,
            PlayerSlot::B1 => player_b,
            PlayerSlot::B2 => player_b2,
        }))
        .insert(animations.to_bank(ustr("idle")))
        .insert(Follow::XY {
            target: player.id(),
            offset: Vec2::new(0., sprite_offset),
        })
        .insert(Transform::from_z(layers::HITO));

    world
        .spawn()
        .insert(PlayerShadowSprite)
        .insert(Sprite {
            image: match team {
                Team::A => root.sprite.p1_shadow,
                Team::B => root.sprite.p2_shadow,
            },
            ..Default::default()
        })
        .insert(Follow::XY {
            target: player.id(),
            offset: Vec2::new(0., -4.),
        })
        .insert(Transform::from_z(layers::HITO_SHADOW));

    // dual stick left or right indicator
    if dual_stick {
        world
            .spawn()
            .insert(StickIndicator)
            .insert(Sprite {
                image: match slot {
                    PlayerSlot::A1 | PlayerSlot::B1 => lstick_indicator,
                    PlayerSlot::A2 | PlayerSlot::B2 => rstick_indicator,
                },
                ..Default::default()
            })
            .insert(Follow::XY {
                target: player.id(),
                offset: Vec2::new(0., 22.),
            })
            .insert(Transform::from_z(layers::HITO_SHADOW));
    }
    if let Some(number) = number {
        world
            .spawn()
            .insert(Sprite {
                image: **root.menu.team_select.player_icons()[number],
                ..Default::default()
            })
            .insert(Follow::XY {
                target: player.id(),
                offset: Vec2::new(0., -18.),
            })
            .insert(Lifetime::seconds(3.0))
            .insert(Transform::from_z(layers::HITO_SHADOW));
    }

    world
        .spawn()
        .insert(AimArrow(player.id()))
        .insert(Sprite {
            color: Color::NONE,
            image: aim_arrow,
            ..Default::default()
        })
        .insert(Transform::from_z(layers::AIMARROW));
    world
        .spawn()
        .insert(AimCone(player.id()))
        .insert(Sprite {
            color: Color::NONE,
            image: aim_cone,
            ..Default::default()
        })
        .insert(Transform::from_z(layers::AIMCONE));

    player.id()
}
