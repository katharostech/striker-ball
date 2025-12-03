use super::*;

pub fn lan_select_hide(world: &World) {
    world.resource_mut::<LanSelect>().visible = false;
}
pub fn lan_select_prep(world: &World) {
    world.resource_mut::<LanSelect>().visible = true;
}
pub fn lan_select_finish(world: &World) {
    *world.resource_mut() = MenuState::LanSelect;
}
pub fn lan_ui_hide(world: &World) {
    world.resource_mut::<Matchmaker>().disable_search();
    world.resource_mut::<LanUI>().visible = false;
}
pub fn lan_ui_leave(world: &World) {
    world.resource_mut::<Matchmaker>().lan_cancel();
    world.resource_mut::<Matchmaker>().disable_search();
    world.resource_mut::<LanUI>().visible = false;
}
pub fn lan_ui_prep(world: &World) {
    world.resource_mut::<Matchmaker>().enable_search();
    world.resource_mut::<LanUI>().visible = true;
}
pub fn lan_ui_finish(world: &World) {
    *world.resource_mut() = MenuState::Lan;
}
pub fn play_online_prep(ui: &World) {
    let socket = ui.resource::<Matchmaker>().network_match_socket().unwrap();
    let service_type = *ui.resource::<ServiceType>();
    let mut sessions = ui.resource_mut::<Sessions>();

    sessions.create_play(PlayMode::Online {
        socket,
        service_type,
    });
    *ui.resource_mut() = MenuState::InNetworkGame;
}
pub fn lan_select_transition(world: &World, output: LanSelectOutput) {
    match output {
        LanSelectOutput::Exit => {
            start_fade(
                world,
                FadeTransition {
                    hide: lan_select_hide,
                    prep: splash_prep,
                    finish: splash_finish,
                },
            );
        }
        LanSelectOutput::ServiceType(service) => {
            world
                .resource_mut::<Matchmaker>()
                .update_service_name(service.service_name());
            world.resource_mut::<LanUI>().service = service;
            start_fade(
                world,
                FadeTransition {
                    hide: lan_select_hide,
                    prep: lan_ui_prep,
                    finish: lan_ui_finish,
                },
            );
        }
    }
}
pub fn lan_ui_transition(world: &World, output: LanUIOutput) {
    let lan_ui = world.resource::<LanUI>();
    let mut matchmaker = world.resource_mut::<Matchmaker>();

    if matchmaker.network_match_socket().is_some() {
        world.resources.insert(lan_ui.service);
        start_fade(
            world,
            FadeTransition {
                hide: lan_ui_hide,
                prep: play_online_prep,
                finish: |_| {},
            },
        );
        return;
    }
    match output {
        LanUIOutput::HostCancel => {
            if matchmaker.is_hosting() {
                matchmaker.lan_cancel();
            } else {
                matchmaker.lan_host();
            }
        }
        LanUIOutput::Server(i) => {
            if let Some(server) = matchmaker.lan_servers().get(i).cloned() {
                matchmaker.lan_join(&server);
            }
        }
        LanUIOutput::Exit => start_fade(
            world,
            FadeTransition {
                hide: lan_ui_leave,
                prep: splash_prep,
                finish: splash_finish,
            },
        ),
    }
}
