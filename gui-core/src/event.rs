use winit::event::MouseButton;

#[derive(Debug, Clone)]
pub enum Event {
    Mouse {
        x: f64,
        y: f64,
        button: Option<MouseButton>,
        pressed: bool,
    },
    Keyboard {
        key: u32, // scancode
        pressed: bool,
    },
}