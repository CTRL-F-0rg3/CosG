use bytemuck::{Pod, Zeroable};
use crate::widget::DrawRect;

/// Wierzchołek prostokąta (jako dwa trójkąty)
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct RectVertex {
    pos:           [f32; 2],
    color:         [f32; 4],
    border_radius: f32,
    // rect bounds dla SDF border-radius w shaderze
    rect_min:      [f32; 2],
    rect_max:      [f32; 2],
}

pub struct Renderer {
    pipeline:    wgpu::RenderPipeline,
    vertex_buf:  wgpu::Buffer,
    screen_size: (u32, u32),
}

impl Renderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, screen_size: (u32, u32)) -> Self {
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
                        0 => Float32x2,  // pos
                        1 => Float32x4,  // color
                        2 => Float32,    // border_radius
                        3 => Float32x2,  // rect_min
                        4 => Float32x2,  // rect_max
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module:      &shader,
                entry_point:"fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive:    wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample:  wgpu::MultisampleState::default(),
            multiview:    None,
            cache:        None,
        });

        // Bufor na max 4096 wierzchołków (682 prostokąty)
        let vertex_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label:              Some("cosg_vbuf"),
            size:               std::mem::size_of::<RectVertex>() as u64 * 4096,
            usage:              wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self { pipeline, vertex_buf, screen_size }
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        self.screen_size = (w, h);
    }

    pub fn render(
        &self,
        device:  &wgpu::Device,
        queue:   &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view:    &wgpu::TextureView,
        rects:   &[DrawRect],
        bg:      [f32; 4],
    ) {
        let (sw, sh) = (self.screen_size.0 as f32, self.screen_size.1 as f32);
        let mut verts: Vec<RectVertex> = Vec::with_capacity(rects.len() * 6);

        for dr in rects {
            let r = &dr.rect;
            // Konwersja px -> NDC
            let ndc = |x: f32, y: f32| -> [f32; 2] {
                [(x / sw) * 2.0 - 1.0, 1.0 - (y / sh) * 2.0]
            };

            let tl = ndc(r.x,       r.y);
            let tr = ndc(r.x + r.w, r.y);
            let bl = ndc(r.x,       r.y + r.h);
            let br = ndc(r.x + r.w, r.y + r.h);

            let rm = [r.x, r.y];
            let rx = [r.x + r.w, r.y + r.h];

            let v = |pos: [f32; 2]| RectVertex {
                pos, color: dr.color,
                border_radius: dr.border_radius,
                rect_min: rm, rect_max: rx,
            };

            // Dwa trójkąty
            verts.extend_from_slice(&[v(tl), v(tr), v(bl), v(tr), v(br), v(bl)]);
        }

        if !verts.is_empty() {
            queue.write_buffer(&self.vertex_buf, 0, bytemuck::cast_slice(&verts));
        }

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

        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
        pass.draw(0..verts.len() as u32, 0..1);
    }
}

// ─── WGSL shader ─────────────────────────────────────────────────────────────

const RECT_SHADER: &str = r#"
struct Vert {
    @location(0) pos:           vec2<f32>,
    @location(1) color:         vec4<f32>,
    @location(2) border_radius: f32,
    @location(3) rect_min:      vec2<f32>,
    @location(4) rect_max:      vec2<f32>,
}

struct Frag {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0)       color:    vec4<f32>,
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

// SDF dla zaokrąglonych rogów
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
