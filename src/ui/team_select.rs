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

pub fn show(world: &World) {
    if !world.resource::<TeamSelect>().visible {
        return;
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
        controller_icon,
        controller_icon_silhouette,
        pad_slot_bg,
        start,
        start_blink,
        back_btn,
        back_buffer,
        ..
    } = root.menu.team_select;

    let team_select = world.resource::<TeamSelect>();
    let small_inner_font = asset_server.get(root.font.small_inner).family_name.clone();
    let small_outer_font = asset_server.get(root.font.small_outer).family_name.clone();

    use egui::*;
    let area = Area::new("team_select_area")
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

    // Pad BGs
    let is_player_id_used = |id: PlayerSlot| -> bool {
        team_select.joins.iter().any(|join| {
            join.is_player_id(id) && join.is_set()
                || join.is_player_id(id.partner()) && join.is_dual_stick()
        })
    };
    let size = pad_slot_bg.egui_size();
    let multiple = 1.2;

    let builder = pad_slot_bg
        .image_painter()
        .size(size)
        .pos(origin + slots.pad_bg_offset.to_array().into())
        .align2(Align2::CENTER_CENTER);

    let xid = Id::new("pad_bg_size_x");
    let yid = Id::new("pad_bg_size_y");
    let animation_time = 0.3;

    // A1
    let player_id = PlayerSlot::A1;
    let target = if is_player_id_used(player_id) {
        size * multiple
    } else {
        size
    };
    let x = ctx
        .0
        .animate_value_with_time(xid.with(player_id), target.x, animation_time);
    let y = ctx
        .0
        .animate_value_with_time(yid.with(player_id), target.y, animation_time);
    builder
        .clone()
        .size(egui::vec2(x, y))
        .offset(slots.a1.to_array().into())
        .paint(&painter, &textures);

    // A2
    let player_id = PlayerSlot::A2;
    let target = if is_player_id_used(player_id) {
        size * multiple
    } else {
        size
    };
    let x = ctx
        .0
        .animate_value_with_time(xid.with(player_id), target.x, animation_time);
    let y = ctx
        .0
        .animate_value_with_time(yid.with(player_id), target.y, animation_time);
    builder
        .clone()
        .size(egui::vec2(x, y))
        .offset(slots.a2.to_array().into())
        .paint(&painter, &textures);

    // B1
    let player_id = PlayerSlot::B1;
    let target = if is_player_id_used(player_id) {
        size * multiple
    } else {
        size
    };
    let x = ctx
        .0
        .animate_value_with_time(xid.with(player_id), target.x, animation_time);
    let y = ctx
        .0
        .animate_value_with_time(yid.with(player_id), target.y, animation_time);
    builder
        .clone()
        .size(egui::vec2(x, y))
        .offset(slots.b1.to_array().into())
        .paint(&painter, &textures);

    // B2
    let player_id = PlayerSlot::B2;
    let target = if is_player_id_used(player_id) {
        size * multiple
    } else {
        size
    };
    let x = ctx
        .0
        .animate_value_with_time(xid.with(player_id), target.x, animation_time);
    let y = ctx
        .0
        .animate_value_with_time(yid.with(player_id), target.y, animation_time);
    builder
        .clone()
        .size(egui::vec2(x, y))
        .offset(slots.b2.to_array().into())
        .paint(&painter, &textures);

    // Pads
    for (index, join) in team_select.joins.iter().enumerate() {
        let player_icon = root.menu.team_select.player_icons()[index];
        let player_slot = join.get_player_slot();
        let center_slot = slots.pad_slots()[index];

        if let Some(player_slot) = player_slot {
            let pad_slot = slots.get_player_pos(player_slot);
            let partner_slot = slots.get_player_pos(player_slot.partner());

            // ready text
            if join.is_ready() {
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
            // dual stick ready text
            if join.is_dual_stick() {
                player_icon.paint_at(
                    origin
                        + partner_slot.to_array().into()
                        + slots.number_icon_offset.to_array().into(),
                    &painter,
                    &textures,
                );

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
            // play both indicator
            let target = if team_select.is_player_id_ready(player_slot)
                && !team_select.is_player_slot_set(player_slot.partner())
            {
                partner_slot.x
            } else {
                match player_slot.team() {
                    Team::A => {
                        area.response.rect.center().x
                            - (controller_icon_silhouette.width() as f32 + root.screen_size.x / 2.)
                    }
                    Team::B => {
                        area.response.rect.center().x
                            + (controller_icon_silhouette.width() as f32 + root.screen_size.x / 2.)
                    }
                }
            };
            let x = ctx.animate_value_with_time(
                Id::new("play_both_indicator").with(player_slot),
                target,
                0.2,
            );
            let pos = Vec2::new(x, partner_slot.y);

            ImagePainter::new(*controller_icon_silhouette)
                .size(controller_icon_silhouette.egui_size())
                .pos(area.response.rect.min + pos)
                .paint(&painter, &textures);

            if team_select.is_player_id_ready(player_slot)
                && !team_select.is_player_slot_dual_stick(player_slot)
                && !team_select.is_player_slot_set(player_slot.partner())
            {
                let builder = TextPainter::new("Play Both")
                    .size(7.0)
                    .pos(origin + (partner_slot + slots.ready_text_offset).to_array().into())
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

        // animate now so empty joins are returned to center on removal
        let target = player_slot
            .map(|slot| slots.get_player_pos(slot))
            .unwrap_or(*center_slot);
        let x = ctx.animate_value_with_time(Id::new("pad_positions_x").with(index), target.x, 0.2);
        let y = ctx.animate_value_with_time(Id::new("pad_positions_y").with(index), target.y, 0.2);

        if join.is_joined() {
            let player_offset = slots.number_icon_offset.to_array().into();

            // faded controller
            let rect = controller_icon
                .image_painter()
                .pos(origin + center_slot.to_array().into())
                .tint(Color32::WHITE.gamma_multiply(0.5))
                .paint(&painter, &textures);

            // faded player number
            player_icon
                .image_painter()
                .pos(rect.min + player_offset)
                .tint(Color32::WHITE.gamma_multiply(0.5))
                .paint(&painter, &textures);

            // mobile controller
            let rect = controller_icon
                .image_painter()
                .pos(origin + vec2(x, y))
                .paint(&painter, &textures);

            // mobile player number
            player_icon
                .image_painter()
                .pos(rect.min + player_offset)
                .paint(&painter, &textures);
        }
    }
    // back button
    let asset = asset_server.get(back_btn);
    let inputs = world.resource::<LocalInputs>();
    let press_input = inputs
        .values()
        .find_map(|btn| btn.west.pressed().then_some(btn.west))
        .unwrap_or_default();
    let cap = back_buffer;
    let frames = if press_input.pressed() {
        press_input.held().min(cap)
    } else {
        0u32
    };
    // one number out of `frames` from 0.0 to 1.0
    let frame_progress = frames as f32 / cap as f32;
    // one number out of `rows` from 0 to `rows`
    let index = (frame_progress * asset.rows as f32 - 1.0).floor() as usize;

    AtlasPainter::new(asset.clone())
        .vertical()
        .index(index)
        .pos(origin + slots.back_btn_offset.to_array().into())
        .paint(&painter, &textures);

    // press start text
    if team_select.get_player_signs().is_some() {
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
    }
}
