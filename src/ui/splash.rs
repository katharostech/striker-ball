use super::*;

#[derive(HasSchema, Clone, Default)]
#[repr(C)]
pub struct SplashAssets {
    pub slots: SplashSlots,
    pub bg: Handle<Image>,
    pub title: SizedImageAsset,
    pub button_bg: SizedImageAsset,
    pub button_bg_web: SizedImageAsset,
    pub lan: SizedImageAsset,
    pub lan_blink: SizedImageAsset,
    pub offline: SizedImageAsset,
    pub offline_blink: SizedImageAsset,
    pub how_to_play: SizedImageAsset,
    pub how_to_play_blink: SizedImageAsset,
    pub settings_button: SizedImageAsset,
    pub settings_button_blink: SizedImageAsset,
    pub credits_button: SizedImageAsset,
    pub credits_button_blink: SizedImageAsset,
}

#[derive(HasSchema, Clone, Copy, Default)]
#[repr(C)]
pub struct SplashSlots {
    pub title: f32,
    pub selection: f32,
    pub button_1: f32,
    pub button_2: f32,
    pub button_3: f32,
    pub settings: Vec2,
    pub credits: Vec2,
}

#[derive(HasSchema, Clone, Copy, Default, PartialEq, Eq)]
pub enum SplashState {
    #[default]
    Offline,
    #[cfg(not(target_arch = "wasm32"))]
    Lan,
    HowToPlay,
    Settings,
    Credits,
}
impl SplashState {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn cycle_up(&mut self) {
        *self = match self {
            Self::Offline => Self::HowToPlay,
            Self::Lan => Self::Offline,
            Self::HowToPlay => Self::Lan,
            Self::Settings => Self::HowToPlay,
            Self::Credits => Self::HowToPlay,
        }
    }
    #[cfg(target_arch = "wasm32")]
    pub fn cycle_up(&mut self) {
        *self = match self {
            Self::Offline => Self::HowToPlay,
            Self::HowToPlay => Self::Offline,
            Self::Settings => Self::HowToPlay,
            Self::Credits => Self::HowToPlay,
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn cycle_down(&mut self) {
        *self = match self {
            Self::Offline => Self::Lan,
            Self::Lan => Self::HowToPlay,
            Self::HowToPlay => Self::Offline,
            Self::Settings => Self::Settings,
            Self::Credits => Self::Credits,
        }
    }
    #[cfg(target_arch = "wasm32")]
    pub fn cycle_down(&mut self) {
        *self = match self {
            Self::Offline => Self::HowToPlay,
            Self::HowToPlay => Self::Settings,
            Self::Settings => Self::Settings,
            Self::Credits => Self::Credits,
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn cycle_left(&mut self) {
        *self = match self {
            Self::Offline | Self::Lan | Self::HowToPlay | Self::Credits => Self::Credits,
            Self::Settings => Self::HowToPlay,
        }
    }
    #[cfg(target_arch = "wasm32")]
    pub fn cycle_left(&mut self) {
        *self = match self {
            Self::Offline | Self::HowToPlay | Self::Credits => Self::Credits,
            Self::Settings => Self::HowToPlay,
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn cycle_right(&mut self) {
        *self = match self {
            Self::Offline | Self::Lan | Self::HowToPlay | Self::Settings => Self::Settings,
            Self::Credits => Self::HowToPlay,
        }
    }
    #[cfg(target_arch = "wasm32")]
    pub fn cycle_right(&mut self) {
        *self = match self {
            Self::Offline | Self::HowToPlay | Self::Settings => Self::Settings,
            Self::Credits => Self::HowToPlay,
        }
    }
}

#[derive(HasSchema, Clone, Default, Deref, DerefMut)]
pub struct Splash {
    #[deref]
    pub state: SplashState,
    pub visible: bool,
}
impl ShowHide for Splash {
    fn show(&mut self) {
        self.visible = true
    }
    fn hide(&mut self) {
        self.visible = false
    }
}

pub type SplashOutput = SplashState;

impl SessionPlugin for Splash {
    fn install(self, session: &mut SessionBuilder) {
        session.insert_resource(self);
    }
}
fn foreground() -> egui::LayerId {
    use egui::*;
    LayerId::new(Order::Foreground, Id::new("splash_foreground"))
}
impl Splash {
    pub fn shown() -> Self {
        Self {
            visible: true,
            ..Default::default()
        }
    }
    pub fn process_input(&mut self, world: &World) -> Option<SplashOutput> {
        let mut output = None;

        let inputs = world.resource::<LocalInputs>();

        for (_gamepad, input) in inputs.iter() {
            if input.menu_up.just_pressed() {
                self.cycle_up();
            }
            if input.menu_down.just_pressed() {
                self.cycle_down();
            }
            if input.menu_left.just_pressed() {
                self.cycle_left();
            }
            if input.menu_right.just_pressed() {
                self.cycle_right();
            }
            if input.menu_select.just_pressed() {
                output = Some(self.state);
            }
        }
        output
    }
    pub fn process_ui(&mut self, world: &World) -> Option<SplashOutput> {
        let mut output = None;

        if !self.visible {
            return output;
        }

        let asset_server = world.resource::<AssetServer>();
        let root = asset_server.root::<Data>();
        let textures = world.resource::<EguiTextures>();
        let ctx = world.resource::<EguiCtx>();

        let SplashAssets {
            slots,
            bg,
            title,
            #[cfg(not(target_arch = "wasm32"))]
            button_bg,
            #[cfg(target_arch = "wasm32")]
                button_bg_web: button_bg,
            offline,
            offline_blink,
            how_to_play,
            how_to_play_blink,

            #[cfg(not(target_arch = "wasm32"))]
            lan,
            #[cfg(not(target_arch = "wasm32"))]
            lan_blink,
            settings_button,
            settings_button_blink,
            credits_button,
            credits_button_blink,
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

        let builder = ImagePainter::new(*title)
            .align2(Align2::CENTER_TOP)
            .pos(area.response.rect.center_top());

        builder
            .clone()
            .image(*title)
            .size(title.egui_size())
            .offset(pos2(0.0, slots.title))
            .paint(&painter, &textures);

        builder
            .clone()
            .image(*button_bg)
            .size(button_bg.egui_size())
            .offset(pos2(0.0, slots.selection))
            .paint(&painter, &textures);

        let image = if self.state == SplashState::Offline {
            offline_blink
        } else {
            offline
        };
        let offline_rect = builder
            .clone()
            .image(*image)
            .size(image.egui_size())
            .offset(pos2(0.0, slots.button_1))
            .paint(&painter, &textures)
            .expand(2.0);

        if ctx.clicked_rect(offline_rect) {
            output = Some(SplashState::Offline);
        }
        if ctx.hovered_rect(offline_rect) {
            self.state = SplashState::Offline;
        }

        let image = if self.state == SplashState::HowToPlay {
            how_to_play_blink
        } else {
            how_to_play
        };
        #[cfg(target_arch = "wasm32")]
        let offset = slots.button_2;
        #[cfg(not(target_arch = "wasm32"))]
        let offset = slots.button_3;

        let howtoplay_rect = builder
            .clone()
            .image(*image)
            .size(image.egui_size())
            .offset(pos2(0.0, offset))
            .paint(&painter, &textures)
            .expand(2.0);

        if ctx.clicked_rect(howtoplay_rect) {
            output = Some(SplashState::HowToPlay);
        }
        if ctx.hovered_rect(howtoplay_rect) {
            self.state = SplashState::HowToPlay;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let image = if self.state == SplashState::Lan {
                lan_blink
            } else {
                lan
            };
            let lan_rect = builder
                .clone()
                .image(*image)
                .size(image.egui_size())
                .offset(pos2(0.0, slots.button_2))
                .paint(&painter, &textures)
                .expand(2.0);

            if ctx.clicked_rect(lan_rect) {
                output = Some(SplashState::Lan);
            }
            if ctx.hovered_rect(lan_rect) {
                self.state = SplashState::Lan;
            }
        }

        let image = if self.state == SplashState::Settings {
            settings_button_blink
        } else {
            settings_button
        };
        let settings_rect = image
            .image_painter()
            .size(image.egui_size())
            .pos(area.response.rect.min)
            .offset(slots.settings.to_array().into())
            .paint(&painter, &textures);

        if ctx.clicked_rect(settings_rect) {
            output = Some(SplashState::Settings);
        }
        if ctx.hovered_rect(settings_rect) {
            self.state = SplashState::Settings;
        }

        let image = if self.state == SplashState::Credits {
            credits_button_blink
        } else {
            credits_button
        };
        let credits_rect = image
            .image_painter()
            .size(image.egui_size())
            .pos(area.response.rect.min)
            .offset(slots.credits.to_array().into())
            .paint(&painter, &textures);

        if ctx.clicked_rect(credits_rect) {
            output = Some(SplashState::Credits);
        }
        if ctx.hovered_rect(credits_rect) {
            self.state = SplashState::Credits;
        }

        output
    }
}
