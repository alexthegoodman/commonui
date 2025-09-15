use winit::event::{MouseButton, ElementState, TouchPhase};
use winit::keyboard::{ModifiersState, KeyCode};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

impl Size {
    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            origin: Point::new(x, y),
            size: Size::new(width, height),
        }
    }
    
    pub fn contains_point(&self, point: Point) -> bool {
        point.x >= self.origin.x && 
        point.x <= self.origin.x + self.size.width &&
        point.y >= self.origin.y && 
        point.y <= self.origin.y + self.size.height
    }
}

#[derive(Debug, Clone)]
pub struct MouseEvent {
    pub position: Point,
    pub button: Option<MouseButton>,
    pub state: ElementState,
    pub modifiers: ModifiersState,
}

#[derive(Debug, Clone)]
pub struct KeyboardEvent {
    pub key_code: Option<KeyCode>,
    pub character: Option<char>,
    pub scancode: u32,
    pub state: ElementState,
    pub modifiers: ModifiersState,
}

#[derive(Debug, Clone)]
pub struct TouchEvent {
    pub id: u64,
    pub phase: TouchPhase,
    pub position: Point,
    pub force: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct ScrollEvent {
    pub position: Point,
    pub delta: Point,
    pub modifiers: ModifiersState,
}

#[derive(Debug, Clone)]
pub enum Event {
    Mouse(MouseEvent),
    Keyboard(KeyboardEvent),
    Touch(TouchEvent),
    Scroll(ScrollEvent),
    WindowResize(Size),
    WindowClose,
    WindowFocus(bool),
}

pub struct EventContext {
    pub target_id: Option<u64>,
    pub propagation_stopped: bool,
    pub default_prevented: bool,
}

impl EventContext {
    pub fn new(target_id: Option<u64>) -> Self {
        Self {
            target_id,
            propagation_stopped: false,
            default_prevented: false,
        }
    }

    pub fn stop_propagation(&mut self) {
        self.propagation_stopped = true;
    }

    pub fn prevent_default(&mut self) {
        self.default_prevented = true;
    }
}

pub trait EventHandler: Send + Sync {
    fn handle_event(&mut self, event: &Event, context: &mut EventContext) -> bool;
}

pub struct EventDispatcher {
    handlers: HashMap<u64, Box<dyn EventHandler>>,
    widget_tree: HashMap<u64, Vec<u64>>, // widget_id -> parent_chain
    spatial_index: SpatialIndex,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            widget_tree: HashMap::new(),
            spatial_index: SpatialIndex::new(),
        }
    }

    pub fn register_handler(&mut self, widget_id: u64, handler: Box<dyn EventHandler>) {
        self.handlers.insert(widget_id, handler);
    }

    pub fn unregister_handler(&mut self, widget_id: u64) {
        self.handlers.remove(&widget_id);
        self.widget_tree.remove(&widget_id);
        self.spatial_index.remove_widget(widget_id);
    }

    pub fn set_widget_parent_chain(&mut self, widget_id: u64, parent_chain: Vec<u64>) {
        self.widget_tree.insert(widget_id, parent_chain);
    }

    pub fn update_widget_bounds(&mut self, widget_id: u64, bounds: Rect, z_index: i32) {
        self.spatial_index.insert_widget(widget_id, bounds, z_index);
    }

    pub fn dispatch_event(&mut self, event: &Event, target_widget_id: Option<u64>) -> bool {
        let mut context = EventContext::new(target_widget_id);
        let mut handled = false;

        if let Some(widget_id) = target_widget_id {
            // Get the parent chain for bubbling
            let parent_chain = self.widget_tree.get(&widget_id).cloned().unwrap_or_default();
            
            // Start with the target widget and bubble up
            let mut event_path = vec![widget_id];
            event_path.extend(parent_chain.iter().rev().cloned());

            for current_widget_id in event_path {
                if context.propagation_stopped {
                    break;
                }

                if let Some(handler) = self.handlers.get_mut(&current_widget_id) {
                    if handler.handle_event(event, &mut context) {
                        handled = true;
                        if context.propagation_stopped {
                            break;
                        }
                    }
                }
            }
        }

        handled
    }

    pub fn hit_test(&self, point: Point) -> Option<u64> {
        self.spatial_index.hit_test_single(point)
    }

    pub fn hit_test_all(&self, point: Point) -> Vec<u64> {
        self.spatial_index.hit_test(point)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusDirection {
    Next,
    Previous,
    Up,
    Down,
    Left,
    Right,
}

pub struct FocusManager {
    focused_widget: Option<u64>,
    focusable_widgets: Vec<u64>,
    tab_order: HashMap<u64, i32>,
}

impl FocusManager {
    pub fn new() -> Self {
        Self {
            focused_widget: None,
            focusable_widgets: Vec::new(),
            tab_order: HashMap::new(),
        }
    }

    pub fn add_focusable_widget(&mut self, widget_id: u64, tab_index: Option<i32>) {
        if !self.focusable_widgets.contains(&widget_id) {
            self.focusable_widgets.push(widget_id);
        }
        
        if let Some(index) = tab_index {
            self.tab_order.insert(widget_id, index);
        }
        
        // Sort focusable widgets by tab order
        self.focusable_widgets.sort_by(|a, b| {
            let order_a = self.tab_order.get(a).unwrap_or(&(i32::MAX));
            let order_b = self.tab_order.get(b).unwrap_or(&(i32::MAX));
            order_a.cmp(order_b).then_with(|| a.cmp(b))
        });
    }

    pub fn remove_focusable_widget(&mut self, widget_id: u64) {
        if self.focused_widget == Some(widget_id) {
            self.focused_widget = None;
        }
        self.focusable_widgets.retain(|&id| id != widget_id);
        self.tab_order.remove(&widget_id);
    }

    pub fn focus_widget(&mut self, widget_id: Option<u64>) -> Option<u64> {
        let previous_focus = self.focused_widget;
        self.focused_widget = widget_id;
        previous_focus
    }

    pub fn get_focused_widget(&self) -> Option<u64> {
        self.focused_widget
    }

    pub fn focus_next(&mut self, direction: FocusDirection) -> Option<u64> {
        if self.focusable_widgets.is_empty() {
            return None;
        }

        let current_index = if let Some(focused) = self.focused_widget {
            self.focusable_widgets.iter().position(|&id| id == focused)
        } else {
            None
        };

        let next_index = match direction {
            FocusDirection::Next => {
                match current_index {
                    Some(idx) => (idx + 1) % self.focusable_widgets.len(),
                    None => 0,
                }
            },
            FocusDirection::Previous => {
                match current_index {
                    Some(idx) => {
                        if idx == 0 {
                            self.focusable_widgets.len() - 1
                        } else {
                            idx - 1
                        }
                    },
                    None => self.focusable_widgets.len() - 1,
                }
            },
            // For now, treat directional focus the same as tab navigation
            // In a real implementation, this would use spatial relationships
            FocusDirection::Up | FocusDirection::Left => {
                match current_index {
                    Some(idx) => {
                        if idx == 0 {
                            self.focusable_widgets.len() - 1
                        } else {
                            idx - 1
                        }
                    },
                    None => self.focusable_widgets.len() - 1,
                }
            },
            FocusDirection::Down | FocusDirection::Right => {
                match current_index {
                    Some(idx) => (idx + 1) % self.focusable_widgets.len(),
                    None => 0,
                }
            },
        };

        let next_widget = self.focusable_widgets[next_index];
        let previous_focus = self.focus_widget(Some(next_widget));
        previous_focus
    }

    pub fn is_widget_focused(&self, widget_id: u64) -> bool {
        self.focused_widget == Some(widget_id)
    }
}

#[derive(Debug, Clone)]
pub struct SpatialIndex {
    widgets: HashMap<u64, (Rect, i32)>, // widget_id -> (bounds, z_index)
    dirty: bool,
}

impl SpatialIndex {
    pub fn new() -> Self {
        Self {
            widgets: HashMap::new(),
            dirty: false,
        }
    }

    pub fn insert_widget(&mut self, widget_id: u64, bounds: Rect, z_index: i32) {
        self.widgets.insert(widget_id, (bounds, z_index));
        self.dirty = true;
    }

    pub fn remove_widget(&mut self, widget_id: u64) {
        self.widgets.remove(&widget_id);
        self.dirty = true;
    }

    pub fn update_widget_bounds(&mut self, widget_id: u64, bounds: Rect) {
        if let Some((_, z_index)) = self.widgets.get(&widget_id) {
            let z_index = *z_index;
            self.widgets.insert(widget_id, (bounds, z_index));
            self.dirty = true;
        }
    }

    pub fn hit_test(&self, point: Point) -> Vec<u64> {
        let mut candidates: Vec<(u64, i32)> = self.widgets
            .iter()
            .filter(|(_, (bounds, _))| bounds.contains_point(point))
            .map(|(&widget_id, (_, z_index))| (widget_id, *z_index))
            .collect();

        // Sort by z-index (highest first)
        candidates.sort_by(|a, b| b.1.cmp(&a.1));
        
        candidates.into_iter().map(|(widget_id, _)| widget_id).collect()
    }

    pub fn hit_test_single(&self, point: Point) -> Option<u64> {
        self.hit_test(point).into_iter().next()
    }

    pub fn query_region(&self, region: Rect) -> Vec<u64> {
        self.widgets
            .iter()
            .filter(|(_, (bounds, _))| {
                // Simple AABB intersection test
                !(region.origin.x > bounds.origin.x + bounds.size.width ||
                  region.origin.x + region.size.width < bounds.origin.x ||
                  region.origin.y > bounds.origin.y + bounds.size.height ||
                  region.origin.y + region.size.height < bounds.origin.y)
            })
            .map(|(&widget_id, _)| widget_id)
            .collect()
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SpatialIndex {
    fn default() -> Self {
        Self::new()
    }
}