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
pub fn lan_ui_update(ui: &World) {
    let mut lan_ui = ui.resource_mut::<LanUI>();
    let local_inputs = ui.resource::<LocalInputs>();
    let mut matchmaker = ui.resource_mut::<Matchmaker>();

    if matchmaker.network_match_socket().is_some() {
        ui.resources.insert(lan_ui.service);
        start_fade(
            ui,
            FadeTransition {
                hide: lan_ui_hide,
                prep: play_online_prep,
                finish: |_| {},
            },
        );
        return;
    }

    let keyboard = ui.resource::<KeyboardInputs>();

    for event in &keyboard.key_events {
        if let Maybe::Set(key_code) = event.key_code {
            if key_code == KeyCode::Escape && event.button_state == ButtonState::Pressed {
                start_fade(
                    ui,
                    FadeTransition {
                        hide: lan_ui_leave,
                        prep: splash_prep,
                        finish: splash_finish,
                    },
                );
            }
        }
    }

    fn lan_ui_action(
        output: LanUIState,
        matchmaker: &mut RefMut<'_, Matchmaker>,
        lan_ui: &mut RefMut<'_, LanUI>,
    ) {
        match output {
            LanUIState::Host => {
                if matchmaker.is_hosting() {
                    matchmaker.lan_cancel();
                } else {
                    matchmaker.lan_host();
                }
            }
            LanUIState::Server(i) => {
                if let Some(server) = matchmaker.lan_servers().get(i).cloned() {
                    matchmaker.lan_join(&server);
                }
            }
            LanUIState::Disconnected => {
                lan_ui.state = LanUIState::Host;
            }
        }
    }

    if let Some(state) = lan_ui.output {
        lan_ui_action(state, &mut matchmaker, &mut lan_ui);
        return;
    }

    for (_gamepad, input) in local_inputs.iter() {
        if input.south.just_pressed() {
            lan_ui_action(lan_ui.state, &mut matchmaker, &mut lan_ui);
            return;
        }
        if input.west.just_pressed() {
            start_fade(
                ui,
                FadeTransition {
                    hide: lan_ui_leave,
                    prep: splash_prep,
                    finish: splash_finish,
                },
            );
            return;
        }
        if input.up.just_pressed() {
            lan_ui.state.cycle_up();
        }
        if input.down.just_pressed() {
            lan_ui.state.cycle_down();
        }
    }
}
