mod assets;
mod data;

pub use assets::*;
pub use data::*;

use super::*;

impl SessionPlugin for TeamSelect {
    fn install(self, session: &mut SessionBuilder) {
        session.insert_resource(self);
    }
}

fn layer_id() -> egui::LayerId {
    use egui::*;
    LayerId::new(Order::Middle, Id::new("team_select_foreground"))
}

pub enum TeamSelectOutput {
    PlayersInfo(PlayersInfo),
    Exit,
}

impl TeamSelect {
    pub fn process_input(&mut self, world: &World) -> Option<TeamSelectOutput> {
        let assignments = self.get_player_signs();
        let local_inputs = world.resource::<LocalInputs>();
        let asset_server = world.asset_server();
        let root = asset_server.root::<Data>();

        for (source, input) in local_inputs.iter() {
            if let Some(ref players_info) = assignments {
                if input.start.just_pressed() && self.contains_source(*source) {
                    return Some(TeamSelectOutput::PlayersInfo(players_info.clone()));
                }
            }
            if input.start.just_pressed()
                || input.north.just_pressed()
                || input.east.just_pressed()
                || input.menu_select.just_pressed()
                || input.menu_back.just_pressed()
                || input.left_bump.just_pressed()
                || input.right_bump.just_pressed()
            {
                self.add_source(*source);
                if let SingleSource::Gamepad(gamepad_id) = source {
                    world.resource_mut::<GamepadsRumble>().set_rumble(
                        *gamepad_id,
                        GamepadRumbleIntensity::LIGHT_BOTH,
                        0.2,
                    );
                }
            }
            if input.menu_select.just_pressed() {
                self.ready_join(*source);
            }
            if input.menu_back.just_pressed() {
                self.reverse_join(*source);
            }
            if input.menu_back.just_held(root.menu.team_select.back_buffer) {
                return Some(TeamSelectOutput::Exit);
            }
            if input.menu_left.just_pressed() {
                self.left_join(*source);
            }
            if input.menu_right.just_pressed() {
                self.right_join(*source);
            }
        }
        None
    }
    pub fn process_ui(&mut self, world: &World) -> Option<TeamSelectOutput> {
        let mut output = None;

        if !self.visible {
            return output;
        }

        let ctx = world.resource::<EguiCtx>();
        let textures = world.resource::<EguiTextures>();
        let asset_server = world.resource::<AssetServer>();
        let root = asset_server.root::<Data>();

        let TeamSelectAssets {
            slots,
            a_team_background,
            b_team_background,
            center_controller_column,
            keyboard_icon,
            controller_icon,
            controller_icon_silhouette,
            pad_slot_bg,
            start,
            start_blink,
            back_btn,
            back_buffer,
            cpu_icon,
            partner_select_arrow,
            ..
        } = root.menu.team_select;

        let small_inner_font = asset_server.get(root.font.small_inner).family_name.clone();
        let small_outer_font = asset_server.get(root.font.small_outer).family_name.clone();

        use egui::*;
        let area = Area::new("self_area")
            .anchor(Align2::CENTER_CENTER, [0., 0.])
            .show(&ctx, |ui| {
                ui.style_mut().spacing.item_spacing = Vec2::ZERO;
                ui.horizontal(|ui| {
                    ui.image(a_team_background.sized_texture(&textures));
                    ui.image(center_controller_column.sized_texture(&textures));
                    ui.image(b_team_background.sized_texture(&textures));
                });
            });
        let origin = area.response.rect.min;

        let mut painter = ctx.layer_painter(layer_id());

        painter.set_clip_rect(Rect::from_min_size(
            origin,
            root.screen_size.to_array().into(),
        ));

        // This fixes a glitch with the ui animations when rendering new text.
        // All of it is invisible with the default `Color32`.
        painter.text(
            default(),
            Align2::LEFT_CENTER,
            "Not Ready",
            FontId {
                size: 7.0,
                family: FontFamily::Name(small_inner_font.clone()),
            },
            default(),
        );
        painter.text(
            default(),
            Align2::LEFT_CENTER,
            "Not Ready",
            FontId {
                size: 7.0,
                family: FontFamily::Name(small_outer_font.clone()),
            },
            default(),
        );
        painter.text(
            default(),
            Align2::LEFT_CENTER,
            "Play Both",
            FontId {
                size: 7.0,
                family: FontFamily::Name(small_inner_font.clone()),
            },
            default(),
        );
        painter.text(
            default(),
            Align2::LEFT_CENTER,
            "Play Both",
            FontId {
                size: 7.0,
                family: FontFamily::Name(small_outer_font.clone()),
            },
            default(),
        );
        painter.text(
            default(),
            Align2::LEFT_CENTER,
            "Ready!",
            FontId {
                size: 7.0,
                family: FontFamily::Name(small_inner_font.clone()),
            },
            default(),
        );
        painter.text(
            default(),
            Align2::LEFT_CENTER,
            "Ready!",
            FontId {
                size: 7.0,
                family: FontFamily::Name(small_outer_font.clone()),
            },
            default(),
        );

        for player_slot in [
            PlayerSlot::A1,
            PlayerSlot::A2,
            PlayerSlot::B1,
            PlayerSlot::B2,
        ] {
            let player_slot_pos = slots.get_player_pos(player_slot).to_array().into();

            // Pad BGs
            let target_size = if self.joins.iter().any(|join| {
                join.is_player_slot(player_slot) && join.is_single()
                    || join.is_player_slot(player_slot.partner()) && join.is_double()
            }) {
                pad_slot_bg.egui_size() * 1.2
            } else {
                pad_slot_bg.egui_size()
            };
            let x = ctx.animate_value_with_time(
                Id::new("pad_bg_size_x").with(player_slot),
                target_size.x,
                0.3,
            );
            let y = ctx.animate_value_with_time(
                Id::new("pad_bg_size_y").with(player_slot),
                target_size.y,
                0.3,
            );
            let animated_size = egui::vec2(x, y);

            pad_slot_bg
                .image_painter()
                .pos(origin + slots.pad_bg_offset.to_array().into())
                .size(animated_size)
                .offset(player_slot_pos)
                .align2(Align2::CENTER_CENTER)
                .paint(&painter, &textures);

            // CPU icons
            let icon_pos = player_slot_pos + slots.cpu_icon_offset.to_array().into();
            let target_x = if self.is_player_slot_empty(player_slot) {
                icon_pos.x
            } else {
                // The standby position, just off-screen.
                match player_slot.team() {
                    Team::A => -(cpu_icon.width() as f32),
                    Team::B => root.screen_size.x + cpu_icon.width() as f32,
                }
            };
            let x =
                ctx.animate_value_with_time(Id::new("cpu_icon_x").with(player_slot), target_x, 0.2);
            let pos = Vec2::new(x, icon_pos.y);

            if let Some(
                Join::Single {
                    partner_setting: PartnerSetting::TwinStick,
                    ..
                }
                | Join::Double {
                    partner_setting: PartnerSetting::TwinStick,
                    ..
                },
            ) = self.get_join_from_slot(player_slot.partner())
            {
            } else {
                cpu_icon
                    .image_painter()
                    .pos(origin + pos)
                    .paint(&painter, &textures);
            }
        }

        // Pads
        for (index, join) in self.joins.iter().enumerate() {
            let player_icon = root.menu.team_select.player_icons()[index];
            let player_slot = join.get_player_slot();
            let center_slot = slots.pad_slots()[index];

            if let Some(player_slot) = player_slot {
                let pad_slot = slots.get_player_pos(player_slot);
                let partner_slot = slots.get_player_pos(player_slot.partner());

                // ready text
                if join.is_single() {
                    let builder = TextPainter::new("Ready!")
                        .size(7.0)
                        .pos(
                            origin
                                + pad_slot.to_array().into()
                                + slots.ready_text_offset.to_array().into(),
                        )
                        .align2(Align2::CENTER_CENTER);
                    builder
                        .clone()
                        .family(small_inner_font.clone())
                        .color(Color32::GREEN)
                        .paint(&painter);
                    builder
                        .clone()
                        .family(small_outer_font.clone())
                        .color(Color32::BLACK)
                        .paint(&painter);
                } else {
                    let builder = TextPainter::new("Not Ready")
                        .size(7.0)
                        .pos(
                            origin
                                + pad_slot.to_array().into()
                                + slots.ready_text_offset.to_array().into(),
                        )
                        .align2(Align2::CENTER_CENTER);
                    builder
                        .clone()
                        .family(small_inner_font.clone())
                        .color(Color32::GRAY)
                        .paint(&painter);
                    builder
                        .clone()
                        .family(small_outer_font.clone())
                        .color(Color32::BLACK)
                        .paint(&painter);
                }
                // twin stick ready text
                if join.is_double() {
                    if join.is_twin_stick() {
                        player_icon.paint_at(
                            origin
                                + partner_slot.to_array().into()
                                + slots.number_icon_offset.to_array().into(),
                            &painter,
                            &textures,
                        );
                    }

                    let builder = TextPainter::new("Ready!")
                        .size(7.0)
                        .pos(
                            origin
                                + partner_slot.to_array().into()
                                + slots.ready_text_offset.to_array().into(),
                        )
                        .align2(Align2::CENTER_CENTER);
                    builder
                        .clone()
                        .family(small_inner_font.clone())
                        .color(Color32::GREEN)
                        .paint(&painter);
                    builder
                        .clone()
                        .family(small_outer_font.clone())
                        .color(Color32::BLACK)
                        .paint(&painter);
                }
                // partner settings

                if let Join::Single {
                    partner_setting, ..
                }
                | Join::Double {
                    partner_setting, ..
                } = join
                {
                    // partner select arrows
                    if self.is_player_slot_set(player_slot)
                        && !self.is_player_slot_double(player_slot)
                        && !self.is_player_slot_hovered(player_slot.partner())
                        && !join.is_source(SingleSource::KeyboardMouse)
                    {
                        let pos = origin + partner_slot.to_array().into();

                        partner_select_arrow
                            .image_painter()
                            .pos(pos)
                            .offset(slots.partner_select_offset_right.to_array().into())
                            .paint(&painter, &textures);

                        partner_select_arrow
                            .image_painter()
                            .uv(Rect::from_min_max(pos2(1.0, 0.0), pos2(0.0, 1.0)))
                            .pos(pos)
                            .offset(slots.partner_select_offset_left.to_array().into())
                            .paint(&painter, &textures);
                    }
                    // play both indicator
                    let target = if self.is_player_slot_set(player_slot)
                        && !self.is_player_slot_hovered(player_slot.partner())
                    {
                        partner_slot.x
                    } else {
                        // The standby position, just off-screen.
                        match player_slot.team() {
                            Team::A => -(controller_icon_silhouette.width() as f32),
                            Team::B => {
                                root.screen_size.x + controller_icon_silhouette.width() as f32
                            }
                        }
                    };
                    let x = ctx.animate_value_with_time(
                        Id::new("play_both_indicator").with(player_slot),
                        target,
                        0.2,
                    );
                    let pos = Vec2::new(x, partner_slot.y);

                    match partner_setting {
                        PartnerSetting::CPU => {}
                        PartnerSetting::TwinStick => {
                            controller_icon_silhouette
                                .image_painter()
                                .pos(origin + pos)
                                .paint(&painter, &textures);
                            if !self.is_player_slot_hovered(player_slot.partner())
                                && !join.is_double()
                            {
                                let builder = TextPainter::new("Play Both")
                                    .size(7.0)
                                    .pos(origin + pos + slots.ready_text_offset.to_array().into())
                                    .align2(Align2::CENTER_CENTER);
                                builder
                                    .clone()
                                    .family(small_inner_font.clone())
                                    .color(Color32::GRAY)
                                    .paint(&painter);
                                builder
                                    .clone()
                                    .family(small_outer_font.clone())
                                    .color(Color32::BLACK)
                                    .paint(&painter);
                            }
                        }
                    }
                }
            }

            // animate now so empty joins are returned to center on removal
            let target = player_slot
                .map(|slot| slots.get_player_pos(slot))
                .unwrap_or(*center_slot);
            let x =
                ctx.animate_value_with_time(Id::new("pad_positions_x").with(index), target.x, 0.2);
            let y =
                ctx.animate_value_with_time(Id::new("pad_positions_y").with(index), target.y, 0.2);
            let animated_offset = vec2(x, y);

            if join.is_joined() {
                let player_icon_offset = slots.number_icon_offset.to_array().into();
                let source_icon = match join.get_source().unwrap() {
                    SingleSource::Gamepad(..) => controller_icon,
                    SingleSource::KeyboardMouse => keyboard_icon,
                    SingleSource::CPU(..) => unreachable!(),
                };

                // faded controller
                let faded_controller_rect = source_icon
                    .image_painter()
                    .pos(origin + center_slot.to_array().into())
                    .tint(Color32::WHITE.gamma_multiply(0.5))
                    .paint(&painter, &textures);

                // faded player number
                player_icon
                    .image_painter()
                    .pos(faded_controller_rect.min + player_icon_offset)
                    .tint(Color32::WHITE.gamma_multiply(0.5))
                    .paint(&painter, &textures);

                // mobile controller
                let source_icon_rect = source_icon
                    .image_painter()
                    .pos(origin + animated_offset)
                    .paint(&painter, &textures);

                // mobile player number
                player_icon
                    .image_painter()
                    .pos(source_icon_rect.min + player_icon_offset)
                    .paint(&painter, &textures);
            }
        }
        // back button
        let asset = asset_server.get(back_btn);
        let inputs = world.resource::<LocalInputs>();

        let back_button_rect = Rect::from_min_size(
            origin + slots.back_btn_offset.to_array().into(),
            asset.tile_size.to_array().into(),
        );
        let pressed = inputs.values().any(|input| input.menu_back.pressed());
        let clicked = ctx.input(|r| {
            r.pointer
                .press_origin()
                .is_some_and(|pos| back_button_rect.contains(pos))
        });

        let mut progress =
            ctx.data_mut(|w| *w.get_temp_mut_or_default::<f32>(Id::new("back_button_progress")));

        if pressed || clicked {
            progress = (progress + 1.0).min(back_buffer as f32);
        } else {
            progress = 0.0;
        };

        ctx.data_mut(|w| {
            *w.get_temp_mut_or_default::<f32>(Id::new("back_button_progress")) = progress
        });

        let progress_fraction = progress / back_buffer as f32;
        let progress_frame = (asset.rows as f32 - 1.0) * progress_fraction;

        let index = progress_frame as usize;

        if progress_fraction >= 1.0 {
            output = Some(TeamSelectOutput::Exit)
        }

        AtlasPainter::new(asset.clone())
            .vertical()
            .index(index)
            .pos(origin + slots.back_btn_offset.to_array().into())
            .paint(&painter, &textures);

        // press start text
        if let Some(players_info) = self.get_player_signs() {
            if world.resource::<Time>().elapsed().as_secs_f32() % 1.0 < 0.5 {
                start.paint_at(
                    origin + slots.start_offset.to_array().into(),
                    &painter,
                    &textures,
                );
            } else {
                start_blink.paint_at(
                    origin + slots.start_offset.to_array().into(),
                    &painter,
                    &textures,
                );
            }
            if ctx.clicked_rect(Rect::from_min_size(
                origin + slots.start_offset.to_array().into(),
                start.egui_size(),
            )) {
                output = Some(TeamSelectOutput::PlayersInfo(players_info));
            }
        }
        output
    }
}
