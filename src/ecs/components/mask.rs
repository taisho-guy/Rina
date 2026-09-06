use shipyard::Component;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaskShapeRef {
    pub sides: u32,
    pub center_x: f32,
    pub center_y: f32,
    pub radius_x: f32,
    pub radius_y: f32,
    pub feather: f32,
    pub invert: bool,
}

#[derive(Clone, Debug, Default, Component)]
pub struct MaskStack(pub Vec<MaskShapeRef>);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Component)]
pub enum BlendMode {
    Normal,
    Add,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    Difference,
    Exclusion,
}

impl Default for BlendMode {
    fn default() -> Self {
        Self::Normal
    }
}

impl BlendMode {
    pub fn pipeline_index(self) -> u32 {
        match self {
            BlendMode::Normal => 0,
            BlendMode::Add => 1,
            BlendMode::Multiply => 2,
            BlendMode::Screen => 3,
            BlendMode::Overlay => 4,
            BlendMode::Darken => 5,
            BlendMode::Lighten => 6,
            BlendMode::Difference => 7,
            BlendMode::Exclusion => 8,
        }
    }
}

impl MaskShapeRef {
    fn contains(&self, px: f32, py: f32) -> bool {
        if self.radius_x <= 0.0 || self.radius_y <= 0.0 {
            return false;
        }
        let dx = (px - self.center_x) / self.radius_x;
        let dy = (py - self.center_y) / self.radius_y;
        let inside = dx * dx + dy * dy <= 1.0;
        inside != self.invert
    }

    fn coverage_at(&self, px: f32, py: f32) -> f32 {
        if self.radius_x <= 0.0 || self.radius_y <= 0.0 {
            return 0.0;
        }
        let dx = (px - self.center_x) / self.radius_x;
        let dy = (py - self.center_y) / self.radius_y;
        let dist = (dx * dx + dy * dy).sqrt();
        let feather = (self.feather / self.radius_x.min(self.radius_y)).max(1e-4);
        let edge = (1.0 - dist) / feather;
        let coverage = edge.clamp(0.0, 1.0);
        if self.invert {
            1.0 - coverage
        } else {
            coverage
        }
    }
}

impl MaskStack {
    pub fn opacity_factor_at_origin(&self) -> f32 {
        self.0
            .iter()
            .fold(1.0_f32, |acc, m| acc * m.coverage_at(0.0, 0.0))
    }

    pub fn contains_origin(&self) -> bool {
        self.0.iter().all(|m| m.contains(0.0, 0.0))
    }
}
