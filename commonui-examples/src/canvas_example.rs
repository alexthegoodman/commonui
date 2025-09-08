use gui_core::{App, Element};
use gui_core::widgets::canvas::canvas;
use gui_core::widgets::container::container;
use vello::peniko::Color;
use vello::kurbo::{Circle, RoundedRect};
use vello::{Scene, kurbo::Affine};
use wgpu::{Device, Queue};

pub fn create_canvas_app() -> Result<App, Box<dyn std::error::Error>> {
    // Create a simple canvas that draws a blue circle and red rounded rectangle
    let custom_canvas = canvas()
        .with_size(400.0, 300.0)
        .with_position(50.0, 50.0)
        .with_render_func(|scene: &mut Scene, _device: &Device, _queue: &Queue, x, y, width, height| {
            // Draw a blue circle in the center
            let circle_center = vello::kurbo::Point::new((x + width / 2.0) as f64, (y + height / 2.0) as f64);
            let circle = Circle::new(circle_center, 50.0);
            scene.fill(
                vello::peniko::Fill::NonZero,
                Affine::IDENTITY,
                Color::BLUE,
                None,
                &circle,
            );
            
            // Draw a red rounded rectangle in the top-left
            let rect = RoundedRect::new(
                (x + 20.0) as f64, 
                (y + 20.0) as f64, 
                (x + 120.0) as f64, 
                (y + 80.0) as f64, 
                10.0
            );
            scene.fill(
                vello::peniko::Fill::NonZero,
                Affine::IDENTITY,
                Color::RED,
                None,
                &rect,
            );
            
            Ok(())
        });

    // Create a container to hold our canvas
    let root = container()
        .with_size(800.0, 600.0)
        .with_background_color(Color::WHITE)
        .with_child(Element::new_widget(Box::new(custom_canvas)))
        .into_container_element();

    let app = App::new()
        .with_title("Canvas Widget Example".to_string())?
        .with_inner_size([800, 600])?
        .with_root(root)?;

    Ok(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] 
    fn test_canvas_app_creation() {
        let app_result = create_canvas_app();
        assert!(app_result.is_ok());
    }
}