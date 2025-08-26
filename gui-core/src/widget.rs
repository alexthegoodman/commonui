use crate::event::Event;
use std::any::Any;

pub trait Widget: Send + Sync {
    fn mount(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
    
    fn unmount(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
    
    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
    
    fn handle_event(&mut self, _event: &Event) -> EventResult {
        EventResult::Ignored
    }
    
    fn needs_layout(&self) -> bool {
        false
    }
    
    fn needs_render(&self) -> bool {
        true
    }
    
    fn render(&self) -> Result<RenderData, WidgetError>;
    
    fn as_any(&self) -> &dyn Any;
    
    fn as_any_mut(&mut self) -> &mut dyn Any;
    
    fn get_id(&self) -> WidgetId;
}

pub type WidgetId = u64;

#[derive(Debug, Clone)]
pub enum EventResult {
    Handled,
    Ignored,
    Propagate,
}

#[derive(Debug)]
pub enum WidgetError {
    LayoutError(String),
    RenderError(String),
    StateError(String),
}

impl std::fmt::Display for WidgetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WidgetError::LayoutError(msg) => write!(f, "Layout error: {}", msg),
            WidgetError::RenderError(msg) => write!(f, "Render error: {}", msg),
            WidgetError::StateError(msg) => write!(f, "State error: {}", msg),
        }
    }
}

impl std::error::Error for WidgetError {}

pub struct RenderData {
    pub dirty_regions: Vec<DirtyRegion>,
    pub z_index: i32,
}

#[derive(Debug, Clone)]
pub struct DirtyRegion {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}