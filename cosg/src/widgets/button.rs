use crate::{
    esc::Esc,
    theme::Theme,
    widget::{DrawRect, Rect, Widget, WidgetEvent},
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum BtnState { Idle, Hovered, Pressed }

pub struct Button {
    label:    String,
    esc:      Esc,
    rect:     Rect,
    state:    BtnState,
    on_press: Option<Box<dyn Fn()>>,
}

impl Button {
    pub fn new() -> Self {
        Self {
            label:    String::new(),
            esc:      Esc::default(),
            rect:     Rect::default(),
            state:    BtnState::Idle,
            on_press: None,
        }
    }

    pub fn label(mut self, s: impl Into<String>) -> Self { self.label = s.into(); self }
    pub fn esc(mut self, e: Esc) -> Self { self.esc = e; self }

    pub fn on_press<F: Fn() + 'static>(mut self, f: F) -> Self {
        self.on_press = Some(Box::new(f));
        self
    }

    pub fn text(&self) -> &str { &self.label }
}

impl Widget for Button {
    fn layout(&mut self, bounds: Rect, _theme: &Theme) {
        self.rect = bounds;
    }

    fn draw(&self, theme: &Theme) -> Vec<DrawRect> {
        let color = match self.state {
            BtnState::Idle    => theme.primary,
            BtnState::Hovered => theme.primary_hover,
            BtnState::Pressed => theme.primary_press,
        };

        let mut color = color;
        color[3] *= self.esc.opacity;

        vec![DrawRect {
            rect:         self.rect,
            color,
            border_radius: self.esc.border_radius,
            border_width:  self.esc.border_width,
            border_color:  self.esc.border_color.unwrap_or(theme.border),
        }]
    }

    fn handle_event(&mut self, event: &WidgetEvent) {
        match event {
            WidgetEvent::MouseMove { x, y } => {
                if self.rect.contains(*x, *y) {
                    if self.state == BtnState::Idle { self.state = BtnState::Hovered; }
                } else {
                    self.state = BtnState::Idle;
                }
            }
            WidgetEvent::MouseDown { x, y } => {
                if self.rect.contains(*x, *y) { self.state = BtnState::Pressed; }
            }
            WidgetEvent::MouseUp { x, y } => {
                if self.rect.contains(*x, *y) && self.state == BtnState::Pressed {
                    if let Some(f) = &self.on_press { f(); }
                    self.state = BtnState::Hovered;
                } else {
                    self.state = BtnState::Idle;
                }
            }
        }
    }

    fn rect(&self) -> Rect { self.rect }
}
