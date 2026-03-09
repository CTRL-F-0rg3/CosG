use crate::theme::Theme;

/// Prostokąt w przestrzeni ekranu (px)
#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self { Self { x, y, w, h } }
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.w &&
        py >= self.y && py <= self.y + self.h
    }
}

/// Dane do wyrenderowania jednego prostokąta (wysyłane do GPU)
#[derive(Debug, Clone)]
pub struct DrawRect {
    pub rect:          Rect,
    pub color:         [f32; 4],
    pub border_radius: f32,
    pub border_width:  f32,
    pub border_color:  [f32; 4],
}

/// Zdarzenia które widget może odebrać
#[derive(Debug, Clone)]
pub enum WidgetEvent {
    MouseMove { x: f32, y: f32 },
    MouseDown { x: f32, y: f32 },
    MouseUp   { x: f32, y: f32 },
}

/// Każdy widget musi implementować ten trait
pub trait Widget {
    /// Oblicz rozmiar i pozycję (wywołane przez rodzica / grid / container)
    fn layout(&mut self, bounds: Rect, theme: &Theme);
    /// Zwróć listę prostokątów do narysowania
    fn draw(&self, theme: &Theme) -> Vec<DrawRect>;
    /// Obsłuż zdarzenie wejścia
    fn handle_event(&mut self, event: &WidgetEvent);
    /// Aktualny obszar zajmowany przez widget
    fn rect(&self) -> Rect;
}
