#![allow(clippy::too_many_arguments)]
use bones_bevy_renderer::BonesBevyRenderer;
use striker_ball::*;

const fn namespace() -> (&'static str, &'static str, &'static str) {
    ("striker_ball", "example", "matchmaking")
}

fn main() {
    setup_logs!(namespace());

    crate::register_schemas();

    let mut game = Game::new();

    game.install_plugin(DefaultGamePlugin);
    game.install_plugin(LocalInputGamePlugin);
    game.init_shared_resource::<AssetServer>();
    game.sessions
        .create_with("matchmaking", MatchmakingExamplePlugin);

    BonesBevyRenderer::new(game)
        .namespace(namespace())
        .app()
        .run();
}

struct MatchmakingExamplePlugin;
impl SessionPlugin for MatchmakingExamplePlugin {
    fn install(self, session: &mut SessionBuilder) {
        session
            .install_plugin(DefaultSessionPlugin)
            .install_plugin(Matchmaker::new("striker_ball").refresh(1.0).player_count(2))
            .install_plugin(MatchmakingMenu(Visual::new_shown()))
            .add_system_to_stage(Update, MatchmakingMenu::show);
    }
}

#[derive(HasSchema, Clone, Default, Deref, DerefMut)]
pub struct MatchmakingMenu(pub Visual);
impl SessionPlugin for MatchmakingMenu {
    fn install(self, session: &mut SessionBuilder) {
        session.insert_resource(self);
    }
}
impl MatchmakingMenu {
    pub fn show(ctx: Res<EguiCtx>, menu: Res<MatchmakingMenu>, mut matchmaker: ResMut<Matchmaker>) {
        if !menu.shown() {
            return;
        }
        use egui::*;
        CentralPanel::default().show(&ctx, |ui| {
            if let Some(socket) = matchmaker.network_match_socket() {
                ui.label(format!("Joined as player: {}", socket.player_idx()));
                if ui.button("Leave").clicked() {
                    matchmaker.lan_join_cancel();
                }
            } else if matchmaker.is_hosting() {
                ui.label("Hosting");
                ui.label(format!(
                    "players: {:?}",
                    matchmaker.joined_players().unwrap()
                ));
                if ui.button("Cancel").clicked() {
                    matchmaker.lan_host_cancel()
                }
            } else {
                ui.text_edit_singleline(&mut matchmaker.host_name);
                if ui.button("Host").clicked() {
                    matchmaker.lan_host()
                }
                ui.label("Servers:");

                let mut join_server = None;
                for server in matchmaker.lan_servers() {
                    let ping = if let Some(ping) = server.ping {
                        ping.to_string()
                    } else {
                        "X".to_string()
                    };
                    let red = server.ping.unwrap_or(0) as u8;
                    ui.horizontal(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("Ping: {ping}"))
                                    .color(Color32::from_rgb(red, 255, 255)),
                            );
                        });
                        ui.label(format!("{}: ", server.service.get_fullname()));
                        if ui
                            .add_enabled(!matchmaker.is_hosting(), Button::new("Join"))
                            .clicked()
                        {
                            join_server = Some(server.clone());
                        }
                    });
                }
                if let Some(server) = join_server {
                    matchmaker.lan_join(&server);
                }
            }
        });
    }
}
