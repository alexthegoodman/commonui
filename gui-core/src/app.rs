use std::sync::Arc;
use tokio::sync::mpsc;
use std::time::{Duration, Instant};
use winit::{
    event::{Event as WinitEvent, WindowEvent, DeviceEvent, DeviceId, ElementState, KeyEvent, Modifiers, MouseButton, MouseScrollDelta},
    event_loop::EventLoop,
    platform::scancode::PhysicalKeyExtScancode,
    window::{Window, WindowId, WindowBuilder},
    keyboard::{KeyCode, ModifiersState},
    dpi::{LogicalSize, PhysicalSize},
};
use wgpu::{Device, Queue, Surface, Instance, Adapter, SurfaceConfiguration, TextureUsages, PresentMode, CommandEncoder};
use gui_reactive::global_frame_scheduler;
use gui_render::{VelloRenderer, primitives::TextRenderer};
use crate::event::{Event, MouseEvent, KeyboardEvent, Point};
use crate::media_query::ViewportSize;
use vello::ExternalResource;

#[derive(Debug)]
enum InternalEvent {
    MousePositionUpdate([f64; 2]),
    GuiEvent(Event),
}
use crate::{WidgetManager, Element};

pub struct App {
    pub window: Option<Arc<Window>>,
    internal_event_sender: mpsc::UnboundedSender<InternalEvent>,
    internal_event_receiver: mpsc::UnboundedReceiver<InternalEvent>,
    widget_manager: WidgetManager,
    // Rendering components
    wgpu_instance: Option<Instance>,
    surface: Option<Surface<'static>>,
    adapter: Option<Adapter>,
    pub device: Option<Arc<Device>>,
    pub queue: Option<Arc<Queue>>,
    surface_config: Option<SurfaceConfiguration>,
    vello_renderer: Option<VelloRenderer>,
    text_renderer: Option<TextRenderer>,
    // Update timing
    last_full_update: Instant,
    full_update_count: i32,
    last_mouse_position: [f64; 2],
    title: String,
    inner_size: [i32; 2],
    // Resume callback
    on_resume_callback: Option<Box<dyn FnOnce(Arc<Device>, Arc<Queue>) + Send>>,
    // Custom render callback for external rendering (like stunts-native)
    custom_render_callback: Option<Box<dyn Fn(&wgpu::Device, &wgpu::Queue, &wgpu::TextureView, u32, u32) -> Result<(), Box<dyn std::error::Error>> + Send + Sync>>,
    // Event handlers
    cursor_moved_handler: Option<Box<dyn Fn(f64, f64, f64, f64)>>,
    mouse_input_handler: Option<Box<dyn Fn(MouseButton, ElementState)>>,
    window_resize_handler: Option<Box<dyn FnMut(PhysicalSize<u32>, LogicalSize<f64>)>>,
    mouse_wheel_handler: Option<Box<dyn FnMut(MouseScrollDelta)>>,
    modifiers_changed_handler: Option<Box<dyn FnMut(Modifiers)>>,
    keyboard_input_handler: Option<Box<dyn FnMut(KeyEvent)>>,
}

impl App {
    pub fn new() -> Self {
        let (internal_event_sender, internal_event_receiver) = mpsc::unbounded_channel();
        
        Self {
            window: None,
            internal_event_sender,
            internal_event_receiver,
            widget_manager: WidgetManager::new(),
            wgpu_instance: None,
            surface: None,
            adapter: None,
            device: None,
            queue: None,
            surface_config: None,
            vello_renderer: None,
            text_renderer: None,
            last_full_update: Instant::now(),
            full_update_count: 0,
            last_mouse_position: [0.0, 0.0],
            title: "CommonUI".to_string(),
            inner_size: [800, 600],
            on_resume_callback: None,
            custom_render_callback: None,
            cursor_moved_handler: None,
            mouse_input_handler: None,
            window_resize_handler: None,
            mouse_wheel_handler: None,
            modifiers_changed_handler: None,
            keyboard_input_handler: None,
        }
    }

    pub fn with_root(mut self, root: Element) -> Result<Self, Box<dyn std::error::Error>> {
        self.widget_manager.set_root(root)?;
        Ok(self)
    }

    pub fn with_title(mut self, title: String) -> Result<Self, Box<dyn std::error::Error>> {
        self.title = title;
        Ok(self)
    }

    pub fn with_inner_size(mut self, inner_size: [i32; 2]) -> Result<Self, Box<dyn std::error::Error>> {
        self.inner_size = inner_size;
        Ok(self)
    }

    pub fn on_resume<F>(mut self, callback: F) -> Self 
    where
        F: FnOnce(Arc<Device>, Arc<Queue>) + Send + 'static,
    {
        self.on_resume_callback = Some(Box::new(callback));
        self
    }

    pub fn with_custom_render<F>(mut self, callback: F) -> Self
    where
        F: Fn(&wgpu::Device, &wgpu::Queue, &wgpu::TextureView, u32, u32) -> Result<(), Box<dyn std::error::Error>> + Send + Sync + 'static,
    {
        self.custom_render_callback = Some(Box::new(callback));
        self
    }

    pub fn with_cursor_moved<F>(mut self, handler: F) -> Self
    where
        F: Fn(f64, f64, f64, f64) + 'static,
    {
        self.cursor_moved_handler = Some(Box::new(handler));
        self
    }

    pub fn with_mouse_input<F>(mut self, handler: F) -> Self
    where
        F: Fn(MouseButton, ElementState) + 'static,
    {
        self.mouse_input_handler = Some(Box::new(handler));
        self
    }

    pub fn with_window_resize<F>(mut self, handler: F) -> Self
    where
        F: FnMut(PhysicalSize<u32>, LogicalSize<f64>) + 'static,
    {
        self.window_resize_handler = Some(Box::new(handler));
        self
    }

    pub fn with_mouse_wheel<F>(mut self, handler: F) -> Self
    where
        F: FnMut(MouseScrollDelta) + 'static,
    {
        self.mouse_wheel_handler = Some(Box::new(handler));
        self
    }

    pub fn with_modifiers_changed<F>(mut self, handler: F) -> Self
    where
        F: FnMut(Modifiers) + 'static,
    {
        self.modifiers_changed_handler = Some(Box::new(handler));
        self
    }

    pub fn with_keyboard_input<F>(mut self, handler: F) -> Self
    where
        F: FnMut(KeyEvent) + 'static,
    {
        self.keyboard_input_handler = Some(Box::new(handler));
        self
    }

    // Method to set up event handlers after GPU resources are available
    pub fn setup_event_handlers_with_resources<F>(mut self, setup_fn: F) -> Self
    where
        F: FnOnce(&mut Self, Arc<Device>, Arc<Queue>) + 'static,
    {
        // Store the setup function and call it during GPU initialization
        let setup_fn = Box::new(setup_fn);
        // We'll need to modify the initialization flow to support this
        self
    }

    pub fn run_with_editor_state<T, F>(
        mut self,
        editor_state: T,
        gpu_setup_callback: impl FnOnce(std::sync::Arc<wgpu::Device>, std::sync::Arc<wgpu::Queue>) + 'static,
        render_callback: Arc<F>,
    ) -> Result<(), Box<dyn std::error::Error>> 
    where
        T: 'static,
        F: Fn(&wgpu::Device, &wgpu::Queue, &mut wgpu::CommandEncoder, &[ExternalResource<'_>], &wgpu::TextureView) -> Result<(), vello::Error> + 'static,
    {
        let event_loop = EventLoop::new()?;
        let mut gpu_setup_callback = Some(gpu_setup_callback);
        
        event_loop.run(move |event, event_loop_window_target| {
            match event {
                WinitEvent::NewEvents(_) => {
                    // Start of new event batch
                }
                WinitEvent::Resumed => {
                    if self.window.is_none() {
                        let window = WindowBuilder::new()
                            .with_title(self.title.clone())
                            .with_inner_size(winit::dpi::LogicalSize::new(self.inner_size[0], self.inner_size[1]))
                            .build(event_loop_window_target)
                            .unwrap();
                        self.window = Some(Arc::new(window));
                        
                        // Initialize wgpu rendering
                        if let Err(e) = self.init_rendering() {
                            eprintln!("Failed to initialize rendering: {}", e);
                        }

                        // Run GPU setup callback if provided
                        if let (Some(device), Some(queue)) = (self.device.as_ref(), self.queue.as_ref()) {
                            if let Some(callback) = gpu_setup_callback.take() {
                                callback(device.clone(), queue.clone());
                            }
                        }

                        // Run on_resume callback if provided
                        if let Some(callback) = self.on_resume_callback.take() {
                            if let (Some(device), Some(queue)) = (self.device.as_ref(), self.queue.as_ref()) {
                                callback(device.clone(), queue.clone());
                            }
                        }
                    }
                }
                WinitEvent::WindowEvent { window_id, event } => {
                    match &event {
                        WindowEvent::RedrawRequested => {
                            // Handle rendering for specific window
                            if let Some(window) = &self.window {
                                if window.id() == window_id {
                                    // Begin frame with frame synchronization
                                    let frame_context = global_frame_scheduler().begin_frame();
                                    
                                    // Render frame with custom editor state render
                                    self.render_frame_with_editor_state(&editor_state, render_callback.clone());
                                    
                                    // End frame - this will flush any batched updates
                                    global_frame_scheduler().end_frame(frame_context);
                                }
                            }
                        }
                        _ => {
                            if self.handle_window_event(window_id, event) {
                                event_loop_window_target.exit();
                            }
                        }
                    }
                }
                WinitEvent::DeviceEvent { device_id, event } => {
                    self.handle_device_event(device_id, event);
                }
                WinitEvent::AboutToWait => {
                    // Request redraw
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
                _ => {}
            }
        })?;
        
        Ok(())
    }

    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new()?;
        
        event_loop.run(move |event, event_loop_window_target| {
            match event {
                WinitEvent::NewEvents(_) => {
                    // Start of new event batch
                }
                WinitEvent::Resumed => {
                    if self.window.is_none() {
                        let window = WindowBuilder::new()
                            .with_title(self.title.clone())
                            .with_inner_size(winit::dpi::LogicalSize::new(self.inner_size[0], self.inner_size[1]))
                            .build(event_loop_window_target)
                            .unwrap();
                        self.window = Some(Arc::new(window));
                        
                        // Initialize wgpu rendering
                        if let Err(e) = self.init_rendering() {
                            eprintln!("Failed to initialize rendering: {}", e);
                        }

                        // Run on_resume callback if provided
                        if let Some(callback) = self.on_resume_callback.take() {
                            if let (Some(device), Some(queue)) = (self.device.as_ref(), self.queue.as_ref()) {
                                callback(device.clone(), queue.clone());
                            }
                        }
                    }
                }
                WinitEvent::WindowEvent { window_id, event } => {
                    match &event {
                        WindowEvent::RedrawRequested => {
                            // Handle rendering for specific window
                            if let Some(window) = &self.window {
                                if window.id() == window_id {
                                    // Begin frame with frame synchronization
                                    let frame_context = global_frame_scheduler().begin_frame();
                                    
                                    // Render frame
                                    self.render_frame();
                                    
                                    // End frame - this will flush any batched updates
                                    global_frame_scheduler().end_frame(frame_context);
                                }
                            }
                        }
                        _ => {
                            if self.handle_window_event(window_id, event) {
                                event_loop_window_target.exit();
                            }
                        }
                    }
                }
                WinitEvent::DeviceEvent { device_id, event } => {
                    self.handle_device_event(device_id, event);
                }
                WinitEvent::AboutToWait => {
                    // Request redraw
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
                _ => {}
            }
        })?;
        
        Ok(())
    }

    pub fn window(&self) -> Option<&Arc<Window>> {
        self.window.as_ref()
    }

    fn handle_window_event(&mut self, _window_id: WindowId, event: WindowEvent) -> bool {
        match event {
            WindowEvent::CloseRequested => {
                return true; // Signal to close the app
            }
            WindowEvent::Resized(new_size) => {
                println!("Window resized to: {}x{}", new_size.width, new_size.height);
                if let Err(e) = self.handle_resize(new_size.width, new_size.height) {
                    eprintln!("Failed to handle window resize: {}", e);
                }
                // Call custom window resize handler
                if let Some(ref mut handler) = self.window_resize_handler {
                    let logical_size = LogicalSize::new(new_size.width as f64, new_size.height as f64);
                    handler(new_size, logical_size);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let _ = self.internal_event_sender.send(InternalEvent::MousePositionUpdate([position.x, position.y]));
                let _ = self.internal_event_sender.send(InternalEvent::GuiEvent(Event::Mouse(MouseEvent {
                    position: Point::new(position.x, position.y),
                    button: None,
                    state: ElementState::Released,
                    modifiers: ModifiersState::default(),
                })));
                // Call custom cursor moved handler
                if let Some(ref handler) = self.cursor_moved_handler {
                    // physical position (position.x, position.y) and logical position (same for now)
                    handler(position.x, position.y, position.x, position.y);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let _ = self.internal_event_sender.send(InternalEvent::GuiEvent(Event::Mouse(MouseEvent {
                    position: Point::new(self.last_mouse_position[0], self.last_mouse_position[1]),
                    button: Some(button),
                    state,
                    modifiers: ModifiersState::default(),
                })));
                // Call custom mouse input handler
                if let Some(ref handler) = self.mouse_input_handler {
                    handler(button, state);
                }
            }
            WindowEvent::KeyboardInput { 
                event,
                ..
            } => {
                if let Some(keycode) = event.physical_key.to_scancode() {
                    // Convert winit PhysicalKey to winit KeyCode if possible
                    let key_code = match event.logical_key {
                        winit::keyboard::Key::Named(named_key) => {
                            use winit::keyboard::NamedKey;
                            match named_key {
                                NamedKey::Enter => Some(KeyCode::Enter),
                                NamedKey::Escape => Some(KeyCode::Escape),
                                NamedKey::Backspace => Some(KeyCode::Backspace),
                                NamedKey::Tab => Some(KeyCode::Tab),
                                NamedKey::Space => Some(KeyCode::Space),
                                NamedKey::ArrowLeft => Some(KeyCode::ArrowLeft),
                                NamedKey::ArrowUp => Some(KeyCode::ArrowUp),
                                NamedKey::ArrowRight => Some(KeyCode::ArrowRight),
                                NamedKey::ArrowDown => Some(KeyCode::ArrowDown),
                                NamedKey::Delete => Some(KeyCode::Delete),
                                NamedKey::Home => Some(KeyCode::Home),
                                NamedKey::End => Some(KeyCode::End),
                                NamedKey::PageUp => Some(KeyCode::PageUp),
                                NamedKey::PageDown => Some(KeyCode::PageDown),
                                _ => None,
                            }
                        },
                        winit::keyboard::Key::Character(ref s) if s.len() == 1 => {
                            let c = s.chars().next().unwrap().to_ascii_uppercase();
                            match c {
                                'A' => Some(KeyCode::KeyA),
                                'B' => Some(KeyCode::KeyB),
                                'C' => Some(KeyCode::KeyC),
                                'D' => Some(KeyCode::KeyD),
                                'E' => Some(KeyCode::KeyE),
                                'F' => Some(KeyCode::KeyF),
                                'G' => Some(KeyCode::KeyG),
                                'H' => Some(KeyCode::KeyH),
                                'I' => Some(KeyCode::KeyI),
                                'J' => Some(KeyCode::KeyJ),
                                'K' => Some(KeyCode::KeyK),
                                'L' => Some(KeyCode::KeyL),
                                'M' => Some(KeyCode::KeyM),
                                'N' => Some(KeyCode::KeyN),
                                'O' => Some(KeyCode::KeyO),
                                'P' => Some(KeyCode::KeyP),
                                'Q' => Some(KeyCode::KeyQ),
                                'R' => Some(KeyCode::KeyR),
                                'S' => Some(KeyCode::KeyS),
                                'T' => Some(KeyCode::KeyT),
                                'U' => Some(KeyCode::KeyU),
                                'V' => Some(KeyCode::KeyV),
                                'W' => Some(KeyCode::KeyW),
                                'X' => Some(KeyCode::KeyX),
                                'Y' => Some(KeyCode::KeyY),
                                'Z' => Some(KeyCode::KeyZ),
                                '0' => Some(KeyCode::Digit0),
                                '1' => Some(KeyCode::Digit1),
                                '2' => Some(KeyCode::Digit2),
                                '3' => Some(KeyCode::Digit3),
                                '4' => Some(KeyCode::Digit4),
                                '5' => Some(KeyCode::Digit5),
                                '6' => Some(KeyCode::Digit6),
                                '7' => Some(KeyCode::Digit7),
                                '8' => Some(KeyCode::Digit8),
                                '9' => Some(KeyCode::Digit9),
                                _ => None,
                            }
                        },
                        _ => None,
                    };
                    
                    let _ = self.internal_event_sender.send(InternalEvent::GuiEvent(Event::Keyboard(KeyboardEvent {
                        key_code,
                        scancode: keycode,
                        state: event.state,
                        modifiers: ModifiersState::default(),
                    })));
                }
                // Call custom keyboard input handler
                if let Some(ref mut handler) = self.keyboard_input_handler {
                    handler(event);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                // Call custom mouse wheel handler
                if let Some(ref mut handler) = self.mouse_wheel_handler {
                    handler(delta);
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                // Call custom modifiers changed handler
                if let Some(ref mut handler) = self.modifiers_changed_handler {
                    handler(modifiers);
                }
            }
            _ => {}
        }
        false
    }

    fn handle_device_event(&self, _device_id: DeviceId, _event: DeviceEvent) {
        // Handle global device events if needed
    }

    fn handle_resize(&mut self, width: u32, height: u32) -> Result<(), Box<dyn std::error::Error>> {
        if width == 0 || height == 0 {
            return Ok(()); // Skip resize if dimensions are zero
        }

        if let (Some(surface), Some(device), Some(surface_config)) = 
            (self.surface.as_ref(), self.device.as_ref(), self.surface_config.as_mut()) {
            
            // Update surface configuration
            surface_config.width = width;
            surface_config.height = height;
            
            // Reconfigure the surface
            surface.configure(device, surface_config);
            
            // Update the renderer's viewport
            if let Some(vello_renderer) = self.vello_renderer.as_mut() {
                vello_renderer.set_viewport(width, height);
            }
            
            // Update the widget manager's viewport for media queries
            self.widget_manager.set_viewport_size(ViewportSize {
                width: width as f32,
                height: height as f32,
            });
        }
        
        Ok(())
    }

    fn render_frame(&mut self) {
        // Process any pending internal events
        let mut needs_immediate_update = false;
        while let Ok(internal_event) = self.internal_event_receiver.try_recv() {
            match internal_event {
                InternalEvent::MousePositionUpdate(position) => {
                    self.last_mouse_position = position;
                }
                InternalEvent::GuiEvent(event) => {
                    let result = self.widget_manager.handle_event(&event);
                    // If event was handled (interaction occurred), trigger immediate update
                    if matches!(result, crate::EventResult::Handled) {
                        needs_immediate_update = true;
                    }
                }
            }
        }
        
        // Check if it's time for a full update (every 1 second)
        // IMPLEMENTED: Run update_all on a regular interval (1 second), and have it update the dirty list with any component that gets updated
        // This way, most updates happen in a more targeted manner, but any missed updates are assured to run every 1 second
        let now = Instant::now();
        let should_full_update = now.duration_since(self.last_full_update) >= Duration::from_secs(1);
        
        if (should_full_update && self.full_update_count == 0) || needs_immediate_update {
            println!("update all");
            // Update all widgets - widgets will mark themselves as dirty when their position/state changes
            if let Err(e) = self.widget_manager.update_all() {
                eprintln!("Widget update error: {:?}", e);
            }
            
            self.last_full_update = now;
            self.full_update_count = self.full_update_count + 1;
        }
        
        // Render widgets to screen
        if let Err(e) = self.render_widgets() {
            eprintln!("Render error 1: {:?}", e);
        }
        
        // Clear dirty widgets for next frame
        self.widget_manager.clear_dirty_widgets();
    }

    fn render_frame_with_editor_state<T, F>(
        &mut self,
        editor_state: &T,
        // render_callback: &impl Fn(&T, &wgpu::Device, &wgpu::Queue, &wgpu::TextureView, u32, u32) -> Result<(), Box<dyn std::error::Error>>,
        render_callback: Arc<F>
    ) where
        F: Fn(&wgpu::Device, &wgpu::Queue, &mut wgpu::CommandEncoder, &[ExternalResource<'_>], &wgpu::TextureView) -> Result<(), vello::Error> + 'static {
        // Process any pending internal events
        let mut needs_immediate_update = false;
        while let Ok(internal_event) = self.internal_event_receiver.try_recv() {
            match internal_event {
                InternalEvent::MousePositionUpdate(position) => {
                    self.last_mouse_position = position;
                }
                InternalEvent::GuiEvent(event) => {
                    let result = self.widget_manager.handle_event(&event);
                    // If event was handled (interaction occurred), trigger immediate update
                    if matches!(result, crate::EventResult::Handled) {
                        needs_immediate_update = true;
                    }
                }
            }
        }
        
        // Check if it's time for a full update (every 1 second)
        let now = Instant::now();
        let should_full_update = now.duration_since(self.last_full_update) >= Duration::from_secs(1);
        
        if (should_full_update && self.full_update_count == 0) || needs_immediate_update {
            println!("update all");
            // Update all widgets - widgets will mark themselves as dirty when their position/state changes
            if let Err(e) = self.widget_manager.update_all() {
                eprintln!("Widget update error: {:?}", e);
            }
            
            self.last_full_update = now;
            self.full_update_count = self.full_update_count + 1;
        }
        
        // Render widgets to screen with editor state
        if let Err(e) = self.render_widgets_with_editor_state(editor_state, render_callback) {
            eprintln!("Render error 2: {:?}", e);
        }
        
        // Clear dirty widgets for next frame
        self.widget_manager.clear_dirty_widgets();
    }

    fn init_rendering(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let window = self.window.as_ref().ok_or("Window not available")?;
        
        // Create wgpu instance
        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: wgpu::InstanceFlags::default(),
            dx12_shader_compiler: wgpu::Dx12Compiler::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });
        
        // Create surface
        let surface = instance.create_surface(window.clone())?;
        
        // Get adapter
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })).ok_or("No adapter found")?;
        
        // Get device and queue
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: None,
        }, None))?;
        
        let device = Arc::new(device);
        let queue = Arc::new(queue);
        
        // Configure surface
        // let surface_caps = surface.get_capabilities(&adapter);
        // let surface_format = surface_caps.formats.iter()
        //     .copied()
        //     .find(|f| f.is_srgb())
        //     .unwrap_or(surface_caps.formats[0]);
        
        // surface format of Rgba8Unorm
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = wgpu::TextureFormat::Rgba8Unorm;

        let size = window.inner_size();
        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::STORAGE_BINDING,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        
        surface.configure(&device, &surface_config);
        
        // Create Vello renderer
        let vello_renderer = VelloRenderer::new(device.clone(), queue.clone(), surface_format)?;
        
        // Create text renderer
        let text_renderer = TextRenderer::new();
        
        // Store everything
        self.wgpu_instance = Some(instance);
        self.surface = Some(surface);
        self.adapter = Some(adapter);
        self.device = Some(device);
        self.queue = Some(queue);
        self.surface_config = Some(surface_config);
        self.vello_renderer = Some(vello_renderer);
        self.text_renderer = Some(text_renderer);
        
        Ok(())
    }

    fn render_widgets(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let surface = self.surface.as_ref().ok_or("Surface not initialized")?;
        let surface_config = self.surface_config.as_ref().ok_or("Surface config not available")?;
        let vello_renderer = self.vello_renderer.as_mut().ok_or("Vello renderer not initialized")?;
        
        // Begin frame
        vello_renderer.begin_frame();
        
        // Render the entire widget tree using the new Element::render method
        if let Some(root) = self.widget_manager.root() {
            if let Some(text_renderer) = &mut self.text_renderer {
                let device = self.device.as_ref().map(|d| d.as_ref());
                let queue = self.queue.as_ref().map(|q| q.as_ref());
                if let Err(e) = root.render(vello_renderer.scene(), text_renderer, device, queue) {
                    eprintln!("Widget render error: {:?}", e);
                }
            }
        }
        
        // Render to surface with direct render function
        let width = surface_config.width;
        let height = surface_config.height;
        vello_renderer.set_viewport(width, height);
        
        let surface_texture = surface.get_current_texture()?;
        let view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        // Execute direct render functions from Canvas widgets BEFORE Vello renders
        if let Some(root) = self.widget_manager.root() {
            if let (Some(device), Some(queue)) = (self.device.as_ref(), self.queue.as_ref()) {
                if let Err(e) = root.execute_direct_render_functions(device, queue, &view, width, height) {
                    eprintln!("Direct render error: {:?}", e);
                }
            }
        }
        
        // Execute custom render callback (for external rendering like stunts-native)
        if let Some(custom_callback) = &self.custom_render_callback {
            if let (Some(device), Some(queue)) = (self.device.as_ref(), self.queue.as_ref()) {
                if let Err(e) = custom_callback(device, queue, &view, width, height) {
                    eprintln!("Custom render error: {:?}", e);
                }
            }
        }
        
        // Now render Vello content with shared encoder render functions
        let shared_encoder_func = if let Some(root) = self.widget_manager.root() {
            root.create_combined_shared_encoder_render_func()
        } else {
            None
        };
        
        // vello_renderer.render_to_texture_view_with_shared_encoder(&view, width, height, shared_encoder_func)?;
        surface_texture.present();
        
        // End frame
        vello_renderer.end_frame();
        
        Ok(())
    }

    fn render_widgets_with_editor_state<T, F>(
        &mut self,
        editor_state: &T,
        // render_callback: &impl Fn(&T, &wgpu::Device, &wgpu::Queue, &wgpu::TextureView, u32, u32) -> Result<(), Box<dyn std::error::Error>>,
        render_callback: Arc<F>
    ) -> Result<(), Box<dyn std::error::Error>> where
        F: Fn(&wgpu::Device, &wgpu::Queue, &mut wgpu::CommandEncoder, &[ExternalResource<'_>], &wgpu::TextureView) -> Result<(), vello::Error> + 'static {
        let surface = self.surface.as_ref().ok_or("Surface not initialized")?;
        let surface_config = self.surface_config.as_ref().ok_or("Surface config not available")?;
        let vello_renderer = self.vello_renderer.as_mut().ok_or("Vello renderer not initialized")?;
        
        // Begin frame
        vello_renderer.begin_frame();
        
        // Render the entire widget tree using the new Element::render method
        if let Some(root) = self.widget_manager.root() {
            if let Some(text_renderer) = &mut self.text_renderer {
                let device = self.device.as_ref().map(|d| d.as_ref());
                let queue = self.queue.as_ref().map(|q| q.as_ref());
                if let Err(e) = root.render(vello_renderer.scene(), text_renderer, device, queue) {
                    eprintln!("Widget render error: {:?}", e);
                }
            }
        }
        
        // Render to surface with direct render function
        let width = surface_config.width;
        let height = surface_config.height;
        vello_renderer.set_viewport(width, height);
        
        let surface_texture = surface.get_current_texture()?;
        let view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        // Execute direct render functions from Canvas widgets BEFORE Vello renders
        if let Some(root) = self.widget_manager.root() {
            if let (Some(device), Some(queue)) = (self.device.as_ref(), self.queue.as_ref()) {
                if let Err(e) = root.execute_direct_render_functions(device, queue, &view, width, height) {
                    eprintln!("Direct render error: {:?}", e);
                }
            }
        }
        
        vello_renderer.render_to_texture_view_with_shared_encoder(editor_state, &view, width, height, render_callback.clone())?;
        surface_texture.present();
        
        // End frame
        vello_renderer.end_frame();
        
        Ok(())
    }


}