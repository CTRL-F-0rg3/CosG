/// ESC (Extra Style Controls) — opcjonalne dodatkowe właściwości wizualne
/// Niezależny od motywu, nałożony na widget jako warstwa nadpisująca
#[derive(Debug, Clone)]
pub struct Esc {
    pub border_radius: f32,
    pub shadow_size:   f32,
    pub shadow_color:  Option<[f32; 4]>,
    pub opacity:       f32,
    pub border_width:  f32,
    pub border_color:  Option<[f32; 4]>,
}

impl Esc {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn border_radius(mut self, v: f32) -> Self { self.border_radius = v; self }
    pub fn shadow(mut self, v: f32) -> Self { self.shadow_size = v; self }
    pub fn shadow_color(mut self, c: [f32; 4]) -> Self { self.shadow_color = Some(c); self }
    pub fn opacity(mut self, v: f32) -> Self { self.opacity = v; self }
    pub fn border_width(mut self, v: f32) -> Self { self.border_width = v; self }
    pub fn border_color(mut self, c: [f32; 4]) -> Self { self.border_color = Some(c); self }
}

impl Default for Esc {
    fn default() -> Self {
        Self {
            border_radius: 6.0,
            shadow_size:   0.0,
            shadow_color:  None,
            opacity:       1.0,
            border_width:  0.0,
            border_color:  None,
        }
    }
}
