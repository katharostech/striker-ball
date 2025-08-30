use super::*;

#[derive(HasSchema, Clone, Default)]
#[type_data(metadata_asset("game"))]
#[repr(C)]
pub struct Data {
    pub matchmaking_server: String,
    pub localization: Handle<LocalizationAsset>,
    pub screen_size: Vec2,
    pub constant: Constants,
    pub sprite: Sprites,
    pub sound: Sounds,
    pub font: Fonts,
    pub menu: Menus,
    pub court: SizedImageAsset,
}

#[derive(HasSchema, Clone, Default)]
#[repr(C)]
pub struct Fonts {
    pub primary_inner: Handle<Font>,
    pub primary_outer: Handle<Font>,
    pub small_outer: Handle<Font>,
    pub small_inner: Handle<Font>,
}

#[derive(HasSchema, Clone, Default)]
#[repr(C)]
pub struct Sounds {
    pub menu_music: VolumeSoundAsset,
    pub countdown_first: VolumeSoundAsset,
    pub countdown_final: VolumeSoundAsset,
    pub winner: VolumeSoundAsset,
    pub pin_explosion: VolumeSoundAsset,
    pub ball_spin: VolumeSoundAsset,
    pub ball_spin_buffer: f32,
    pub ball_bounced: VolumeSoundAsset,
    pub ball_kicked: VolumeSoundAsset,
    pub player_tackle: VolumeSoundAsset,
    pub player_tackled: VolumeSoundAsset,
}

#[derive(HasSchema, Clone, Default)]
#[repr(C)]
pub struct Constants {
    pub ball_bounds: Vec2,
    pub player_bounds: Vec2,
    pub pin_count: usize,
    pub pin_padding: Vec2,

    pub dribble_speed: f32,
    pub run_speed: f32,
    pub tackle_speed: f32,
    pub tackle_friction: f32,
    pub kick_power: f32,
    pub player_radius: f32,

    pub kick_frames: u64,
    pub tackle_frames: u64,
    pub tackled_frames: u64,
    pub pass_frames: u64,
    pub recieve_frames: u64,
    pub turn_frames: u64,
    pub dribble_smoothing: f32,
    pub dribble_smoothing_threshold: f32,

    pub ball_radius: f32,
    pub ball_friction: f32,
    pub ball_etransfer: f32,
    pub ball_border_slide: f32,

    pub pin_radius: f32,
}

#[derive(HasSchema, Clone, Default)]
#[repr(C)]
pub struct Sprites {
    pub ball: Handle<Atlas>,

    pub player_a: Handle<Atlas>,
    pub player_b: Handle<Atlas>,
    pub player_a2: Handle<Atlas>,
    pub player_b2: Handle<Atlas>,
    pub player_animations: Handle<AnimationBankAsset>,
    pub lstick_indicator: Handle<Image>,
    pub rstick_indicator: Handle<Image>,
    pub p1_shadow: Handle<Image>,
    pub p2_shadow: Handle<Image>,
    pub p3_shadow: Handle<Image>,
    pub p4_shadow: Handle<Image>,
    pub aim_cone: Handle<Image>,
    pub aim_arrow: Handle<Image>,

    pub a_pin: Handle<Atlas>,
    pub b_pin: Handle<Atlas>,
}
impl Sprites {
    pub fn player_shadows(&self) -> [Handle<Image>; 4] {
        [
            self.p1_shadow,
            self.p2_shadow,
            self.p3_shadow,
            self.p4_shadow,
        ]
    }
}

#[derive(HasSchema, Clone, Default)]
#[repr(C)]
pub struct Menus {
    pub numbers: Handle<Atlas>,
    pub bframe: BorderImageMeta,
    pub score_bg: SizedImageAsset,
    pub go_text: SizedImageAsset,
    pub countdown_bg: SizedImageAsset,
    pub winner_banner: WinnerBannerAssets,
    pub splash: SplashAssets,
    pub how_to_play: HowToPlayAssets,
    pub team_select: TeamSelectAssets,
    pub match_done: MatchDoneAssets,
    pub pause: PauseAssets,
}

#[derive(HasSchema, Clone, Copy, Default)]
#[repr(C)]
pub struct ImageSize(pub u32, pub u32);
impl ImageSize {
    pub fn width(&self) -> u32 {
        self.0
    }
    pub fn height(&self) -> u32 {
        self.1
    }
    pub fn egui_size(&self) -> egui::Vec2 {
        egui::Vec2::new(self.width() as f32, self.height() as f32)
    }
}

#[derive(HasSchema, Clone, Copy, Default)]
#[repr(C)]
pub struct StaticImageAsset {
    pub image: Handle<Image>,
    pub size: ImageSize,
    pub pos: Vec2,
}
impl StaticImageAsset {
    pub fn width(&self) -> u32 {
        self.size.width()
    }
    pub fn height(&self) -> u32 {
        self.size.height()
    }
    pub fn egui_size(&self) -> egui::Vec2 {
        self.size.egui_size()
    }
}

#[derive(HasSchema, Clone, Copy, Default)]
#[repr(C)]
pub struct SizedImageAsset(pub Handle<Image>, pub u32, pub u32);
impl SizedImageAsset {
    pub fn image(&self) -> Handle<Image> {
        self.0
    }
    pub fn width(&self) -> u32 {
        self.1
    }
    pub fn height(&self) -> u32 {
        self.2
    }
    pub fn size(&self) -> bones::Vec2 {
        bones::Vec2::new(self.1 as f32, self.2 as f32)
    }
    pub fn egui_size(&self) -> egui::Vec2 {
        egui::Vec2::new(self.1 as f32, self.2 as f32)
    }
    pub fn sized_texture(&self, textures: &EguiTextures) -> egui::load::SizedTexture {
        egui::load::SizedTexture::new(textures.get(self.image()), self.egui_size())
    }
    /// Returns an [`ImagePainter`] with the handle and size of `self`.
    pub fn image_painter(&self) -> crate::ImagePainter {
        crate::ImagePainter::new(self.image()).size(self.egui_size())
    }
    pub fn paint_at(
        &self,
        min: egui::Pos2,
        painter: &egui::Painter,
        textures: &EguiTextures,
    ) -> egui::Rect {
        let rect = egui::Rect::from_min_size(min, self.egui_size());
        painter.image(
            textures.get(self.image()),
            rect,
            crate::default_uv(),
            egui::Color32::WHITE,
        );
        rect
    }
}
impl std::ops::Deref for SizedImageAsset {
    type Target = Handle<Image>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(HasSchema, Clone, Copy, Default)]
#[repr(C)]
pub struct VolumeSoundAsset(pub Handle<AudioSource>, pub f64);
impl VolumeSoundAsset {
    pub fn volume(&self) -> f64 {
        self.1
    }
}
impl std::ops::Deref for VolumeSoundAsset {
    type Target = Handle<AudioSource>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(HasSchema, Clone, Default)]
#[type_data(metadata_asset("animations"))]
#[repr(C)]
pub struct AnimationBankAsset(SMap<Ustr, AnimationAsset>);

#[derive(HasSchema, Clone, Default)]
#[repr(C)]
pub struct AnimationAsset {
    pub frames: SVec<u32>,
    pub fps: f32,
    pub repeat: bool,
}

impl AnimationBankAsset {
    pub fn to_bank(&self, current: Ustr) -> AnimationBankSprite {
        AnimationBankSprite {
            current,
            last_animation: current,
            animations: self
                .0
                .iter()
                .map(|(key, anim)| {
                    (
                        *key,
                        AnimatedSprite {
                            frames: anim.frames.clone(),
                            fps: anim.fps,
                            repeat: anim.repeat,
                            ..Default::default()
                        },
                    )
                })
                .collect(),
        }
    }
}
