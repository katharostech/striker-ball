use crate::bones;
use bones::*;
use egui::*;
use std::sync::Arc;

pub fn default_uv() -> egui::Rect {
    use egui::*;
    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0))
}

pub trait EguiCtxExt {
    fn clicked_rect(&self, rect: Rect) -> bool;
    fn hovered_rect(&self, rect: Rect) -> bool;
}
impl EguiCtxExt for egui::Context {
    fn clicked_rect(&self, rect: Rect) -> bool {
        if let Some(pos2) = self.pointer_hover_pos() {
            rect.contains(pos2) && self.input(|r| r.pointer.primary_clicked())
        } else {
            false
        }
    }
    fn hovered_rect(&self, rect: Rect) -> bool {
        if let Some(pos2) = self.pointer_hover_pos() {
            rect.contains(pos2)
        } else {
            false
        }
    }
}

#[derive(Clone)]
pub struct ImagePainter {
    pub image: Handle<bones::Image>,
    pub size: egui::Vec2,
    pub pos: Pos2,
    pub align2: Align2,
    pub tint: Color32,
    pub uv: Rect,
}
impl ImagePainter {
    fn standard() -> Self {
        Self {
            image: Handle::default(),
            size: egui::Vec2::splat(32.0),
            pos: Pos2::ZERO,
            align2: Align2::LEFT_TOP,
            tint: Color32::WHITE,
            uv: Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        }
    }
    pub fn new(image: Handle<bones::Image>) -> Self {
        Self {
            image,
            ..Self::standard()
        }
    }
    pub fn image(self, image: Handle<bones::Image>) -> Self {
        Self { image, ..self }
    }
    pub fn size(self, size: egui::Vec2) -> Self {
        Self { size, ..self }
    }
    pub fn pos(self, pos: Pos2) -> Self {
        Self { pos, ..self }
    }
    pub fn offset(self, pos: Pos2) -> Self {
        Self {
            pos: self.pos + egui::vec2(pos.x, pos.y),
            ..self
        }
    }
    pub fn align2(self, align2: Align2) -> Self {
        Self { align2, ..self }
    }
    pub fn tint(self, tint: Color32) -> Self {
        Self { tint, ..self }
    }
    pub fn uv(self, uv: Rect) -> Self {
        Self { uv, ..self }
    }
    pub fn paint(self, painter: &Painter, textures: &EguiTextures) -> Rect {
        let x = match self.align2.x() {
            Align::Min => self.pos.x + self.size.x / 2.,
            Align::Center => self.pos.x,
            Align::Max => self.pos.x - self.size.x / 2.,
        };
        let y = match self.align2.y() {
            Align::Min => self.pos.y + self.size.y / 2.,
            Align::Center => self.pos.y,
            Align::Max => self.pos.y - self.size.y / 2.,
        };
        let rect = Rect::from_center_size(pos2(x, y), self.size);
        painter.image(textures.get(self.image), rect, self.uv, self.tint);
        rect
    }
}
// TODO: Maybe make this independant of the `Atlas` type and just use the types inside the `Atlas`.
// TODO: Handle animation wrapping.
#[derive(Clone)]
pub struct AtlasPainter {
    pub atlas: Atlas,
    pub index: usize,
    pub horizontal: bool,
    pub size: egui::Vec2,
    pub pos: Pos2,
    pub align2: Align2,
    pub tint: Color32,
}
impl AtlasPainter {
    fn standard() -> Self {
        Self {
            atlas: Atlas::default(),
            index: 0,
            horizontal: true,
            size: egui::Vec2::splat(32.0),
            pos: Pos2::ZERO,
            align2: Align2::LEFT_TOP,
            tint: Color32::WHITE,
        }
    }
    pub fn new(atlas: Atlas) -> Self {
        Self {
            size: atlas.tile_size.to_array().into(),
            atlas,
            ..Self::standard()
        }
    }
    pub fn atlas(self, atlas: Atlas) -> Self {
        Self { atlas, ..self }
    }
    pub fn index(self, index: usize) -> Self {
        Self { index, ..self }
    }
    pub fn horizonal(self) -> Self {
        Self {
            horizontal: true,
            ..self
        }
    }
    pub fn vertical(self) -> Self {
        Self {
            horizontal: false,
            ..self
        }
    }
    pub fn size(self, size: egui::Vec2) -> Self {
        Self { size, ..self }
    }
    pub fn pos(self, pos: Pos2) -> Self {
        Self { pos, ..self }
    }
    pub fn offset(self, pos: Pos2) -> Self {
        Self {
            pos: self.pos + egui::vec2(pos.x, pos.y),
            ..self
        }
    }
    pub fn align2(self, align2: Align2) -> Self {
        Self { align2, ..self }
    }
    pub fn tint(self, tint: Color32) -> Self {
        Self { tint, ..self }
    }
    pub fn paint(self, painter: &Painter, textures: &EguiTextures) -> Rect {
        let Self {
            atlas,
            index,
            horizontal,
            size,
            pos,
            align2,
            tint,
        } = self;

        let uv = if horizontal {
            let index = index.min(atlas.columns as usize);
            let uv_width = 1.0 / atlas.columns as f32;
            Rect::from_min_max(
                pos2(index as f32 * uv_width, 0.0),
                pos2(index as f32 * uv_width + uv_width, 1.0),
            )
        } else {
            let index = index.min(atlas.rows as usize);
            let uv_height = 1.0 / atlas.rows as f32;
            Rect::from_min_max(
                pos2(0.0, index as f32 * uv_height),
                pos2(1.0, index as f32 * uv_height + uv_height),
            )
        };
        ImagePainter::new(atlas.image)
            .size(size)
            .pos(pos)
            .align2(align2)
            .tint(tint)
            .uv(uv)
            .paint(painter, textures)
    }
}
#[derive(Clone)]
pub struct TextPainter {
    pub family: Option<Arc<str>>,
    pub text: String,
    pub size: f32,
    pub pos: Pos2,
    pub align2: Align2,
    pub color: Color32,
}
impl TextPainter {
    pub fn standard() -> Self {
        Self {
            family: None,
            text: "UNSET_TEXT".into(),
            size: 5.0,
            pos: Pos2::ZERO,
            align2: Align2::LEFT_TOP,
            color: Color32::BLACK,
        }
    }
    pub fn new(text: impl ToString) -> Self {
        Self::standard().text(text)
    }
    pub fn text(self, to_string: impl ToString) -> Self {
        Self {
            text: to_string.to_string(),
            ..self
        }
    }
    pub fn size(self, size: f32) -> Self {
        Self { size, ..self }
    }
    pub fn pos(self, pos: Pos2) -> Self {
        Self { pos, ..self }
    }
    pub fn align2(self, align2: Align2) -> Self {
        Self { align2, ..self }
    }
    pub fn color(self, color: Color32) -> Self {
        Self { color, ..self }
    }
    pub fn family(self, name: Arc<str>) -> Self {
        Self {
            family: Some(name),
            ..self
        }
    }
    pub fn paint(self, painter: &Painter) -> Rect {
        let family = if let Some(name) = self.family {
            FontFamily::Name(name)
        } else {
            FontFamily::Proportional
        };
        painter.text(
            self.pos,
            self.align2,
            self.text,
            FontId {
                size: self.size,
                family,
            },
            self.color,
        )
    }
}
