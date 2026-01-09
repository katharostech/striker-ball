use super::*;

#[derive(HasSchema, Clone, Default)]
#[repr(C)]
pub struct SettingsAssets {
    pub settings_frame: SizedImageAsset,
    pub settings_slider: SizedImageAsset,

    pub sfx_volume_label_highlight: SizedImageAsset,
    pub sfx_volume_label_offset: Vec2,
    pub sfx_volume_slider_start: Vec2,
    pub sfx_volume_slider_length: f32,

    pub music_volume_label_highlight: SizedImageAsset,
    pub music_volume_label_offset: Vec2,
    pub music_volume_slider_start: Vec2,
    pub music_volume_slider_length: f32,
}

#[derive(HasSchema, Clone, Copy, Default, PartialEq, Eq)]
pub enum SettingsState {
    #[default]
    SFX,
    Music,
}
impl SettingsState {
    pub fn cycle(&mut self) {
        *self = match self {
            SettingsState::SFX => SettingsState::Music,
            SettingsState::Music => SettingsState::SFX,
        }
    }
}

#[derive(HasSchema, Clone, Default, Deref, DerefMut)]
pub struct Settings {
    #[deref]
    pub state: SettingsState,
    pub visible: bool,
}

impl SessionPlugin for Settings {
    fn install(self, session: &mut SessionBuilder) {
        session.insert_resource(self);
    }
}
fn foreground() -> egui::LayerId {
    use egui::*;
    LayerId::new(Order::Foreground, Id::new("splash_foreground"))
}

pub const SLIDER_INCREMENTS: u8 = 7;

// TODO: Use one settings type for settings storage

#[derive(HasSchema, Clone, Copy, Deref, DerefMut)]
#[repr(C)]
pub struct SfxVolumeSetting(pub u8);
impl Default for SfxVolumeSetting {
    fn default() -> Self {
        Self(SLIDER_INCREMENTS)
    }
}
impl SfxVolumeSetting {
    pub fn scale(&self) -> f32 {
        self.0 as f32 / SLIDER_INCREMENTS as f32
    }
}

#[derive(HasSchema, Clone, Copy, Deref, DerefMut)]
#[repr(C)]
pub struct MusicVolumeSetting(pub u8);
impl Default for MusicVolumeSetting {
    fn default() -> Self {
        Self(SLIDER_INCREMENTS)
    }
}
impl MusicVolumeSetting {
    pub fn scale(&self) -> f32 {
        self.0 as f32 / SLIDER_INCREMENTS as f32
    }
}

pub struct SettingsOutput;

impl Settings {
    pub fn process_input(&mut self, world: &World) -> Option<SettingsOutput> {
        let mut output = None;

        let local_inputs = world.resource::<LocalInputs>();
        let mut storage = world.resource_mut::<Storage>();
        let asset_server = world.asset_server();
        let root = asset_server.root::<Data>();

        for (_source, input) in local_inputs.iter() {
            if input.menu_back.just_pressed() {
                output = Some(SettingsOutput)
            }
            if input.menu_down.just_pressed() || input.menu_up.just_pressed() {
                self.cycle();
            }
            if input.menu_left.just_pressed() {
                match self.state {
                    SettingsState::SFX => {
                        let sfx_volume = storage.get_or_insert_default_mut::<SfxVolumeSetting>();
                        **sfx_volume = sfx_volume.saturating_sub(1);
                        storage.save();
                        tracing::info!("storage saved");
                        world.resource_mut::<AudioCenter>().set_volume_scales(
                            1.0,
                            storage
                                .get_or_insert_default::<MusicVolumeSetting>()
                                .scale(),
                            storage.get_or_insert_default::<SfxVolumeSetting>().scale(),
                        );
                        world.resource_mut::<AudioCenter>().play_sound(
                            *root.sound.pin_explosion,
                            root.sound.pin_explosion.volume(),
                        );
                    }
                    SettingsState::Music => {
                        let music_volume =
                            storage.get_or_insert_default_mut::<MusicVolumeSetting>();
                        **music_volume = music_volume.saturating_sub(1);
                        storage.save();
                        tracing::info!("storage saved");
                        world.resource_mut::<AudioCenter>().set_volume_scales(
                            1.0,
                            storage
                                .get_or_insert_default::<MusicVolumeSetting>()
                                .scale(),
                            storage.get_or_insert_default::<SfxVolumeSetting>().scale(),
                        );
                    }
                }
            }
            if input.menu_right.just_pressed() {
                match self.state {
                    SettingsState::SFX => {
                        let sfx_volume = storage.get_or_insert_default_mut::<SfxVolumeSetting>();
                        **sfx_volume = (**sfx_volume + 1).min(SLIDER_INCREMENTS);
                        storage.save();
                        tracing::info!("storage saved");
                        world.resource_mut::<AudioCenter>().set_volume_scales(
                            1.0,
                            storage
                                .get_or_insert_default::<MusicVolumeSetting>()
                                .scale(),
                            storage.get_or_insert_default::<SfxVolumeSetting>().scale(),
                        );
                        world.resource_mut::<AudioCenter>().play_sound(
                            *root.sound.pin_explosion,
                            root.sound.pin_explosion.volume(),
                        );
                    }
                    SettingsState::Music => {
                        let music_volume =
                            storage.get_or_insert_default_mut::<MusicVolumeSetting>();
                        **music_volume = (**music_volume + 1).min(SLIDER_INCREMENTS);
                        storage.save();
                        tracing::info!("storage saved");
                        world.resource_mut::<AudioCenter>().set_volume_scales(
                            1.0,
                            storage
                                .get_or_insert_default::<MusicVolumeSetting>()
                                .scale(),
                            storage.get_or_insert_default::<SfxVolumeSetting>().scale(),
                        );
                    }
                }
            }
        }
        output
    }

    pub fn process_ui(&mut self, world: &World) -> Option<SettingsOutput> {
        let mut output = None;

        if !self.visible {
            return output;
        }

        let asset_server = world.resource::<AssetServer>();
        let root = asset_server.root::<Data>();
        let textures = world.resource::<EguiTextures>();
        let ctx = world.resource::<EguiCtx>();
        let mut storage = world.resource_mut::<Storage>();

        let SettingsAssets {
            settings_frame,
            settings_slider,
            sfx_volume_label_highlight,
            sfx_volume_label_offset,
            sfx_volume_slider_start,
            sfx_volume_slider_length,
            music_volume_label_highlight,
            music_volume_label_offset,
            music_volume_slider_start,
            music_volume_slider_length,
        } = root.menu.settings;

        use egui::*;

        let area = Area::new("splash")
            .anchor(Align2::CENTER_CENTER, [0., 0.])
            .show(&ctx, |ui| {
                ui.image(load::SizedTexture::new(
                    textures.get(root.menu.splash.bg),
                    root.screen_size.to_array(),
                ));
            });
        let mut painter = ctx.layer_painter(foreground());

        painter.set_clip_rect(area.response.rect);

        let settings_rect = settings_frame
            .image_painter()
            .align2(Align2::CENTER_CENTER)
            .pos(area.response.rect.center())
            .paint(&painter, &textures);

        match self.state {
            SettingsState::SFX => {
                sfx_volume_label_highlight
                    .image_painter()
                    .pos(settings_rect.min)
                    .offset(sfx_volume_label_offset.to_array().into())
                    .paint(&painter, &textures);
            }
            SettingsState::Music => {
                music_volume_label_highlight
                    .image_painter()
                    .pos(settings_rect.min)
                    .offset(music_volume_label_offset.to_array().into())
                    .paint(&painter, &textures);
            }
        }

        // Naming scheme
        // _setting is u8
        // _slider is f32

        let sfx_volume_setting = **storage.get_or_insert_default::<SfxVolumeSetting>();

        let sfx_volume_slider_step = sfx_volume_slider_length / SLIDER_INCREMENTS as f32;
        let sfx_volume_slider_drag =
            ctx.data_mut(|w| *w.get_temp_mut_or_default::<f32>(Id::new("sfx_volume_slider_drag")));
        let sfx_volume_slider_dragging = ctx.data_mut(|w| {
            *w.get_temp_mut_or_default::<bool>(Id::new("sfx_volume_slider_dragging"))
        });
        let sfx_volume_slider_drag_steps = sfx_volume_slider_drag / sfx_volume_slider_step;
        let sfx_volume_slider_drag_steps_clamped = sfx_volume_slider_drag_steps
            .clamp(
                0.0 - sfx_volume_setting as f32,
                SLIDER_INCREMENTS as f32 - sfx_volume_setting as f32,
            )
            .round();
        let sfx_volume_slider_steps = sfx_volume_setting as f32 * sfx_volume_slider_step;
        let sfx_volume_slider_offset =
            sfx_volume_slider_drag_steps_clamped * sfx_volume_slider_step + sfx_volume_slider_steps;

        let music_volume_setting = **storage.get_or_insert_default::<MusicVolumeSetting>();

        let music_volume_slider_step = music_volume_slider_length / SLIDER_INCREMENTS as f32;
        let music_volume_slider_drag = ctx
            .data_mut(|w| *w.get_temp_mut_or_default::<f32>(Id::new("music_volume_slider_drag")));
        let music_volume_slider_dragging = ctx.data_mut(|w| {
            *w.get_temp_mut_or_default::<bool>(Id::new("music_volume_slider_dragging"))
        });
        let music_volume_slider_drag_steps = music_volume_slider_drag / music_volume_slider_step;
        let music_volume_slider_drag_steps_clamped = music_volume_slider_drag_steps
            .clamp(
                0.0 - music_volume_setting as f32,
                SLIDER_INCREMENTS as f32 - music_volume_setting as f32,
            )
            .round();
        let music_volume_slider_steps = music_volume_setting as f32 * music_volume_slider_step;
        let music_volume_slider_offset = music_volume_slider_drag_steps_clamped
            * music_volume_slider_step
            + music_volume_slider_steps;

        let sfx_volume_slider_rect = settings_slider
            .image_painter()
            .pos(sfx_volume_slider_start.to_array().into())
            .offset(pos2(sfx_volume_slider_offset, 0.0))
            .paint(&painter, &textures);

        let music_volume_slider_rect = settings_slider
            .image_painter()
            .pos(music_volume_slider_start.to_array().into())
            .offset(pos2(music_volume_slider_offset, 0.0))
            .paint(&painter, &textures);

        let dragging = ctx.0.input(|r| r.pointer.is_decidedly_dragging());
        let press_origin = ctx.0.input(|r| r.pointer.press_origin());
        let latest_pos = ctx.0.input(|r| r.pointer.latest_pos());

        if dragging {
            if let Some(press_origin) = press_origin {
                let latest_pos = latest_pos.unwrap_or(press_origin);
                if sfx_volume_slider_dragging {
                    ctx.data_mut(|w| {
                        *w.get_temp_mut_or_default::<f32>(Id::new("sfx_volume_slider_drag")) =
                            latest_pos.x - press_origin.x;
                    });
                } else if sfx_volume_slider_rect.contains(press_origin) {
                    self.state = SettingsState::SFX;
                    ctx.data_mut(|w| {
                        *w.get_temp_mut_or_default::<bool>(Id::new("sfx_volume_slider_dragging")) =
                            true
                    });
                }
                if music_volume_slider_dragging {
                    ctx.data_mut(|w| {
                        *w.get_temp_mut_or_default::<f32>(Id::new("music_volume_slider_drag")) =
                            latest_pos.x - press_origin.x;
                    });
                } else if music_volume_slider_rect.contains(press_origin) {
                    self.state = SettingsState::Music;
                    ctx.data_mut(|w| {
                        *w.get_temp_mut_or_default::<bool>(Id::new(
                            "music_volume_slider_dragging",
                        )) = true
                    });
                }
            }
        } else {
            **storage.get_or_insert_default_mut::<SfxVolumeSetting>() =
                (sfx_volume_setting as i8 + sfx_volume_slider_drag_steps_clamped as i8) as u8;
            **storage.get_or_insert_default_mut::<MusicVolumeSetting>() =
                (music_volume_setting as i8 + music_volume_slider_drag_steps_clamped as i8) as u8;
            // TODO: use u8s reliably (make sure they are staying within bounds)

            if sfx_volume_slider_dragging || music_volume_slider_dragging {
                world.resource_mut::<AudioCenter>().set_volume_scales(
                    1.0,
                    storage
                        .get_or_insert_default::<MusicVolumeSetting>()
                        .scale(),
                    storage.get_or_insert_default::<SfxVolumeSetting>().scale(),
                );
                storage.save();
                tracing::info!("storage saved");

                // play demos
                if sfx_volume_slider_dragging {
                    world
                        .resource_mut::<AudioCenter>()
                        .play_sound(*root.sound.pin_explosion, root.sound.pin_explosion.volume());
                }
            }

            ctx.data_mut(|w| {
                *w.get_temp_mut_or_default::<f32>(Id::new("sfx_volume_slider_drag")) = 0.0;
            });
            ctx.data_mut(|w| {
                *w.get_temp_mut_or_default::<bool>(Id::new("sfx_volume_slider_dragging")) = false
            });
            ctx.data_mut(|w| {
                *w.get_temp_mut_or_default::<f32>(Id::new("music_volume_slider_drag")) = 0.0;
            });
            ctx.data_mut(|w| {
                *w.get_temp_mut_or_default::<bool>(Id::new("music_volume_slider_dragging")) = false
            });
        }

        output
    }
}
