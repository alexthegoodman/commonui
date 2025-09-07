use gui_core::{App, Element};
use gui_core::widgets::*;
use gui_core::widgets::container::Padding;
use vello::peniko::Color;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting CommonUI Hello World Example...");
    
    // Create the UI tree
    let hello_text = text("Hello, CommonUI World!")
        .with_font_size(24.0)
        .with_color(Color::rgba8(250, 250, 200, 255));
    
    let subtitle_text = text("A high-performance GUI toolkit for Rust")
        .with_font_size(16.0)
        .with_color(Color::rgba8(100, 100, 100, 255));
    
    let click_button = button("Click Me!")
        .with_size(120.0, 40.0);
    
    // Create a column layout to arrange elements vertically
    let main_column = column()
        .with_size(200.0, 200.0)
        .with_main_axis_alignment(MainAxisAlignment::Center)
        .with_cross_axis_alignment(CrossAxisAlignment::Center)
        .with_gap(20.0)
        .with_child(Element::new_widget(Box::new(hello_text)))
        .with_child(Element::new_widget(Box::new(subtitle_text)))
        .with_child(Element::new_widget(Box::new(click_button)));

        // Create the root element with some padding
    let container2 = container()
        .with_size(300.0, 300.0)
        .with_background_color(Color::rgba8(240, 40, 50, 255))
        .with_padding(Padding::all(50.0))
        .with_child(main_column.into_container_element());
    
    // Create the root element with some padding  
    let container = container()
        .with_size(500.0, 500.0)
        .with_background_color(Color::rgba8(40, 40, 250, 255))
        .with_padding(Padding::all(50.0))
        .with_child(container2.into_container_element());
    
    let root = container.into_container_element();
    
    // Start the application with the UI tree
    let app = App::new().with_root(root)?;
    app.run()
}