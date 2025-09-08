use gui_core::{App, Element};
use gui_core::widgets::*;
use gui_core::widgets::container::Padding;
use gui_core::widgets::text::text_signal;
use gui_reactive::Signal;
use vello::peniko::Color;
use gui_core::widgets::canvas::canvas;
use vello::kurbo::{Circle, RoundedRect};
use vello::{Scene, kurbo::Affine};
use wgpu::{Device, Queue};

mod canvas_example;

// Commented code is retained for reference

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if we should run the canvas examples
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "canvas" => {
                println!("Starting Canvas Widget Example...");
                let canvas_app = canvas_example::create_canvas_app()?;
                return canvas_app.run();
            }
            _ => {}
        }
    }

    println!("Starting CommonUI Hello World Example with Signals...");
    
    // Create reactive signals
    let counter_signal = Signal::new(0i32);
    let message_signal = Signal::new("Hello, CommonUI World!".to_string());
    
    // Demonstrate signal reactivity with subscriptions
    counter_signal.subscribe_fn(move |count| {
        println!("Counter updated to: {}", count);
        if *count == 5 {
            println!("You've clicked 5 times! Well done!");
        }
    });
    
    message_signal.subscribe_fn(move |msg| {
        println!("Message changed to: {}", msg);
    });
    
    // Create text that reacts to the message signal with shadow
    let hello_text = text_signal(message_signal.clone())
        .with_font_size(24.0)
        .with_color(Color::rgba8(250, 250, 200, 255))
        .with_shadow(2.0, 2.0, 4.0, Color::rgba8(0, 0, 0, 128));
    
    // Create a reactive computed signal for the subtitle
    let subtitle_signal = Signal::new(format!("Clicked {} times", counter_signal.get()));
    
    // Create reactive subtitle that shows counter with shadow
    let subtitle_text = text_signal(subtitle_signal.clone())
        .with_font_size(16.0)
        .with_color(Color::rgba8(100, 100, 100, 255))
        .with_shadow(1.0, 1.0, 2.0, Color::rgba8(0, 0, 0, 80));
    
    // Create a text input for message editing
    let message_for_input = message_signal.clone();
    let text_input = input()
        .with_size(250.0, 32.0)
        .with_placeholder("Enter your message...")
        .with_shadow(2.0, 2.0, 4.0, Color::rgba8(0, 0, 0, 100))
        .on_change(move |text| {
            message_for_input.set(text.to_string());
        });

    // Create a number input for the counter
    let counter_for_input = counter_signal.clone();
    let subtitle_for_input = subtitle_signal.clone();
    let number_input = input()
        .with_size(120.0, 32.0)
        .with_placeholder("Counter value...")
        .with_shadow(2.0, 2.0, 4.0, Color::rgba8(0, 0, 0, 100))
        .on_submit(move |text| {
            if let Ok(value) = text.parse::<i32>() {
                counter_for_input.set(value);
                subtitle_for_input.set(format!("Clicked {} times", value));
            }
        });

    // Create a slider for controlling the counter
    // let counter_for_slider = counter_signal.clone();
    // let subtitle_for_slider = subtitle_signal.clone();
    // let counter_slider = slider(0.0, 20.0)
    //     .with_size(200.0, 24.0)
    //     .with_value(counter_signal.get() as f32)
    //     .on_change(move |value| {
    //         let int_value = value as i32;
    //         counter_for_slider.set(int_value);
    //         subtitle_for_slider.set(format!("Clicked {} times", int_value));
    //     });
    
    // Create a button with shadow that will update the counter signal when clicked
    let counter_for_button = counter_signal.clone();
    let message_for_button = message_signal.clone();
    let subtitle_for_button = subtitle_signal.clone();
    let click_button = button("Click Me!")
        .with_size(120.0, 40.0)
        .with_shadow(13.0, 13.0, 16.0, Color::rgba8(0, 0, 0, 100))
        .on_click(move || {
            let current_count = counter_for_button.get();
            let new_count = current_count + 1;
            counter_for_button.set(new_count);
            
            // Update the subtitle signal
            subtitle_for_button.set(format!("Clicked {} times", new_count));
            
            // Update message after a few clicks
            if new_count == 1 {
                println!("click!");
                message_for_button.set("Great! You clicked the button!".to_string());
            } else if new_count == 3 {
                println!("click again!");
                message_for_button.set("Keep going!".to_string());
            } else if new_count >= 10 {
                println!("click champ!");
                message_for_button.set("Wow! You're a clicking champion!".to_string());
            }
        });
    
    // Create a column layout to arrange elements vertically
    let main_column = column()
        .with_size(300.0, 240.0) // Increased height to accommodate new widgets
        .with_main_axis_alignment(MainAxisAlignment::Center)
        .with_cross_axis_alignment(CrossAxisAlignment::Center)
        // .with_gap(10.0)
        .with_child(Element::new_widget(Box::new(hello_text)))
        .with_child(Element::new_widget(Box::new(subtitle_text)))
        .with_child(Element::new_widget(Box::new(text_input)))
        .with_child(Element::new_widget(Box::new(number_input)))
        // .with_child(Element::new_widget(Box::new(counter_slider)))
        .with_child(Element::new_widget(Box::new(click_button)));

    //     // Create the inner container with responsive shadow
    // let container2 = container()
    //     .with_size(300.0, 380.0) // Increased height to accommodate new widgets
    //     .with_background_color(Color::rgba8(200, 200, 200, 255))
    //     .with_padding(Padding::only(20.0, 0.0, 0.0, 0.0))
    //     .with_shadow(15.0, 15.0, 30.0, Color::rgba8(0, 0, 0, 150))
    //     // Responsive sizing for inner container
    //     // .with_responsive_style(
    //     //     mobile(),
    //     //     ResponsiveStyle::new()
    //     //         .with_size(200.0, 250.0)
    //     //         .with_padding(Padding::all(10.0))
    //     // )
    //     // .with_responsive_style(
    //     //     tablet(),
    //     //     ResponsiveStyle::new()
    //     //         .with_size(250.0, 275.0)
    //     //         .with_padding(Padding::all(15.0))
    //     // )
    //     // .with_responsive_style(
    //     //     desktop(),
    //     //     ResponsiveStyle::new()
    //     //         .with_size(400.0, 400.0)
    //     //         .with_padding(Padding::all(25.0))
    //     // )
    //     .with_child(main_column.into_container_element());

    // Create a simple canvas that draws a blue circle and red rounded rectangle
    let custom_canvas = canvas()
        .with_size(400.0, 300.0)
        // .with_position(50.0, 50.0)
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

    let main_row = row()
        .with_size(1000.0, 600.0) // Increased height to accommodate new widgets
        .with_main_axis_alignment(MainAxisAlignment::Start)
        .with_cross_axis_alignment(CrossAxisAlignment::Start)
        .with_gap(40.0)
        .with_child(main_column.into_container_element())
        .with_child(Element::new_widget(Box::new(custom_canvas)));
    
    // Create the root element with responsive styling
    let container = container()
        .with_size(1100.0, 700.0) // Increased height to accommodate new widgets
        .with_background_color(Color::rgba8(240, 240, 240, 255))
        // .with_padding(Padding::only(50.0, 0.0, 0.0, 0.0))
        .with_shadow(8.0, 8.0, 15.0, Color::rgba8(0, 0, 0, 80))
        // // Mobile styling - smaller size and padding
        // .with_responsive_style(
        //     mobile(),
        //     ResponsiveStyle::new()
        //         .with_size(300.0, 400.0)
        //         .with_padding(Padding::all(20.0))
        //         .with_background_color(Color::rgba8(220, 220, 255, 255))
        // )
        // // Tablet styling - medium size
        // .with_responsive_style(
        //     tablet(), 
        //     ResponsiveStyle::new()
        //         .with_size(400.0, 450.0)
        //         .with_padding(Padding::all(30.0))
        //         .with_background_color(Color::rgba8(255, 220, 220, 255))
        // )
        // // Desktop styling - larger size
        // .with_responsive_style(
        //     desktop(),
        //     ResponsiveStyle::new()
        //         .with_size(600.0, 600.0)
        //         .with_padding(Padding::all(50.0))
        //         .with_background_color(Color::rgba8(220, 255, 220, 255))
        // )
        .with_child(main_row.into_container_element());
    
    let root = container.into_container_element();
    
    // Start the application with the UI tree
    let app = App::new().with_title("CommonUI Example".to_string())?.with_inner_size([1200, 800])?.with_root(root)?;
    app.run()
}