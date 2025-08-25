use std::sync::Arc;
use tokio::sync::mpsc;
use winit::{
    event::{Event as WinitEvent, WindowEvent, DeviceEvent, DeviceId, ElementState},
    event_loop::EventLoop,
    platform::scancode::PhysicalKeyExtScancode,
    window::{Window, WindowId, WindowBuilder},
};
use crate::Event;

pub struct App {
    window: Option<Arc<Window>>,
    event_sender: mpsc::UnboundedSender<Event>,
    event_receiver: mpsc::UnboundedReceiver<Event>,
}

impl App {
    pub fn new() -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            window: None,
            event_sender,
            event_receiver,
        }
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
                                    // Render frame
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
                let _ = self.event_sender.send(Event::Mouse {
                    x: position.x,
                    y: position.y,
                    button: None,
                    pressed: false,
                });
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let _ = self.event_sender.send(Event::Mouse {
                    x: 0.0,
                    y: 0.0,
                    button: Some(button),
                    pressed: state == ElementState::Pressed,
                });
            }
            WindowEvent::KeyboardInput { 
                event,
                ..
            } => {
                if let Some(keycode) = event.physical_key.to_scancode() {
                    let _ = self.event_sender.send(Event::Keyboard {
                        key: keycode,
                        pressed: event.state == ElementState::Pressed,
                    });
                }
            }
            _ => {}
        }
        false
    }

    fn handle_device_event(&self, _device_id: DeviceId, _event: DeviceEvent) {
        // Handle global device events if needed
    }
}