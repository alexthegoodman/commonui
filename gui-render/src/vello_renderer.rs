use vello::{Renderer, Scene, RenderParams, AaConfig, AaSupport, RendererOptions};
use wgpu::{Device, Queue, Surface, TextureFormat};
use std::sync::Arc;
use std::num::NonZeroUsize;

pub struct VelloRenderer {
    device: Arc<Device>,
    queue: Arc<Queue>,
    renderer: Renderer,
    scene: Scene,
    surface_format: TextureFormat,
}

impl VelloRenderer {
    pub fn new(device: Arc<Device>, queue: Arc<Queue>, surface_format: TextureFormat) -> Result<Self, Box<dyn std::error::Error>> {
        let renderer = Renderer::new(
            &device,
            RendererOptions {
                surface_format: Some(surface_format),
                use_cpu: false,
                antialiasing_support: AaSupport::all(),
                num_init_threads: NonZeroUsize::new(std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1)),
            },
        )?;
        
        Ok(Self {
            device,
            queue,
            renderer,
            scene: Scene::new(),
            surface_format,
        })
    }

    pub fn render_to_surface(&mut self, surface: &Surface, width: u32, height: u32) -> Result<(), Box<dyn std::error::Error>> {
        let surface_texture = surface.get_current_texture()?;
        let view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let params = RenderParams {
            base_color: vello::peniko::Color::BLACK,
            width,
            height,
            antialiasing_method: AaConfig::Msaa16,
        };

        self.renderer.render_to_texture(&self.device, &self.queue, &self.scene, &view, &params)?;
        surface_texture.present();
        
        Ok(())
    }

    pub fn scene(&mut self) -> &mut Scene {
        &mut self.scene
    }

    pub fn clear_scene(&mut self) {
        self.scene.reset();
    }
}