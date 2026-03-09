/// Fioletowy ciemny motyw domyślny CosG
#[derive(Debug, Clone)]
pub struct Theme {
    pub bg:             [f32; 4],
    pub surface:        [f32; 4],
    pub surface_raised: [f32; 4],
    pub primary:        [f32; 4],
    pub primary_hover:  [f32; 4],
    pub primary_press:  [f32; 4],
    pub text:           [f32; 4],
    pub text_muted:     [f32; 4],
    pub border:         [f32; 4],
    pub font_size:      f32,
    pub font_size_sm:   f32,
}

impl Theme {
    pub fn violet_dark() -> Self {
        Self {
            bg:             hex(0x0D0B14FF),
            surface:        hex(0x16132AFF),
            surface_raised: hex(0x1F1B38FF),
            primary:        hex(0x7C3AEDFF),
            primary_hover:  hex(0x6D28D9FF),
            primary_press:  hex(0x5B21B6FF),
            text:           hex(0xF3F0FFFF),
            text_muted:     hex(0x9D8FC2FF),
            border:         hex(0x3B2F6EFF),
            font_size:      16.0,
            font_size_sm:   12.0,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::violet_dark()
    }
}

fn hex(c: u32) -> [f32; 4] {
    [
        ((c >> 24) & 0xFF) as f32 / 255.0,
        ((c >> 16) & 0xFF) as f32 / 255.0,
        ((c >>  8) & 0xFF) as f32 / 255.0,
        ( c        & 0xFF) as f32 / 255.0,
    ]
}
