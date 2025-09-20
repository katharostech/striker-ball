use bones_framework::prelude::*;

/// Returns the points necessary for a [`Path2d::points`] to display
/// a rectangle around the center.
pub fn rect_points(Vec2 { x, y }: Vec2) -> Vec<Vec2> {
    vec![
        Vec2::new(-x, -y),
        Vec2::new(-x, y),
        Vec2::new(x, y),
        Vec2::new(x, -y),
        Vec2::new(-x, -y),
    ]
}
/// Returns the points necessary for a [`Path2d::points`] to display
/// a circle around the center with the chosen amount of lines making it up.
pub fn circle_points(radius: f32, lines: usize) -> Vec<Vec2> {
    let mut vec = Vec::new();
    let start = Vec2::X;
    let rotate = 360. / lines as f32;
    for n in 0..=lines {
        vec.push(start.rotate(Vec2::from_angle((rotate * n as f32).to_radians())) * radius);
    }
    vec
}

#[derive(HasSchema, Clone, Default, Deref, DerefMut)]
pub struct Path2dToggle {
    #[deref]
    /// The true color when using the toggle component.
    pub color: Color,
    /// Whether or not the color should be transparent.
    pub hide: bool,
}
impl Path2dToggle {
    pub fn new(color: Color) -> Self {
        Self {
            color,
            ..Default::default()
        }
    }
    pub fn display_color(&self) -> Color {
        if self.hide {
            Color::NONE
        } else {
            self.color
        }
    }
}
pub struct Path2dTogglePlugin;
impl Path2dTogglePlugin {
    pub fn apply_color(
        entities: Res<Entities>,
        toggles: Comp<Path2dToggle>,
        mut path2ds: CompMut<Path2d>,
    ) {
        for (_entity, (toggle, path2d)) in entities.iter_with((&toggles, &mut path2ds)) {
            path2d.color = toggle.display_color()
        }
    }
}
impl SessionPlugin for Path2dTogglePlugin {
    fn install(self, session: &mut SessionBuilder) {
        session.add_system_to_stage(First, Self::apply_color);
    }
}
