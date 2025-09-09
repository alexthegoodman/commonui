use crate::{Widget, WidgetId, EventResult, WidgetError, RenderData, DirtyRegion, WidgetUpdateContext};
use crate::event::Event;
use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use vello::Scene;
use wgpu::{Device, Queue, CommandEncoder};
use vello::ExternalResource;
use std::sync::Arc;

static WIDGET_ID_COUNTER: AtomicU64 = AtomicU64::new(2000);

pub type CanvasRenderFunc = Box<dyn Fn(&mut Scene, &Device, &Queue, f32, f32, f32, f32) -> Result<(), WidgetError> + Send + Sync>;
pub type CanvasDirectRenderFunc = Box<dyn Fn(&Device, &Queue, &wgpu::TextureView, u32, u32, f32, f32, f32, f32) -> Result<(), Box<dyn std::error::Error>> + Send + Sync>;
pub type CanvasSharedEncoderRenderFunc = Arc<dyn Fn(&Device, &Queue, &mut CommandEncoder, &[ExternalResource], f32, f32, f32, f32) -> Result<(), vello::Error> + Send + Sync>;

pub struct CanvasWidget {
    id: WidgetId,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    render_func: Option<CanvasRenderFunc>,
    direct_render_func: Option<CanvasDirectRenderFunc>,
    shared_encoder_render_func: Option<CanvasSharedEncoderRenderFunc>,
    dirty: bool,
    z_index: i32,
}

impl CanvasWidget {
    pub fn new() -> Self {
        Self {
            id: WIDGET_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            render_func: None,
            direct_render_func: None,
            shared_encoder_render_func: None,
            dirty: true,
            z_index: 0,
        }
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self.dirty = true;
        self
    }

    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.x = x;
        self.y = y;
        self.dirty = true;
        self
    }

    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self.dirty = true;
        self
    }

    pub fn with_render_func<F>(mut self, render_func: F) -> Self
    where
        F: Fn(&mut Scene, &Device, &Queue, f32, f32, f32, f32) -> Result<(), WidgetError> + Send + Sync + 'static,
    {
        self.render_func = Some(Box::new(render_func));
        self.dirty = true;
        self
    }

    pub fn with_direct_render_func<F>(mut self, direct_render_func: F) -> Self
    where
        F: Fn(&Device, &Queue, &wgpu::TextureView, u32, u32, f32, f32, f32, f32) -> Result<(), Box<dyn std::error::Error>> + Send + Sync + 'static,
    {
        self.direct_render_func = Some(Box::new(direct_render_func));
        self.dirty = true;
        self
    }

    pub fn with_shared_encoder_render_func<F>(mut self, shared_encoder_render_func: F) -> Self
    where
        F: Fn(&Device, &Queue, &mut CommandEncoder, &[ExternalResource], f32, f32, f32, f32) -> Result<(), vello::Error> + Send + Sync + 'static,
    {
        self.shared_encoder_render_func = Some(Arc::new(shared_encoder_render_func));
        self.dirty = true;
        self
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        if self.x != x || self.y != y {
            self.x = x;
            self.y = y;
            self.dirty = true;
        }
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        if self.width != width || self.height != height {
            self.width = width;
            self.height = height;
            self.dirty = true;
        }
    }

    pub fn set_render_func<F>(&mut self, render_func: F)
    where
        F: Fn(&mut Scene, &Device, &Queue, f32, f32, f32, f32) -> Result<(), WidgetError> + Send + Sync + 'static,
    {
        self.render_func = Some(Box::new(render_func));
        self.dirty = true;
    }

    pub fn clear_render_func(&mut self) {
        self.render_func = None;
        self.dirty = true;
    }

    pub fn set_direct_render_func<F>(&mut self, direct_render_func: F)
    where
        F: Fn(&Device, &Queue, &wgpu::TextureView, u32, u32, f32, f32, f32, f32) -> Result<(), Box<dyn std::error::Error>> + Send + Sync + 'static,
    {
        self.direct_render_func = Some(Box::new(direct_render_func));
        self.dirty = true;
    }

    pub fn clear_direct_render_func(&mut self) {
        self.direct_render_func = None;
        self.dirty = true;
    }

    pub fn set_shared_encoder_render_func<F>(&mut self, shared_encoder_render_func: F)
    where
        F: Fn(&Device, &Queue, &mut CommandEncoder, &[ExternalResource], f32, f32, f32, f32) -> Result<(), vello::Error> + Send + Sync + 'static,
    {
        self.shared_encoder_render_func = Some(Arc::new(shared_encoder_render_func));
        self.dirty = true;
    }

    pub fn clear_shared_encoder_render_func(&mut self) {
        self.shared_encoder_render_func = None;
        self.dirty = true;
    }

    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.width, self.height)
    }

    pub fn render_to_scene(&self, scene: &mut Scene, device: &Device, queue: &Queue) -> Result<(), WidgetError> {
        if let Some(ref render_func) = self.render_func {
            render_func(scene, device, queue, self.x, self.y, self.width, self.height)?;
        }
        Ok(())
    }

    pub fn has_direct_render_func(&self) -> bool {
        self.direct_render_func.is_some()
    }

    pub fn has_shared_encoder_render_func(&self) -> bool {
        self.shared_encoder_render_func.is_some()
    }

    pub fn execute_direct_render(&self, device: &Device, queue: &Queue, view: &wgpu::TextureView, view_width: u32, view_height: u32) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref direct_render_func) = self.direct_render_func {
            direct_render_func(device, queue, view, view_width, view_height, self.x, self.y, self.width, self.height)?;
        }
        Ok(())
    }

    /// Creates a shared encoder render function that captures the canvas position/size
    pub fn create_shared_encoder_render_func(&self) -> Option<impl Fn(&Device, &Queue, &mut CommandEncoder, &[ExternalResource]) -> Result<(), vello::Error> + Send + Sync + 'static> {
        let func = Arc::clone(self.shared_encoder_render_func.as_ref()?);
        let x = self.x;
        let y = self.y;
        let width = self.width;
        let height = self.height;
        
        Some(move |device: &Device, queue: &Queue, encoder: &mut CommandEncoder, external_resources: &[ExternalResource]| -> Result<(), vello::Error> {
            func(device, queue, encoder, external_resources, x, y, width, height)
        })
    }
}

impl Widget for CanvasWidget {
    fn mount(&mut self) -> Result<(), WidgetError> {
        self.dirty = true;
        Ok(())
    }

    fn unmount(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }

    fn update(&mut self, ctx: &mut dyn WidgetUpdateContext) -> Result<(), WidgetError> {
        if self.dirty {
            ctx.mark_dirty(self.id);
        }
        Ok(())
    }

    fn handle_event(&mut self, _event: &Event) -> EventResult {
        EventResult::Ignored
    }

    fn needs_layout(&self) -> bool {
        self.dirty
    }

    fn needs_render(&self) -> bool {
        self.dirty || self.render_func.is_some()
    }

    fn render(&self) -> Result<RenderData, WidgetError> {
        let dirty_region = DirtyRegion {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
        };

        Ok(RenderData {
            dirty_regions: vec![dirty_region],
            z_index: self.z_index,
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_id(&self) -> WidgetId {
        self.id
    }
}

pub fn canvas() -> CanvasWidget {
    CanvasWidget::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use vello::peniko::Color;
    use vello::kurbo::RoundedRect;

    #[test]
    fn test_canvas_creation() {
        let canvas = CanvasWidget::new();
        assert_eq!(canvas.width, 100.0);
        assert_eq!(canvas.height, 100.0);
        assert!(canvas.dirty);
        assert!(canvas.render_func.is_none());
    }

    #[test]
    fn test_canvas_with_size() {
        let canvas = CanvasWidget::new().with_size(200.0, 150.0);
        assert_eq!(canvas.width, 200.0);
        assert_eq!(canvas.height, 150.0);
    }

    #[test]
    fn test_canvas_with_position() {
        let canvas = CanvasWidget::new().with_position(10.0, 20.0);
        assert_eq!(canvas.x, 10.0);
        assert_eq!(canvas.y, 20.0);
    }

    #[test]
    fn test_canvas_bounds() {
        let canvas = CanvasWidget::new()
            .with_position(5.0, 10.0)
            .with_size(100.0, 80.0);
        let (x, y, width, height) = canvas.bounds();
        assert_eq!(x, 5.0);
        assert_eq!(y, 10.0);
        assert_eq!(width, 100.0);
        assert_eq!(height, 80.0);
    }
}