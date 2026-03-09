use crate::theme::Theme;

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

#[derive(Debug, Clone)]
pub struct DrawRect {
    pub rect:         Rect,
    pub color:        [f32; 4],
    pub border_radius: f32,
    pub border_width:  f32,
    pub border_color:  [f32; 4],
}

/// Polecenie rysowania tekstu zbierane przez renderer i przekazywane do glyphon
#[derive(Debug, Clone)]
pub struct TextCmd {
    pub text:      String,
    pub x:         f32,
    pub y:         f32,
    pub font_size: f32,
    pub color:     [f32; 4],
}

#[derive(Debug, Clone)]
pub enum WidgetEvent {
    MouseMove { x: f32, y: f32 },
    MouseDown { x: f32, y: f32 },
    MouseUp   { x: f32, y: f32 },
    KeyInput  { ch: char },
}

pub trait Widget {
    fn layout(&mut self, bounds: Rect, theme: &Theme);
    fn draw(&self, theme: &Theme) -> Vec<DrawRect>;
    fn draw_text(&self, _theme: &Theme) -> Vec<TextCmd> { vec![] }
    fn handle_event(&mut self, event: &WidgetEvent);
    fn rect(&self) -> Rect;
}