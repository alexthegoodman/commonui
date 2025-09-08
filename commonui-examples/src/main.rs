use gui_core::{App, Element};
use gui_core::widgets::*;
use gui_core::widgets::container::Padding;
use gui_reactive::Signal;
use vello::peniko::Color;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    
    // Create text that reacts to the message signal
    let hello_text = text(&message_signal.get())
        .with_font_size(24.0)
        .with_color(Color::rgba8(250, 250, 200, 255));
    
    // Create reactive subtitle that shows counter
    let subtitle_text = text(&format!("Clicked {} times", counter_signal.get()))
        .with_font_size(16.0)
        .with_color(Color::rgba8(100, 100, 100, 255));
    
    // Create a button that will update the counter signal when clicked
    let counter_for_button = counter_signal.clone();
    let message_for_button = message_signal.clone();
    let click_button = button("Click Me!")
        .with_size(120.0, 40.0)
        .on_click(move || {
            let current_count = counter_for_button.get();
            counter_for_button.set(current_count + 1);
            
            // Update message after a few clicks
            if current_count + 1 == 1 {
                println!("click!");
                message_for_button.set("Great! You clicked the button!".to_string());
            } else if current_count + 1 == 3 {
                println!("click again!");
                message_for_button.set("Keep going!".to_string());
            } else if current_count + 1 >= 10 {
                println!("click champ!");
                message_for_button.set("Wow! You're a clicking champion!".to_string());
            }
        });
    
    // Create a column layout to arrange elements vertically
    let main_column = column()
        .with_size(300.0, 100.0)
        .with_main_axis_alignment(MainAxisAlignment::Center)
        .with_cross_axis_alignment(CrossAxisAlignment::Center)
        // .with_gap(10.0)
        .with_child(Element::new_widget(Box::new(hello_text)))
        .with_child(Element::new_widget(Box::new(subtitle_text)))
        .with_child(Element::new_widget(Box::new(click_button)));

        // Create the root element with some padding
    let container2 = container()
        .with_size(300.0, 300.0)
        .with_background_color(Color::rgba8(240, 40, 50, 255))
        .with_padding(Padding::only(20.0, 0.0, 0.0, 0.0))
        .with_child(main_column.into_container_element());
    
    // Create the root element with some padding  
    let container = container()
        .with_size(500.0, 500.0)
        .with_background_color(Color::rgba8(40, 40, 250, 255))
        .with_padding(Padding::all(40.0))
        .with_child(container2.into_container_element());
    
    let root = container.into_container_element();
    
    // Start the application with the UI tree
    let app = App::new().with_root(root)?;
    app.run()
}