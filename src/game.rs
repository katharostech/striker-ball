#![allow(clippy::too_many_arguments)]
use crate::*;
use bones_bevy_renderer::BonesBevyRenderer;
use bones_framework::prelude::*;

pub const fn namespace() -> (&'static str, &'static str, &'static str) {
    ("ktech", "studio", "striker_ball")
}

pub fn run() {
    setup_logs!(namespace());

    crate::register_schemas();

    let mut game = Game::new();

    game.install_plugin(DefaultGamePlugin);
    game.init_shared_resource::<AssetServer>();

    // By inserting `ClearColor` as a shared resource, every session
    // will by default read its own `ClearColor` as `BLACK`.
    // TODO: Check if shared_resources can be overwritten.
    game.insert_shared_resource(ClearColor(Color::BLACK));

    game.install_plugin(LocalInputGamePlugin);
    game.sessions.create_with(session::UI, UiSessionPlugin);

    BonesBevyRenderer::new(game)
        .namespace(namespace())
        .app()
        .run();
}
