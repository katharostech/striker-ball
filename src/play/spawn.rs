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
        #[cfg(not(target_arch = "wasm32"))]
        PlayMode::Online {
            socket,
            service_type,
        } => {
            let primary_info = service_type.player_info_primary();
            let secondary_info = service_type.player_info_secondary();

            let local_team = match socket.player_idx() {
                0 => Team::A,
                1 => Team::B,
                _ => panic!("index out of player count bounds"),
            };

            let a1 = if local_team == Team::A {
                primary_info
            } else {
                PlayerInfo::Network
            };
            let a2 = if local_team == Team::A {
                secondary_info
            } else {
                PlayerInfo::Network
            };
            let b1 = if local_team == Team::B {
                primary_info
            } else {
                PlayerInfo::Network
            };
            let b2 = if local_team == Team::B {
                secondary_info
            } else {
                PlayerInfo::Network
            };

            PlayerEntSigns {
                a1: self::player(world, a1, PlayerSlot::A1),
                a2: self::player(world, a2, PlayerSlot::A2),
                b1: self::player(world, b1, PlayerSlot::B1),
                b2: self::player(world, b2, PlayerSlot::B2),
            }
        }
        PlayMode::Offline(PlayersInfo { a1, a2, b1, b2 }) => PlayerEntSigns {
            a1: self::player(world, *a1, PlayerSlot::A1),
            a2: self::player(world, *a2, PlayerSlot::A2),
            b1: self::player(world, *b1, PlayerSlot::B1),
            b2: self::player(world, *b2, PlayerSlot::B2),
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

pub fn new_player_transform(player_slot: PlayerSlot, root: &Data) -> Transform {
    let bounds = root.constant.player_bounds;
    let mut pos = Vec2::new(bounds.x / 2., bounds.y / 2.);

    match player_slot {
        PlayerSlot::A1 => pos *= vec2(-1., 1.),
        PlayerSlot::A2 => pos *= vec2(-1., -1.),
        PlayerSlot::B1 => (),
        PlayerSlot::B2 => pos *= vec2(1., -1.),
    }

    Transform::from_translation(Vec3::new(pos.x, pos.y, layers::PLAYER))
}

pub fn player(world: &World, player_info: PlayerInfo, slot: PlayerSlot) -> Entity {
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
        .insert(Follow::XYZ {
            target: player.id(),
            offset: Vec3::new(0., sprite_offset, 0.),
        })
        .insert(Transform::from_z(layers::PLAYER));

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
        .insert(Transform::from_z(layers::PLAYER_SHADOW));

    // dual stick left or right indicator
    if let PlayerInfo::Local {
        dual_stick: true, ..
    } = player_info
    {
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
            .insert(Transform::from_z(layers::PLAYER_SHADOW));
    }
    match player_info {
        PlayerInfo::Network => {}
        PlayerInfo::Local { number, .. } => {
            world
                .spawn()
                .insert(PlayerIndicator {
                    timer: Timer::from_seconds(3.0, TimerMode::Once),
                })
                .insert(Sprite {
                    image: **root.menu.team_select.player_icons()[number],
                    ..Default::default()
                })
                .insert(Follow::XY {
                    target: player.id(),
                    offset: Vec2::new(0., -18.),
                })
                .insert(Transform::from_z(layers::PLAYER_SHADOW));
        }
        PlayerInfo::CPU => {
            world
                .spawn()
                .insert(PlayerIndicator {
                    timer: Timer::from_seconds(3.0, TimerMode::Once),
                })
                .insert(Sprite {
                    image: root.sprite.cpu_indicator,
                    ..Default::default()
                })
                .insert(Follow::XY {
                    target: player.id(),
                    offset: Vec2::new(0., -18.),
                })
                .insert(Transform::from_z(layers::PLAYER_SHADOW));

            let mut cpu_state = world.spawn();
            // This entity is not tagged very much but the only thing
            // that needs to reference it is the cpu component below it.
            cpu_state
                .insert(CpuPlayerState)
                .insert(State::new(&cpu_player::state::chase()));

            player.insert(CpuPlayer {
                state_e: cpu_state.entity,
                input: Default::default(),
            });
        }
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
