use vello::{Renderer, Scene, RenderParams, AaConfig, AaSupport, RendererOptions};
use wgpu::{Device, Queue, Surface, TextureFormat, TextureView};
use std::sync::Arc;
use std::num::NonZeroUsize;
use crate::scene_cache::SceneCache;

pub type CustomRenderFn = Box<dyn Fn(&Device, &Queue, &TextureView, u32, u32) -> Result<(), Box<dyn std::error::Error>> + Send + Sync>;

#[derive(Debug)]
pub enum RenderError {
    VelloError(Box<dyn std::error::Error>),
    SurfaceError(wgpu::SurfaceError),
}

impl From<Box<dyn std::error::Error>> for RenderError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        RenderError::VelloError(err)
    }
}

impl From<wgpu::SurfaceError> for RenderError {
    fn from(err: wgpu::SurfaceError) -> Self {
        RenderError::SurfaceError(err)
    }
}


impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RenderError::VelloError(e) => write!(f, "Vello error: {}", e),
            RenderError::SurfaceError(e) => write!(f, "Surface error: {:?}", e),
        }
    }
}

impl std::error::Error for RenderError {}

pub struct VelloRenderer {
    device: Arc<Device>,
    queue: Arc<Queue>,
    renderer: Renderer,
    scene: Scene,
    surface_format: TextureFormat,
    scene_cache: SceneCache,
    viewport_width: u32,
    viewport_height: u32,
    custom_render_fns: Vec<CustomRenderFn>,
}

impl VelloRenderer {
    pub fn new(device: Arc<Device>, queue: Arc<Queue>, surface_format: TextureFormat) -> Result<Self, RenderError> {
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
            scene_cache: SceneCache::new(),
            viewport_width: 0,
            viewport_height: 0,
            custom_render_fns: Vec::new(),
        })
    }

    pub fn set_viewport(&mut self, width: u32, height: u32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }

    pub fn viewport_size(&self) -> (u32, u32) {
        (self.viewport_width, self.viewport_height)
    }

    pub fn render_to_surface(&mut self, surface: &Surface, width: u32, height: u32) -> Result<(), RenderError> {
        self.set_viewport(width, height);
        
        let surface_texture = surface.get_current_texture()?;
        let view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        self.render_to_texture_view(&view, width, height)?;
        surface_texture.present();
        
        Ok(())
    }

    pub fn render_to_texture_view(&mut self, view: &TextureView, width: u32, height: u32) -> Result<(), RenderError> {
        // Execute custom render functions BEFORE Vello renders
        // This allows custom GPU commands to render to the same texture view
        self.execute_custom_render_fns(view, width, height)?;

        let params = RenderParams {
            base_color: vello::peniko::Color::TRANSPARENT, // Changed to transparent so custom rendering shows through
            width,
            height,
            antialiasing_method: AaConfig::Msaa16,
        };

        self.renderer.render_to_texture(&self.device, &self.queue, &self.scene, view, &params)?;
        
        Ok(())
    }

    pub fn render_to_texture_view_with_direct<F>(&mut self, view: &TextureView, width: u32, height: u32, direct_render_fn: Option<F>) -> Result<(), RenderError> 
    where 
        F: FnOnce(&wgpu::Device, &wgpu::Queue, &TextureView, u32, u32) -> Result<(), Box<dyn std::error::Error>>
    {
        // Execute custom render functions BEFORE Vello renders
        self.execute_custom_render_fns(view, width, height)?;
        
        // Execute direct render function if provided
        if let Some(render_fn) = direct_render_fn {
            if let Err(e) = render_fn(&self.device, &self.queue, view, width, height) {
                eprintln!("Direct render function error: {}", e);
            }
        }

        let params = RenderParams {
            base_color: vello::peniko::Color::TRANSPARENT,
            width,
            height,
            antialiasing_method: AaConfig::Msaa16,
        };

        self.renderer.render_to_texture(&self.device, &self.queue, &self.scene, view, &params)?;
        
        Ok(())
    }

    pub fn scene(&mut self) -> &mut Scene {
        &mut self.scene
    }

    pub fn scene_cache(&mut self) -> &mut SceneCache {
        &mut self.scene_cache
    }

    pub fn clear_scene(&mut self) {
        self.scene.reset();
    }

    pub fn begin_frame(&mut self) {
        self.scene_cache.next_frame();
        self.clear_scene();
    }

    pub fn end_frame(&mut self) {
        // Frame complete - could add any cleanup here
    }

    pub fn add_custom_render_fn(&mut self, render_fn: CustomRenderFn) {
        self.custom_render_fns.push(render_fn);
    }

    pub fn clear_custom_render_fns(&mut self) {
        self.custom_render_fns.clear();
    }

    fn execute_custom_render_fns(&self, view: &TextureView, width: u32, height: u32) -> Result<(), RenderError> {
        for render_fn in &self.custom_render_fns {
            if let Err(e) = render_fn(&self.device, &self.queue, view, width, height) {
                eprintln!("Custom render function error: {}", e);
            }
        }
        Ok(())
    }
}