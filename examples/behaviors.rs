#![allow(clippy::too_many_arguments)]
use bones_bevy_renderer::BonesBevyRenderer;
use bones_framework::prelude::*;
use striker_ball::*;

const fn namespace() -> (&'static str, &'static str, &'static str) {
    ("striker_ball", "example", "behaviors")
}

fn main() {
    setup_logs!(namespace());

    crate::register_schemas();

    let mut game = Game::new();

    game.install_plugin(DefaultGamePlugin);
    game.install_plugin(LocalInputGamePlugin);
    game.init_shared_resource::<AssetServer>();

    game.sessions
        .create_with("play", |builder: &mut SessionBuilder| {
            builder
                .install_plugin(DefaultSessionPlugin)
                .install_plugin(self::BehaviorsPlugin)
                .install_plugin(self::ScenePlugin { mode: default() })
                .add_startup_system(play::set_player_states_free);
        });

    BonesBevyRenderer::new(game)
        .namespace(namespace())
        .app()
        .run();
}
