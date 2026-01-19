use super::*;

#[derive(HasSchema, Clone, Default)]
#[repr(C)]
pub struct HowToPlayAssets {
    pub slots: HowToPlaySlots,
    pub rules: SizedImageAsset,
    pub single_stick: SizedImageAsset,
    pub twin_stick: SizedImageAsset,
    pub keyboard: SizedImageAsset,
    pub left_arrow: SizedImageAsset,
    pub right_arrow: SizedImageAsset,
}
#[derive(HasSchema, Clone, Default)]
#[repr(C)]
pub struct HowToPlaySlots {
    pub primary_header: Vec2,
    pub secondary_header: Vec2,
    pub left_arrow: Vec2,
    pub right_arrow: Vec2,
    pub overview_tl: Vec2,
    pub overview_tr: Vec2,
    pub overview_bl: Vec2,
    pub overview_br: Vec2,
}

#[derive(HasSchema, Clone, Default, PartialEq, Eq)]
pub enum HowToPlay {
    #[default]
    Hidden,
    GameOverview,
    SingleStickControls,
    TwinStickControls,
    KeyboardControls,
}
impl ShowHide for HowToPlay {
    fn show(&mut self) {
        *self = Self::GameOverview
    }
    fn hide(&mut self) {
        *self = Self::Hidden
    }
}
impl HowToPlay {
    pub fn left(&mut self) {
        match self {
            Self::Hidden => {}
            Self::GameOverview => {}
            Self::SingleStickControls => *self = Self::GameOverview,
            Self::TwinStickControls => *self = Self::SingleStickControls,
            Self::KeyboardControls => *self = Self::TwinStickControls,
        }
    }
    pub fn right(&mut self) {
        match self {
            Self::Hidden => {}
            Self::GameOverview => *self = Self::SingleStickControls,
            Self::SingleStickControls => *self = Self::TwinStickControls,
            Self::TwinStickControls => *self = Self::KeyboardControls,
            Self::KeyboardControls => {}
        }
    }
}

impl SessionPlugin for HowToPlay {
    fn install(self, session: &mut SessionBuilder) {
        session.insert_resource(self);
    }
}
fn foreground() -> egui::LayerId {
    use egui::*;
    LayerId::new(Order::Foreground, Id::new("how_to_play_foreground"))
}

#[derive(HasSchema, Clone, Default)]
pub struct HowToPlayOutput;

impl HowToPlay {
    pub fn process_input(&mut self, world: &World) -> Option<HowToPlayOutput> {
        let inputs = world.resource::<LocalInputs>();

        for (_gamepad, input) in inputs.iter() {
            if input.menu_back.just_pressed() {
                return Some(HowToPlayOutput);
            }
            if input.menu_left.just_pressed() {
                self.left();
            }
            if input.menu_right.just_pressed() {
                self.right();
            }
        }
        None
    }
    pub fn process_ui(&mut self, world: &World) -> Option<HowToPlayOutput> {
        let mut output = None;

        if HowToPlay::Hidden == *self {
            return output;
        }

        let textures = world.resource::<EguiTextures>();
        let ctx = world.resource::<EguiCtx>();
        let asset_server = world.resource::<AssetServer>();
        let root = asset_server.root::<Data>();
        let pointer_navigation = world.resource::<LocalInputs>().pointer_navigation;
        let locale = &asset_server.get(root.localization);

        let HowToPlayAssets {
            slots,
            rules,
            single_stick,
            twin_stick,
            keyboard,
            left_arrow,
            right_arrow,
        } = &root.menu.how_to_play;

        let inner_font = asset_server
            .get(root.font.primary_inner)
            .family_name
            .clone();
        let outer_font = asset_server
            .get(root.font.primary_outer)
            .family_name
            .clone();
        let inner = TextPainter::standard()
            .size(7.0)
            .family(inner_font)
            .color(Color32::WHITE);
        let outer = TextPainter::standard().size(7.0).family(outer_font);

        let splash_bg = root.menu.splash.bg;

        use egui::*;

        let area = Area::new("howtoplay")
            .anchor(Align2::CENTER_CENTER, [0., 0.])
            .show(&ctx, |ui| {
                ui.image(load::SizedTexture::new(
                    textures.get(splash_bg),
                    root.screen_size.to_array(),
                ));
            });
        let origin = area.response.rect.min;
        let mut painter = ctx.layer_painter(foreground());

        painter.set_clip_rect(Rect::from_min_size(
            origin,
            root.screen_size.to_array().into(),
        ));

        let target_offset_x = match *self {
            HowToPlay::GameOverview => 0.,
            HowToPlay::SingleStickControls => -root.screen_size.x,
            HowToPlay::TwinStickControls => -root.screen_size.x * 2.,
            HowToPlay::KeyboardControls => -root.screen_size.x * 3.,
            HowToPlay::Hidden => unreachable!(),
        };
        let offset_x =
            ctx.animate_value_with_time(Id::new("howtoplay_offset_x"), target_offset_x, 0.3);
        let offset = vec2(offset_x, 0.);

        rules.paint_at(origin + offset, &painter, &textures);

        let atlas = asset_server.get(root.sprite.ball);
        AtlasPainter::new(atlas.clone())
            .size((atlas.tile_size * 2.).to_array().into())
            .align2(Align2::RIGHT_CENTER)
            .pos(origin + offset + slots.overview_tl.to_array().into())
            .paint(&painter, &textures);
        inner
            .clone()
            .text(locale.get("get-ball"))
            .pos(origin + offset + slots.overview_tl.to_array().into())
            .paint(&painter);
        outer
            .clone()
            .text(locale.get("get-ball"))
            .pos(origin + offset + slots.overview_tl.to_array().into())
            .paint(&painter);

        let atlas = asset_server.get(root.sprite.b_pin);
        AtlasPainter::new(atlas.clone())
            .size((atlas.tile_size * 2.).to_array().into())
            .align2(Align2::RIGHT_CENTER)
            .pos(origin + offset + slots.overview_tr.to_array().into())
            .paint(&painter, &textures);
        let atlas = asset_server.get(root.sprite.a_pin);
        AtlasPainter::new(atlas.clone())
            .size((atlas.tile_size * 2.).to_array().into())
            .align2(Align2::RIGHT_TOP)
            .pos(origin + offset + slots.overview_tr.to_array().into())
            .offset(pos2(0.0, 5.0))
            .paint(&painter, &textures);
        inner
            .clone()
            .text(locale.get("kick-it"))
            .pos(origin + offset + slots.overview_tr.to_array().into())
            .paint(&painter);
        outer
            .clone()
            .text(locale.get("kick-it"))
            .pos(origin + offset + slots.overview_tr.to_array().into())
            .paint(&painter);

        let atlas = asset_server.get(root.sprite.b_pin);
        AtlasPainter::new(atlas.clone())
            .size((atlas.tile_size * 2.).to_array().into())
            .index(2)
            .align2(Align2::RIGHT_CENTER)
            .pos(origin + offset + slots.overview_bl.to_array().into())
            .paint(&painter, &textures);
        inner
            .clone()
            .text(locale.get("play-resets"))
            .pos(origin + offset + slots.overview_bl.to_array().into())
            .paint(&painter);
        outer
            .clone()
            .text(locale.get("play-resets"))
            .pos(origin + offset + slots.overview_bl.to_array().into())
            .paint(&painter);
        ImagePainter::new(root.sprite.aim_cone)
            .size((atlas.tile_size * 2.).to_array().into())
            .align2(Align2::RIGHT_TOP)
            .pos(origin + offset + slots.overview_br.to_array().into())
            .offset(
                (atlas.tile_size * bones::Vec2::new(0.0, -0.5))
                    .to_array()
                    .into(),
            )
            .paint(&painter, &textures);
        inner
            .clone()
            .text(locale.get("get-multiples"))
            .pos(origin + offset + slots.overview_br.to_array().into())
            .paint(&painter);
        outer
            .clone()
            .text(locale.get("get-multiples"))
            .pos(origin + offset + slots.overview_br.to_array().into())
            .paint(&painter);

        single_stick.paint_at(
            origin + offset + vec2(root.screen_size.x, 0.),
            &painter,
            &textures,
        );

        twin_stick.paint_at(
            origin + offset + vec2(root.screen_size.x * 2., 0.),
            &painter,
            &textures,
        );

        keyboard.paint_at(
            origin + offset + vec2(root.screen_size.x * 3.0, 0.),
            &painter,
            &textures,
        );

        // Arrows
        match *self {
            HowToPlay::GameOverview => {
                let right_rect = right_arrow.paint_at(
                    origin + slots.right_arrow.to_array().into(),
                    &painter,
                    &textures,
                );
                if ctx.clicked_rect(right_rect) {
                    self.right();
                }
            }
            HowToPlay::SingleStickControls | HowToPlay::TwinStickControls => {
                let left_rect = left_arrow.paint_at(
                    origin + slots.left_arrow.to_array().into(),
                    &painter,
                    &textures,
                );
                let right_rect = right_arrow.paint_at(
                    origin + slots.right_arrow.to_array().into(),
                    &painter,
                    &textures,
                );
                if ctx.clicked_rect(left_rect) {
                    self.left();
                }
                if ctx.clicked_rect(right_rect) {
                    self.right();
                }
            }
            HowToPlay::KeyboardControls => {
                let left_rect = left_arrow.paint_at(
                    origin + slots.left_arrow.to_array().into(),
                    &painter,
                    &textures,
                );
                if ctx.clicked_rect(left_rect) {
                    self.left();
                }
            }
            HowToPlay::Hidden => unreachable!(),
        }

        let rect = Rect::from_min_size(
            origin + root.menu.back_button_pos.to_array().into(),
            root.menu.back_button.egui_size(),
        );
        let image = if ctx.hovered_rect(rect) && pointer_navigation {
            root.menu.back_button_blink
        } else {
            root.menu.back_button
        };
        if ctx.clicked_rect(rect) {
            output = Some(HowToPlayOutput);
        }
        image
            .image_painter()
            .size(image.egui_size())
            .pos(origin)
            .offset(root.menu.back_button_pos.to_array().into())
            .paint(&painter, &textures);

        output
    }
}
