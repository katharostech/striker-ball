use super::*;

#[derive(HasSchema, Clone, Default)]
#[repr(C)]
pub struct CreditsAssets {
    pub credits_header: SizedImageAsset,
    pub credits_border: SizedImageAsset,
    pub credits_text: String,
    pub credits_offset: Vec2,
    pub credits_scroll_max: f32,
}

#[derive(HasSchema, Clone, Default)]
pub struct CreditsUi {
    pub visible: bool,
    pub scroll: f32,
}
impl ShowHide for CreditsUi {
    fn show(&mut self) {
        self.visible = true;
        self.scroll = 0.0;
    }
    fn hide(&mut self) {
        self.visible = false
    }
}

impl SessionPlugin for CreditsUi {
    fn install(self, session: &mut SessionBuilder) {
        session.insert_resource(self);
    }
}
fn foreground() -> egui::LayerId {
    use egui::*;
    LayerId::new(Order::Foreground, Id::new("splash_foreground"))
}

pub struct CreditsOutput;

impl CreditsUi {
    pub fn process_input(&mut self, world: &World) -> Option<CreditsOutput> {
        let mut output = None;

        let local_inputs = world.resource::<LocalInputs>();

        let mut delta_scroll: f32 = 0.0;

        for (_source, input) in local_inputs.iter() {
            if input.menu_back.just_pressed() {
                output = Some(CreditsOutput)
            }
            if input.menu_down.pressed() {
                delta_scroll -= 0.6;
            }
            if input.menu_up.pressed() {
                delta_scroll += 0.6;
            }
        }
        self.scroll += delta_scroll.clamp(-0.6, 0.6);

        output
    }

    pub fn process_ui(&mut self, world: &World) -> Option<CreditsOutput> {
        let mut output = None;

        if !self.visible {
            return output;
        }

        let asset_server = world.resource::<AssetServer>();
        let root = asset_server.root::<Data>();
        let textures = world.resource::<EguiTextures>();
        let ctx = world.resource::<EguiCtx>();
        let pointer_navigation = world.resource::<LocalInputs>().pointer_navigation;

        let CreditsAssets {
            credits_header,
            credits_border,
            credits_text,
            credits_offset,
            credits_scroll_max,
        } = &root.menu.credits;

        let inner_font = asset_server
            .get(root.font.primary_inner)
            .family_name
            .clone();
        let inner = TextPainter::standard()
            .size(7.0)
            .family(inner_font)
            .color(Color32::WHITE);

        use egui::*;

        let area = Area::new("credits_bg")
            .anchor(Align2::CENTER_CENTER, [0., 0.])
            .show(&ctx, |ui| {
                ui.image(load::SizedTexture::new(
                    textures.get(**credits_border),
                    root.screen_size.to_array(),
                ));
            });
        let origin = area.response.rect.min;

        let mut painter = ctx.layer_painter(foreground());

        painter.set_clip_rect(area.response.rect);

        let delta_scroll = ctx.input(|r| r.scroll_delta);

        self.scroll += delta_scroll.y / 10.0;
        self.scroll = self.scroll.clamp(-*credits_scroll_max, 0.0);

        credits_header
            .image_painter()
            .pos(origin)
            .offset(pos2(0.0, self.scroll))
            .paint(&painter, &textures);
        inner
            .clone()
            .text(credits_text)
            .pos(origin + credits_offset.to_array().into() + vec2(0.0, self.scroll))
            .paint(&painter);

        credits_border
            .image_painter()
            .pos(origin)
            .paint(&painter, &textures);

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
            output = Some(CreditsOutput);
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
