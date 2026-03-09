use crate::{
    esc::Esc,
    theme::Theme,
    widget::{DrawRect, Rect, TextCmd, Widget, WidgetEvent},
};

pub struct Container {
    children: Vec<Box<dyn Widget>>,
    esc:      Esc,
    padding:  f32,
    rect:     Rect,
}

impl Container {
    pub fn new() -> Self {
        Self { children: vec![], esc: Esc::default(), padding: 8.0, rect: Rect::default() }
    }
    pub fn padding(mut self, p: f32) -> Self { self.padding = p; self }
    pub fn esc(mut self, e: Esc) -> Self { self.esc = e; self }
    pub fn add(mut self, widget: impl Widget + 'static) -> Self {
        self.children.push(Box::new(widget)); self
    }
}

impl Widget for Container {
    fn layout(&mut self, bounds: Rect, theme: &Theme) {
        self.rect = bounds;
        let inner = Rect {
            x: bounds.x + self.padding,
            y: bounds.y + self.padding,
            w: bounds.w - self.padding * 2.0,
            h: bounds.h - self.padding * 2.0,
        };
        let child_h = if self.children.is_empty() { 0.0 } else {
            (inner.h - self.padding * (self.children.len() - 1) as f32)
                / self.children.len() as f32
        };
        let mut cy = inner.y;
        for child in &mut self.children {
            child.layout(Rect::new(inner.x, cy, inner.w, child_h), theme);
            cy += child_h + self.padding;
        }
    }

    fn draw(&self, theme: &Theme) -> Vec<DrawRect> {
        let mut out = vec![DrawRect {
            rect:          self.rect,
            color:         theme.surface,
            border_radius: self.esc.border_radius,
            border_width:  self.esc.border_width,
            border_color:  self.esc.border_color.unwrap_or(theme.border),
        }];
        for child in &self.children { out.extend(child.draw(theme)); }
        out
    }

    fn draw_text(&self, theme: &Theme) -> Vec<TextCmd> {
        self.children.iter().flat_map(|c| c.draw_text(theme)).collect()
    }

    fn handle_event(&mut self, event: &WidgetEvent) {
        for child in &mut self.children { child.handle_event(event); }
    }

    fn rect(&self) -> Rect { self.rect }
}