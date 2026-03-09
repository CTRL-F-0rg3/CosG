use crate::{
    esc::Esc,
    theme::Theme,
    widget::{DrawRect, Rect, TextCmd, Widget, WidgetEvent},
};

pub struct Label {
    text:  String,
    esc:   Esc,
    rect:  Rect,
    small: bool,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into(), esc: Esc::default(), rect: Rect::default(), small: false }
    }
    pub fn small(mut self) -> Self { self.small = true; self }
    pub fn esc(mut self, e: Esc) -> Self { self.esc = e; self }
    pub fn text(&self) -> &str { &self.text }
    pub fn font_size(&self, theme: &Theme) -> f32 {
        if self.small { theme.font_size_sm } else { theme.font_size }
    }
}

impl Widget for Label {
    fn layout(&mut self, bounds: Rect, _theme: &Theme) { self.rect = bounds; }

    fn draw(&self, _theme: &Theme) -> Vec<DrawRect> { vec![] }

    fn draw_text(&self, theme: &Theme) -> Vec<TextCmd> {
        vec![TextCmd {
            text:      self.text.clone(),
            x:         self.rect.x,
            y:         self.rect.y + self.font_size(theme) * 0.2, // małe wyrównanie baseline
            font_size: self.font_size(theme),
            color:     theme.text,
        }]
    }

    fn handle_event(&mut self, _event: &WidgetEvent) {}
    fn rect(&self) -> Rect { self.rect }
}