use super::*;

pub struct MenuPlugin;
impl SessionPlugin for MenuPlugin {
    fn install(self, session: &mut SessionBuilder) {
        session.init_resource::<MenuState>();
        session.init_resource::<FadeTransition>();

        session.install_plugin(SettingsUi::default());
        session.install_plugin(CreditsUi::default());
        session.install_plugin(HowToPlay::default());
        session.install_plugin(TeamSelect::default());
        session.install_plugin(Pause::default());
        session.install_plugin(Splash {
            visual: Visual::new_shown(),
            ..Default::default()
        });
        session.install_plugin(Fade::new(
            0.5,
            0.15,
            0.5,
            Color::BLACK,
            egui::Order::Tooltip,
        ));

        #[cfg(not(target_arch = "wasm32"))]
        {
            session.install_plugin(
                MatchmakerPlugin::new(MATCHMAKER_SERVICE_NAME)
                    .refresh(1.0)
                    .player_count(2),
            );
            session.install_plugin(LanSelect::default());
            session.install_plugin(LanUI::default());
            session.install_plugin(NetworkQuit::default());
        }

        session.add_startup_system(set_volume_scales);
        session.add_startup_system(play_menu_music);
        session.add_system_to_stage(First, update_menu);
    }
}

fn play_menu_music(root: Root<Data>, mut audio: ResMut<AudioCenter>) {
    audio.play_music_advanced(
        *root.sound.menu_music,
        root.sound.menu_music.volume(),
        true,
        false,
        0.0,
        1.0,
        true,
    );
}
fn set_volume_scales(mut storage: ResMut<Storage>, mut audio: ResMut<AudioCenter>) {
    let Settings {
        sfx_volume,
        music_volume,
    } = storage.get_or_insert_default::<Settings>();

    audio.set_volume_scales(1.0, music_volume.scale(), sfx_volume.scale());
}
