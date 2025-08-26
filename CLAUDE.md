# Welcome to CommonUI!

CommonUI is intended to be a lightweight, yet powerful and performant GUI kit for Rust, particularly for use in games and high-performance software. It is inspired by Rust's Floem.

Goals:

- High-performance: 60FPS (game-ready)
- Thread-safe State and Reactivity

Libraries:

- Vello
- Taffy

## Priority Tasks

Setup:

- ~~Set up core crates: gui-core, gui-reactive, gui-layout, gui-render~~
- ~~Configure development dependencies (criterion for benchmarks, etc.)~~

Core Dependencies Integration:

- ~~Add and configure Vello with wgpu backend~~
- ~~Integrate Taffy for layout engine~~
- ~~Add cosmic-text for text rendering~~
- ~~Set up tokio for async runtime (channels, tasks)~~
- ~~Configure winit for window management and input handling~~

Reactive System Foundation

Signal System:

- ~~Implement basic Signal<T> type with thread-safe updates~~
- ~~Create Computed<T> for derived values~~
- ~~Build Effect system for side effects~~
- ~~Add signal batching and frame-synchronized updates~~
- ~~Implement weak reference cleanup for memory management~~

Threading Architecture:

- ~~Design message passing between main/UI threads~~
- ~~Create thread-safe signal propagation system~~
- ~~Implement frame-synchronized state updates~~
- ~~Add proper shutdown and cleanup mechanisms~~

Layout and Rendering Pipeline

Layout Integration:

- ~~Wrap Taffy with reactive bindings~~
- ~~Create layout invalidation system tied to signals~~
- ~~Implement layout caching and dirty region tracking~~
- ~~Add support for common layout patterns (flexbox, grid)~~

Vello Rendering Backend:

- ~~Create VelloRenderer abstraction over raw Vello (already exists or started)~~
- ~~Implement scene caching for static UI elements~~
- ~~Add batching system for dynamic updates~~
- ~~Create primitive types (rectangles, text, images, shadows)~~

Widget System

Core Widget Traits:

- ~~Define Widget trait with lifecycle methods~~
- ~~Create Element enum for UI tree representation~~
- ~~Implement widget mounting/unmounting system~~
- ~~Add widget state management and updates~~

Essential Widgets:

- ~~Text widget with cosmic-text integration~~
- ~~Container widgets (Box, Stack, etc.)~~
- ~~Interactive widgets (Button, Input, Slider)~~
- ~~Layout widgets (Row, Column, Grid)~~

Input and Event System

Event Handling:

- ~~Create event types (Mouse, Keyboard, Touch)~~
- ~~Implement event propagation and bubbling~~
- ~~Add hit-testing with spatial indexing~~
- ~~Create focus management system~~
- Integrate winit events with reactive system (is this needed?)

Documentation and Examples

- Write comprehensive API documentation
- ~~Create "Hello World" example~~
- Add complex layout examples featuring multiple widgets

Testing and Validation

Testing Framework:

- Unit tests for reactive system
- Integration tests for widget interactions
- Performance benchmarks for 60FPS validation

Quality Assurance:

- Implement keyboard navigation
- Create accessibility hooks

## Final Instructions

Please take up whichever task or group of tasks you want (so long as they are not crossed off), and then cross them off when you are finished. This entire project will be completed over many sessions together.
