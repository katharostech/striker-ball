use super::*;

/// Collection type variants for gathering inputs for two characters on one computer.
#[derive(HasSchema, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TeamSource {
    /// Represents one controller for two characters.
    OnePlayer(u32),
    /// Represents two controllers for two characters.
    TwoPlayer(u32, u32),
    /// Represents one controller and a CPU partner in offline play.
    OneCPU(u32, PlayerSlot),
    /// Represents two CPU players controlling a team in offline play.
    TwoCPUs(PlayerSlot, PlayerSlot),
}
impl Default for TeamSource {
    fn default() -> Self {
        Self::OnePlayer(0)
    }
}

#[derive(Default, Debug, PartialEq)]
// NOTE: For Ggrs, the only need for the collector is to give dense input every SessionRunner::step.
pub struct PlayTeamInputCollector {
    source: TeamSource,
    // TODO: I think current may be the wrong name but I still have to nail down all the input stuff
    current: PlayTeamInput,
    // TODO: I think I can put individual collectors into this collector.
}
impl PlayTeamInputCollector {
    pub fn new(source: TeamSource) -> Self {
        Self {
            source,
            current: Default::default(),
        }
    }
    pub fn set_source(&mut self, source: TeamSource) {
        self.source = source;
    }
    /// This is a temporary function that is needed for CPU player input.
    /// I believe this can be replaced with the [`InputCollector`] trait method
    /// once it supports the borrowing of the world and that is considered safe.
    pub fn offline_apply_inputs(&mut self, world: &World) {
        match self.source {
            TeamSource::TwoCPUs(p1_slot, p2_slot) => {
                cpu_player::apply_cpu_input(world, p1_slot, &mut self.current.p1);
                cpu_player::apply_cpu_input(world, p2_slot, &mut self.current.p2);
            }
            TeamSource::OneCPU(_p1, p2_slot) => {
                self.apply_inputs(
                    &world.resource::<Mapping>(),
                    &world.resource::<KeyboardInputs>(),
                    &world.resource::<GamepadInputs>(),
                );
                cpu_player::apply_cpu_input(world, p2_slot, &mut self.current.p2);
            }
            _ => {
                self.apply_inputs(
                    &world.resource::<Mapping>(),
                    &world.resource::<KeyboardInputs>(),
                    &world.resource::<GamepadInputs>(),
                );
            }
        }
    }
}
impl InputCollector<'_, Mapping, BlankSource, PlayTeamInput> for PlayTeamInputCollector {
    // Called on cpu cycle as opposed to the frame update.
    fn apply_inputs(
        &mut self,
        _mapping: &Mapping,
        keyboard: &KeyboardInputs,
        gamepad: &GamepadInputs,
    ) {
        match self.source {
            TeamSource::TwoCPUs(..) => {}
            TeamSource::OneCPU(id, ..) => {
                let input = &mut self.current;
                for event in &gamepad.gamepad_events {
                    if *event.gamepad_id() != id {
                        continue;
                    }
                    match event {
                        GamepadEvent::Axis(GamepadAxisEvent { axis, value, .. }) => match axis {
                            GamepadAxis::LeftStickX => {
                                input.p1.x = *value;
                            }
                            GamepadAxis::LeftStickY => {
                                input.p1.y = *value;
                            }
                            _ => {}
                        },
                        GamepadEvent::Button(GamepadButtonEvent { button, value, .. }) => {
                            match button {
                                GamepadButton::South => {
                                    input.p1.shoot.apply_value(*value);
                                }
                                GamepadButton::West => {
                                    input.p1.pass.apply_value(*value);
                                }
                                GamepadButton::RightTrigger2 => {
                                    input.p1.shoot.apply_value(*value);
                                }
                                GamepadButton::LeftTrigger2 => {
                                    input.p1.shoot.apply_value(*value);
                                }
                                GamepadButton::LeftTrigger => {
                                    input.p1.pass.apply_value(*value);
                                }
                                GamepadButton::RightTrigger => {
                                    input.p1.pass.apply_value(*value);
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
            TeamSource::OnePlayer(id) => {
                let input = &mut self.current;
                for event in &gamepad.gamepad_events {
                    if *event.gamepad_id() != id {
                        continue;
                    }
                    match event {
                        GamepadEvent::Axis(GamepadAxisEvent { axis, value, .. }) => match axis {
                            GamepadAxis::LeftStickX => {
                                input.p1.x = *value;
                            }
                            GamepadAxis::LeftStickY => {
                                input.p1.y = *value;
                            }
                            GamepadAxis::RightStickX => {
                                input.p2.x = *value;
                            }
                            GamepadAxis::RightStickY => {
                                input.p2.y = *value;
                            }
                            _ => {}
                        },
                        GamepadEvent::Button(GamepadButtonEvent { button, value, .. }) => {
                            match button {
                                GamepadButton::South => {
                                    input.p1.shoot.apply_value(*value);
                                    input.p2.shoot.apply_value(*value);
                                }
                                GamepadButton::West => {
                                    input.p1.pass.apply_value(*value);
                                    input.p2.pass.apply_value(*value);
                                }
                                GamepadButton::RightTrigger2 => {
                                    input.p1.shoot.apply_value(*value);
                                    input.p2.shoot.apply_value(*value);
                                }
                                GamepadButton::LeftTrigger2 => {
                                    input.p1.shoot.apply_value(*value);
                                    input.p2.shoot.apply_value(*value);
                                }
                                GamepadButton::LeftTrigger => {
                                    input.p1.pass.apply_value(*value);
                                }
                                GamepadButton::RightTrigger => {
                                    input.p2.pass.apply_value(*value);
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
            TeamSource::TwoPlayer(p1, p2) => {
                for event in &gamepad.gamepad_events {
                    let input = if *event.gamepad_id() == p1 {
                        &mut self.current.p1
                    } else if *event.gamepad_id() == p2 {
                        &mut self.current.p2
                    } else {
                        continue;
                    };
                    match event {
                        GamepadEvent::Axis(GamepadAxisEvent { axis, value, .. }) => match axis {
                            GamepadAxis::LeftStickX => {
                                input.x = *value;
                            }
                            GamepadAxis::LeftStickY => {
                                input.y = *value;
                            }
                            _ => {}
                        },
                        GamepadEvent::Button(GamepadButtonEvent { button, value, .. }) => {
                            match button {
                                GamepadButton::South => {
                                    input.shoot.apply_value(*value);
                                }
                                GamepadButton::West => {
                                    input.pass.apply_value(*value);
                                }
                                GamepadButton::RightTrigger2 => {
                                    input.shoot.apply_value(*value);
                                }
                                GamepadButton::LeftTrigger2 => {
                                    input.shoot.apply_value(*value);
                                }
                                GamepadButton::RightTrigger => {
                                    input.pass.apply_value(*value);
                                }
                                GamepadButton::LeftTrigger => {
                                    input.pass.apply_value(*value);
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    fn update_just_pressed(&mut self) {
        // Not neccessary as this is called at the same time as `apply_inputs`.
    }
    // Called on frame update as opposed to the cpu cycle.
    // By the time this is called, the dense input is sent across the network.
    fn advance_frame(&mut self) {
        self.current.p1.pass.advance();
        self.current.p1.shoot.advance();
        self.current.p2.pass.advance();
        self.current.p2.shoot.advance();
    }
    // This is only ever used for getting *dense* input, so we should probably return
    // the dense instead. This would reduce some processing as well since we're not
    // necessarily using all of the non-dense input to output dense input.
    //
    // I'm skipping use of the control_source to opt for chosen source directly on
    // the collector.
    fn get_control(&self, _player_idx: usize, _control_source: BlankSource) -> &PlayTeamInput {
        &self.current
    }
}
