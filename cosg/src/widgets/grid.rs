use crate::{
    esc::Esc,
    theme::Theme,
    widget::{DrawRect, Rect, Widget, WidgetEvent},
};

struct Cell {
    col:    usize,
    row:    usize,
    widget: Box<dyn Widget>,
}

pub struct Grid {
    cols:    usize,
    rows:    usize,
    cells:   Vec<Cell>,
    gap:     f32,
    esc:     Esc,
    rect:    Rect,
}

impl Grid {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self { cols, rows, cells: vec![], gap: 8.0, esc: Esc::default(), rect: Rect::default() }
    }

    pub fn gap(mut self, g: f32) -> Self { self.gap = g; self }
    pub fn esc(mut self, e: Esc) -> Self { self.esc = e; self }

    /// Umieść widget w komórce (col, row)
    pub fn place(mut self, widget: impl Widget + 'static, col: usize, row: usize) -> Self {
        self.cells.push(Cell { col, row, widget: Box::new(widget) });
        self
    }
}

impl Widget for Grid {
    fn layout(&mut self, bounds: Rect, theme: &Theme) {
        self.rect = bounds;
        let cell_w = (bounds.w - self.gap * (self.cols + 1) as f32) / self.cols as f32;
        let cell_h = (bounds.h - self.gap * (self.rows + 1) as f32) / self.rows as f32;
        for cell in &mut self.cells {
            let cx = bounds.x + self.gap + cell.col as f32 * (cell_w + self.gap);
            let cy = bounds.y + self.gap + cell.row as f32 * (cell_h + self.gap);
            cell.widget.layout(Rect::new(cx, cy, cell_w, cell_h), theme);
        }
    }

    fn draw(&self, theme: &Theme) -> Vec<DrawRect> {
        let mut out = vec![DrawRect {
            rect:          self.rect,
            color:         theme.bg,
            border_radius: self.esc.border_radius,
            border_width:  self.esc.border_width,
            border_color:  self.esc.border_color.unwrap_or(theme.border),
        }];
        for cell in &self.cells {
            out.extend(cell.widget.draw(theme));
        }
        out
    }

    fn handle_event(&mut self, event: &WidgetEvent) {
        for cell in &mut self.cells {
            cell.widget.handle_event(event);
        }
    }

    fn rect(&self) -> Rect { self.rect }
}
