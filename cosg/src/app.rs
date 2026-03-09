use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};
use crate::{
    renderer::Renderer,
    theme::Theme,
    widget::{Widget, WidgetEvent},
};

pub struct AppConfig {
    pub title:  String,
    pub width:  u32,
    pub height: u32,
    pub theme:  Theme,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            title:  "CosG App".into(),
            width:  800,
            height: 600,
            theme:  Theme::violet_dark(),
        }
    }
}

struct State {
    window:   std::sync::Arc<Window>,
    device:   wgpu::Device,
    queue:    wgpu::Queue,
    surface:  wgpu::Surface<'static>,
    config:   wgpu::SurfaceConfiguration,
    renderer: Renderer,
    theme:    Theme,
    root:     Box<dyn Widget>,
    cursor:   (f32, f32),
}

impl State {
    async fn new(window: std::sync::Arc<Window>, theme: Theme, root: Box<dyn Widget>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();

        let caps   = surface.get_capabilities(&adapter);
        let format = caps.formats[0];
        let cfg = wgpu::SurfaceConfiguration {
            usage:        wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width:        size.width,
            height:       size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode:   caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &cfg);

        let renderer = Renderer::new(&device, format, (size.width, size.height));

        let mut root = root;
        root.layout(
            crate::widget::Rect::new(0.0, 0.0, size.width as f32, size.height as f32),
            &theme,
        );

        Self { window, device, queue, surface, config: cfg, renderer, theme, root, cursor: (0.0, 0.0) }
    }

    fn resize(&mut self, w: u32, h: u32) {
        self.config.width  = w;
        self.config.height = h;
        self.surface.configure(&self.device, &self.config);
        self.renderer.resize(w, h);
        self.root.layout(
            crate::widget::Rect::new(0.0, 0.0, w as f32, h as f32),
            &self.theme,
        );
    }

    fn render(&self) {
        let frame   = self.surface.get_current_texture().unwrap();
        let view    = frame.texture.create_view(&Default::default());
        let mut enc = self.device.create_command_encoder(&Default::default());
        let rects   = self.root.draw(&self.theme);
        self.renderer.render(&self.device, &self.queue, &mut enc, &view, &rects, self.theme.bg);
        self.queue.submit(std::iter::once(enc.finish()));
        frame.present();
    }
}

// ─── ApplicationHandler ───────────────────────────────────────────────────────

pub struct App {
    config:      AppConfig,
    root_fn:     Box<dyn FnOnce() -> Box<dyn Widget>>,
    state:       Option<State>,
}

impl App {
    pub fn new(config: AppConfig, root: impl FnOnce() -> Box<dyn Widget> + 'static) -> Self {
        Self { config, root_fn: Box::new(root), state: None }
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.run_app(&mut self).unwrap();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title(&self.config.title)
            .with_inner_size(winit::dpi::PhysicalSize::new(self.config.width, self.config.height));
        let window = std::sync::Arc::new(el.create_window(attrs).unwrap());

        // Dummy root żeby zastąpić root_fn (FnOnce nie może być wywołane dwukrotnie)
        let root_fn = std::mem::replace(&mut self.root_fn, Box::new(|| {
            Box::new(crate::widgets::Container::new()) as Box<dyn Widget>
        }));
        let root = root_fn();

        let state = pollster::block_on(
            State::new(window, self.config.theme.clone(), root)
        );
        self.state = Some(state);
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(state) = &mut self.state else { return };

        match event {
            WindowEvent::CloseRequested => el.exit(),

            WindowEvent::Resized(size) => state.resize(size.width, size.height),

            WindowEvent::CursorMoved { position, .. } => {
                state.cursor = (position.x as f32, position.y as f32);
                state.root.handle_event(&WidgetEvent::MouseMove {
                    x: state.cursor.0, y: state.cursor.1,
                });
                state.window.request_redraw();
            }

            WindowEvent::MouseInput { state: btn_state, button: MouseButton::Left, .. } => {
                let (x, y) = state.cursor;
                let ev = match btn_state {
                    ElementState::Pressed  => WidgetEvent::MouseDown { x, y },
                    ElementState::Released => WidgetEvent::MouseUp   { x, y },
                };
                state.root.handle_event(&ev);
                state.window.request_redraw();
            }

            WindowEvent::RedrawRequested => state.render(),

            _ => {}
        }
    }
}
