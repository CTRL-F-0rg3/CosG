use bytemuck::{Pod, Zeroable};
use glyphon::{
    Attrs, Buffer as GBuffer, Cache, Color as GColor, Family, FontSystem,
    Metrics, Resolution, Shaping, SwashCache, TextArea, TextAtlas,
    TextBounds, TextRenderer as GTextRenderer, Viewport,
};
use crate::widget::{DrawRect, TextCmd};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct RectVertex {
    pos:           [f32; 2],
    color:         [f32; 4],
    border_radius: f32,
    rect_min:      [f32; 2],
    rect_max:      [f32; 2],
}

pub struct Renderer {
    // rect pipeline
    pipeline:    wgpu::RenderPipeline,
    vertex_buf:  wgpu::Buffer,
    screen_size: (u32, u32),
    // text
    font_system:   FontSystem,
    swash_cache:   SwashCache,
    text_atlas:    TextAtlas,
    text_renderer: GTextRenderer,
    viewport:      Viewport,
}

impl Renderer {
    pub fn new(
        device:      &wgpu::Device,
        queue:       &wgpu::Queue,
        format:      wgpu::TextureFormat,
        screen_size: (u32, u32),
    ) -> Self {
        // ── rect pipeline ────────────────────────────────────────────────────
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label:  Some("cosg_rect"),
            source: wgpu::ShaderSource::Wgsl(RECT_SHADER.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label:                Some("cosg_layout"),
            bind_group_layouts:   &[],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label:  Some("cosg_rect_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module:      &shader,
                entry_point: "vs_main",
                buffers:     &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<RectVertex>() as u64,
                    step_mode:    wgpu::VertexStepMode::Vertex,
                    attributes:   &wgpu::vertex_attr_array![
                        0 => Float32x2,
                        1 => Float32x4,
                        2 => Float32,
                        3 => Float32x2,
                        4 => Float32x2,
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module:      &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend:      Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive:     wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample:   wgpu::MultisampleState::default(),
            multiview:     None,
            cache:         None,
        });

        let vertex_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label:              Some("cosg_vbuf"),
            size:               std::mem::size_of::<RectVertex>() as u64 * 4096,
            usage:              wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // ── glyphon ──────────────────────────────────────────────────────────
        let font_system  = FontSystem::new();
        let swash_cache  = SwashCache::new();
        let cache        = Cache::new(device);
        let viewport     = Viewport::new(device, &cache);
        let mut text_atlas = TextAtlas::new(device, queue, &cache, format);
        let text_renderer  = GTextRenderer::new(
            &mut text_atlas, device,
            wgpu::MultisampleState::default(), None,
        );

        Self {
            pipeline, vertex_buf, screen_size,
            font_system, swash_cache, text_atlas, text_renderer, viewport,
        }
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        self.screen_size = (w, h);
    }

    pub fn render(
        &mut self,
        device:    &wgpu::Device,
        queue:     &wgpu::Queue,
        encoder:   &mut wgpu::CommandEncoder,
        view:      &wgpu::TextureView,
        rects:     &[DrawRect],
        text_cmds: &[TextCmd],
        bg:        [f32; 4],
    ) {
        let (sw, sh) = (self.screen_size.0 as f32, self.screen_size.1 as f32);

        // ── build rect vertices ──────────────────────────────────────────────
        let mut verts: Vec<RectVertex> = Vec::with_capacity(rects.len() * 6);
        for dr in rects {
            let r = &dr.rect;
            let ndc = |x: f32, y: f32| -> [f32; 2] {
                [(x / sw) * 2.0 - 1.0, 1.0 - (y / sh) * 2.0]
            };
            let tl = ndc(r.x,       r.y);
            let tr = ndc(r.x + r.w, r.y);
            let bl = ndc(r.x,       r.y + r.h);
            let br = ndc(r.x + r.w, r.y + r.h);
            let rm = [r.x,       r.y];
            let rx = [r.x + r.w, r.y + r.h];
            let v  = |pos: [f32; 2]| RectVertex {
                pos, color: dr.color,
                border_radius: dr.border_radius,
                rect_min: rm, rect_max: rx,
            };
            verts.extend_from_slice(&[v(tl), v(tr), v(bl), v(tr), v(br), v(bl)]);
        }
        if !verts.is_empty() {
            queue.write_buffer(&self.vertex_buf, 0, bytemuck::cast_slice(&verts));
        }

        // ── prepare glyphon buffers ──────────────────────────────────────────
        self.viewport.update(queue, Resolution {
            width:  self.screen_size.0,
            height: self.screen_size.1,
        });

        // Zbuduj glyphon Buffer dla każdego TextCmd
        let mut gbuffers: Vec<GBuffer> = text_cmds.iter().map(|cmd| {
            let mut buf = GBuffer::new(
                &mut self.font_system,
                Metrics::new(cmd.font_size, cmd.font_size * 1.2),
            );
            buf.set_size(&mut self.font_system, Some(sw), Some(sh));
            buf.set_text(
                &mut self.font_system,
                &cmd.text,
                Attrs::new().family(Family::SansSerif),
                Shaping::Advanced,
            );
            buf
        }).collect();

        let text_areas: Vec<TextArea> = text_cmds.iter().zip(gbuffers.iter()).map(|(cmd, buf)| {
            let c = cmd.color;
            TextArea {
                buffer:          buf,
                left:            cmd.x,
                top:             cmd.y,
                scale:           1.0,
                bounds:          TextBounds { left: 0, top: 0, right: sw as i32, bottom: sh as i32 },
                default_color:   GColor::rgba(
                    (c[0] * 255.0) as u8,
                    (c[1] * 255.0) as u8,
                    (c[2] * 255.0) as u8,
                    (c[3] * 255.0) as u8,
                ),
                custom_glyphs: &[],
            }
        }).collect();

        self.text_renderer.prepare(
            device, queue,
            &mut self.font_system,
            &mut self.text_atlas,
            &self.viewport,
            text_areas,
            &mut self.swash_cache,
        ).unwrap();

        // ── render pass ──────────────────────────────────────────────────────
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("cosg_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load:  wgpu::LoadOp::Clear(wgpu::Color {
                        r: bg[0] as f64, g: bg[1] as f64,
                        b: bg[2] as f64, a: bg[3] as f64,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes:         None,
            occlusion_query_set:      None,
        });

        // rects najpierw
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
        if !verts.is_empty() {
            pass.draw(0..verts.len() as u32, 0..1);
        }

        // tekst na wierzchu
        if !text_cmds.is_empty() {
            self.text_renderer.render(&self.text_atlas, &self.viewport, &mut pass).unwrap();
        }
    }
}

// ── WGSL ─────────────────────────────────────────────────────────────────────

const RECT_SHADER: &str = r#"
struct Vert {
    @location(0) pos:           vec2<f32>,
    @location(1) color:         vec4<f32>,
    @location(2) border_radius: f32,
    @location(3) rect_min:      vec2<f32>,
    @location(4) rect_max:      vec2<f32>,
}
struct Frag {
    @builtin(position) clip_pos:      vec4<f32>,
    @location(0)       color:         vec4<f32>,
    @location(1)       border_radius: f32,
    @location(2)       rect_min:      vec2<f32>,
    @location(3)       rect_max:      vec2<f32>,
}
@vertex
fn vs_main(v: Vert) -> Frag {
    var out: Frag;
    out.clip_pos      = vec4<f32>(v.pos, 0.0, 1.0);
    out.color         = v.color;
    out.border_radius = v.border_radius;
    out.rect_min      = v.rect_min;
    out.rect_max      = v.rect_max;
    return out;
}
fn sdf_rounded_box(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b + vec2<f32>(r, r);
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
}
@fragment
fn fs_main(f: Frag) -> @location(0) vec4<f32> {
    let center = (f.rect_min + f.rect_max) * 0.5;
    let half   = (f.rect_max - f.rect_min) * 0.5;
    let p      = f.clip_pos.xy - center;
    let d      = sdf_rounded_box(p, half, f.border_radius);
    let alpha  = 1.0 - smoothstep(-1.0, 0.5, d);
    return vec4<f32>(f.color.rgb, f.color.a * alpha);
}
"#;