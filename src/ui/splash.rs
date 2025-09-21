use super::*;

#[derive(HasSchema, Clone, Default)]
#[repr(C)]
pub struct SplashAssets {
    pub slots: SplashSlots,
    pub bg: Handle<Image>,
    pub title: SizedImageAsset,
    pub button_bg: SizedImageAsset,
    pub lan: SizedImageAsset,
    pub lan_blink: SizedImageAsset,
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
    pub lan: Vec2,
    pub how_to_play: Vec2,
}

#[derive(HasSchema, Clone, Copy, Default, PartialEq, Eq)]
pub enum SplashState {
    #[default]
    PressGamepad,
    Offline,
    Lan,
    HowToPlay,
}
impl SplashState {
    pub fn cycle_up(&mut self) {
        *self = match self {
            Self::Offline => Self::HowToPlay,
            Self::Lan => Self::Offline,
            Self::HowToPlay => Self::Lan,
            Self::PressGamepad => *self,
        }
    }
    pub fn cycle_down(&mut self) {
        *self = match self {
            Self::Offline => Self::Lan,
            Self::Lan => Self::HowToPlay,
            Self::HowToPlay => Self::Offline,
            Self::PressGamepad => *self,
        }
    }
}

#[derive(HasSchema, Clone, Default, Deref, DerefMut)]
pub struct Splash {
    #[deref]
    pub state: SplashState,
    pub interact: Option<SplashState>,
    pub visual: Visual,
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
    let mut splash = world.resource_mut::<Splash>();

    splash.interact = None;

    if !splash.visual.shown() {
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
        lan,
        lan_blink,
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

    if let SplashState::PressGamepad = splash.state {
        let inner_font = asset_server
            .get(root.font.primary_inner)
            .family_name
            .clone();
        let outer_font = asset_server
            .get(root.font.primary_outer)
            .family_name
            .clone();

        let builder = TextPainter::new("CONNECT GAMEPAD AND PRESS START")
            .family(outer_font)
            .size(7.0)
            .align2(Align2::CENTER_CENTER)
            .pos(area.response.rect.center() + vec2(0., 40.));

        let rect = builder.clone().paint(&painter).expand(6.0);

        painter.add(
            BorderedFrame::new(&root.menu.bframe).paint(textures.get(root.menu.bframe.image), rect),
        );

        if ctx.clicked_rect(rect) {
            splash.interact = Some(SplashState::PressGamepad);
        }

        builder.clone().paint(&painter);
        builder
            .clone()
            .family(inner_font)
            .color(if world.resource::<Time>().elapsed_seconds() % 1.0 > 0.5 {
                Color32::WHITE
            } else {
                Color32::YELLOW
            })
            .paint(&painter);
        return;
    }

    builder
        .clone()
        .image(*button_bg)
        .size(button_bg.egui_size())
        .offset(slots.selection.to_array().into())
        .paint(&painter, &textures);

    let image = if splash.state == SplashState::Offline {
        offline_blink
    } else {
        offline
    };
    let offline_rect = builder
        .clone()
        .image(*image)
        .size(image.egui_size())
        .offset(slots.offline.to_array().into())
        .paint(&painter, &textures)
        .expand(2.0);

    if ctx.clicked_rect(offline_rect) {
        splash.interact = Some(SplashState::Offline);
    }
    if ctx.hovered_rect(offline_rect) {
        splash.state = SplashState::Offline;
    }

    let image = if splash.state == SplashState::HowToPlay {
        how_to_play_blink
    } else {
        how_to_play
    };
    let howtoplay_rect = builder
        .clone()
        .image(*image)
        .size(image.egui_size())
        .offset(slots.how_to_play.to_array().into())
        .paint(&painter, &textures)
        .expand(2.0);

    if ctx.clicked_rect(howtoplay_rect) {
        splash.interact = Some(SplashState::HowToPlay);
    }
    if ctx.hovered_rect(howtoplay_rect) {
        splash.state = SplashState::HowToPlay;
    }

    let image = if splash.state == SplashState::Lan {
        lan_blink
    } else {
        lan
    };
    let lan_rect = builder
        .clone()
        .image(*image)
        .size(image.egui_size())
        .offset(slots.lan.to_array().into())
        .paint(&painter, &textures)
        .expand(2.0);

    if ctx.clicked_rect(lan_rect) {
        splash.interact = Some(SplashState::Lan);
    }
    if ctx.hovered_rect(lan_rect) {
        splash.state = SplashState::Lan;
    }
}
