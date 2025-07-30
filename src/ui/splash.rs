use super::*;

#[derive(HasSchema, Clone, Default)]
#[repr(C)]
pub struct SplashAssets {
    pub slots: SplashSlots,
    pub bg: Handle<Image>,
    pub title: SizedImageAsset,
    pub button_bg: SizedImageAsset,
    pub offline: SizedImageAsset,
    pub offline_blink: SizedImageAsset,
    pub how_to_play: SizedImageAsset,
    pub how_to_play_blink: SizedImageAsset,
}

#[derive(HasSchema, Clone, Copy, Default)]
#[repr(C)]
pub struct SplashSlots {
    pub title: Vec2,
    pub selection: Vec2,
    pub offline: Vec2,
    pub how_to_play: Vec2,
}

#[derive(HasSchema, Clone, Default, PartialEq, Eq)]
pub enum Splash {
    #[default]
    Hidden,
    Offline,
    // Online,
    HowToPlay,
}
impl Splash {
    pub fn cycle_up(&mut self) {
        match self {
            Splash::Offline => *self = /* Self::Online */Self::HowToPlay,
            // Splash::Online => *self = Self::Offline,
            Splash::HowToPlay => *self = Self::Offline,
            Splash::Hidden => {}
        }
    }
    pub fn cycle_down(&mut self) {
        match self {
            Splash::Offline => *self = /* Self::Online */Self::HowToPlay,
            // Splash::Online => *self = Self::HowToPlay,
            Splash::HowToPlay => *self = Self::Offline,
            Splash::Hidden => {}
        }
    }
}
impl SessionPlugin for Splash {
    fn install(self, session: &mut SessionBuilder) {
        session.insert_resource(self);
    }
}
fn foreground() -> egui::LayerId {
    use egui::*;
    LayerId::new(Order::Foreground, Id::new("splash_foreground"))
}
pub fn show(world: &World) {
    let splash = world.resource::<Splash>();
    if Splash::Hidden == *splash {
        return;
    }

    let asset_server = world.resource::<AssetServer>();
    let root = asset_server.root::<Data>();
    let textures = world.resource::<EguiTextures>();
    let ctx = world.resource::<EguiCtx>();

    let SplashAssets {
        slots,
        bg,
        title,
        button_bg,
        offline,
        offline_blink,
        how_to_play,
        how_to_play_blink,
        ..
    } = root.menu.splash;

    use egui::*;

    let area = Area::new("splash")
        .anchor(Align2::CENTER_CENTER, [0., 0.])
        .show(&ctx, |ui| {
            ui.image(load::SizedTexture::new(
                textures.get(bg),
                root.screen_size.to_array(),
            ));
        });
    let mut painter = ctx.layer_painter(foreground());

    painter.set_clip_rect(area.response.rect);

    let builder = ImagePainter::new(*title).pos(area.response.rect.left_top());

    builder
        .clone()
        .image(*title)
        .size(title.egui_size())
        .offset(slots.title.to_array().into())
        .paint(&painter, &textures);

    builder
        .clone()
        .image(*button_bg)
        .size(button_bg.egui_size())
        .offset(slots.selection.to_array().into())
        .paint(&painter, &textures);

    let image = if splash == Splash::Offline {
        offline_blink
    } else {
        offline
    };
    builder
        .clone()
        .image(*image)
        .size(image.egui_size())
        .offset(slots.offline.to_array().into())
        .paint(&painter, &textures);

    let image = if splash == Splash::HowToPlay {
        how_to_play_blink
    } else {
        how_to_play
    };
    builder
        .clone()
        .image(*image)
        .size(image.egui_size())
        .offset(slots.how_to_play.to_array().into())
        .paint(&painter, &textures);
}
