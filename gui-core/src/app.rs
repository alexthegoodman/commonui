use std::sync::Arc;
use tokio::sync::mpsc;
use winit::{
    event::{Event as WinitEvent, WindowEvent, DeviceEvent, DeviceId, ElementState},
    event_loop::EventLoop,
    platform::scancode::PhysicalKeyExtScancode,
    window::{Window, WindowId, WindowBuilder},
};
use gui_reactive::{global_frame_scheduler, FrameContext};
use crate::event::{Event, MouseEvent, KeyboardEvent, Point};
use crate::{WidgetManager, Element};
use winit::keyboard::ModifiersState;

pub struct App {
    window: Option<Arc<Window>>,
    event_sender: mpsc::UnboundedSender<Event>,
    event_receiver: mpsc::UnboundedReceiver<Event>,
    widget_manager: WidgetManager,
}

impl App {
    pub fn new() -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            window: None,
            event_sender,
            event_receiver,
            widget_manager: WidgetManager::new(),
        }
    }

    pub fn with_root(mut self, root: Element) -> Result<Self, Box<dyn std::error::Error>> {
        self.widget_manager.set_root(root)?;
        Ok(self)
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
                            .with_title("CommonUI Application")
                            .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
                            .build(event_loop_window_target)
                            .unwrap();
                        self.window = Some(Arc::new(window));
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

    fn handle_window_event(&self, _window_id: WindowId, event: WindowEvent) -> bool {
        match event {
            WindowEvent::CloseRequested => {
                return true; // Signal to close the app
            }
            WindowEvent::Resized(new_size) => {
                println!("Window resized to: {}x{}", new_size.width, new_size.height);
            }
            WindowEvent::CursorMoved { position, .. } => {
                let _ = self.event_sender.send(Event::Mouse(MouseEvent {
                    position: Point::new(position.x, position.y),
                    button: None,
                    state: ElementState::Released,
                    modifiers: ModifiersState::default(),
                }));
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let _ = self.event_sender.send(Event::Mouse(MouseEvent {
                    position: Point::new(0.0, 0.0), // Will be updated with actual position
                    button: Some(button),
                    state,
                    modifiers: ModifiersState::default(),
                }));
            }
            WindowEvent::KeyboardInput { 
                event,
                ..
            } => {
                if let Some(keycode) = event.physical_key.to_scancode() {
                    let _ = self.event_sender.send(Event::Keyboard(KeyboardEvent {
                        key_code: None, // TODO: proper key code conversion
                        scancode: keycode,
                        state: event.state,
                        modifiers: ModifiersState::default(),
                    }));
                }
            }
            _ => {}
        }
        false
    }

    fn handle_device_event(&self, _device_id: DeviceId, _event: DeviceEvent) {
        // Handle global device events if needed
    }

    fn render_frame(&mut self) {
        // Process any pending events
        while let Ok(event) = self.event_receiver.try_recv() {
            self.widget_manager.handle_event(&event);
        }
        
        // Update all widgets
        if let Err(e) = self.widget_manager.update_all() {
            eprintln!("Widget update error: {:?}", e);
        }
        
        // Clear dirty widgets for next frame
        self.widget_manager.clear_dirty_widgets();
    }
}