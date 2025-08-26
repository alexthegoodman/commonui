use vello::{Scene, kurbo::{Affine, Stroke as VelloStroke}, peniko::{Color, Fill}};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub enum RenderCommand {
    FillRect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
        transform: Affine,
    },
    StrokeRect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
        stroke_width: f32,
        transform: Affine,
    },
    FillRoundedRect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        radius: f32,
        color: Color,
        transform: Affine,
    },
    Text {
        x: f32,
        y: f32,
        text: String,
        color: Color,
        font_size: f32,
        transform: Affine,
    },
}

impl RenderCommand {
    pub fn can_batch_with(&self, other: &Self) -> bool {
        match (self, other) {
            // Same-type commands with similar properties can be batched
            (RenderCommand::FillRect { color: c1, transform: t1, .. }, 
             RenderCommand::FillRect { color: c2, transform: t2, .. }) => {
                c1 == c2 && t1 == t2
            },
            (RenderCommand::StrokeRect { color: c1, stroke_width: w1, transform: t1, .. }, 
             RenderCommand::StrokeRect { color: c2, stroke_width: w2, transform: t2, .. }) => {
                c1 == c2 && w1 == w2 && t1 == t2
            },
            (RenderCommand::Text { color: c1, font_size: s1, transform: t1, .. }, 
             RenderCommand::Text { color: c2, font_size: s2, transform: t2, .. }) => {
                c1 == c2 && s1 == s2 && t1 == t2
            },
            _ => false,
        }
    }

    pub fn draw_to_scene(&self, scene: &mut Scene) {
        match self {
            RenderCommand::FillRect { x, y, width, height, color, transform } => {
                let rect = vello::kurbo::Rect::new(*x as f64, *y as f64, 
                                                   (*x + *width) as f64, (*y + *height) as f64);
                scene.fill(Fill::NonZero, *transform, *color, None, &rect);
            },
            RenderCommand::StrokeRect { x, y, width, height, color, stroke_width, transform } => {
                let rect = vello::kurbo::Rect::new(*x as f64, *y as f64, 
                                                   (*x + *width) as f64, (*y + *height) as f64);
                let stroke = VelloStroke::new(*stroke_width as f64);
                scene.stroke(&stroke, *transform, *color, None, &rect);
            },
            RenderCommand::FillRoundedRect { x, y, width, height, radius, color, transform } => {
                let rect = vello::kurbo::Rect::new(*x as f64, *y as f64, 
                                                   (*x + *width) as f64, (*y + *height) as f64);
                let rounded_rect = vello::kurbo::RoundedRect::from_rect(rect, *radius as f64);
                scene.fill(Fill::NonZero, *transform, *color, None, &rounded_rect);
            },
            RenderCommand::Text { x, y, text, color, font_size, transform } => {
                // Use a simplified text rendering approach
                // In a real implementation, this would use a shared FontSystem and SwashCache
                let text_height = *font_size;
                
                // Create individual rectangles for each character to simulate text
                for (i, _) in text.char_indices() {
                    let char_x = *x + (i as f32 * font_size * 0.6);
                    let char_rect = vello::kurbo::Rect::new(
                        char_x as f64, 
                        *y as f64, 
                        (char_x + font_size * 0.5) as f64, 
                        (*y + text_height) as f64
                    );
                    scene.fill(Fill::NonZero, *transform, *color, None, &char_rect);
                }
            },
        }
    }
}

#[derive(Debug)]
pub struct RenderBatch {
    commands: Vec<RenderCommand>,
    layer: u32,
    blend_mode: BlendMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
}

impl RenderBatch {
    pub fn new(layer: u32, blend_mode: BlendMode) -> Self {
        Self {
            commands: Vec::new(),
            layer,
            blend_mode,
        }
    }

    pub fn add_command(&mut self, command: RenderCommand) {
        self.commands.push(command);
    }

    pub fn can_merge_with(&self, other: &Self) -> bool {
        self.layer == other.layer && self.blend_mode == other.blend_mode
    }

    pub fn merge(&mut self, other: RenderBatch) {
        if self.can_merge_with(&other) {
            self.commands.extend(other.commands);
        }
    }

    pub fn draw_to_scene(&self, scene: &mut Scene) {
        for command in &self.commands {
            command.draw_to_scene(scene);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn layer(&self) -> u32 {
        self.layer
    }
}

pub struct BatchRenderer {
    batches: VecDeque<RenderBatch>,
    current_layer: u32,
    max_batch_size: usize,
}

impl BatchRenderer {
    pub fn new() -> Self {
        Self {
            batches: VecDeque::new(),
            current_layer: 0,
            max_batch_size: 1000,
        }
    }

    pub fn with_max_batch_size(max_batch_size: usize) -> Self {
        Self {
            batches: VecDeque::new(),
            current_layer: 0,
            max_batch_size,
        }
    }

    pub fn set_layer(&mut self, layer: u32) {
        self.current_layer = layer;
    }

    pub fn add_command(&mut self, command: RenderCommand, blend_mode: BlendMode) {
        // Try to find an existing batch to add to
        for batch in &mut self.batches {
            if batch.layer == self.current_layer && 
               batch.blend_mode == blend_mode && 
               batch.len() < self.max_batch_size {
                
                // Check if we can batch this command
                if batch.commands.is_empty() || 
                   batch.commands.last().map_or(false, |last| last.can_batch_with(&command)) {
                    batch.add_command(command);
                    return;
                }
            }
        }

        // If we couldn't add to an existing batch, create a new one
        let mut new_batch = RenderBatch::new(self.current_layer, blend_mode);
        new_batch.add_command(command);
        
        // Insert in layer order
        let insert_pos = self.batches
            .iter()
            .position(|b| b.layer > self.current_layer)
            .unwrap_or(self.batches.len());
        
        self.batches.insert(insert_pos, new_batch);
    }

    pub fn render_batches(&mut self, scene: &mut Scene) {
        // Sort batches by layer for proper rendering order
        let mut sorted_batches: Vec<_> = self.batches.drain(..).collect();
        sorted_batches.sort_by_key(|b| b.layer);
        
        for batch in sorted_batches {
            batch.draw_to_scene(scene);
        }
    }

    pub fn clear(&mut self) {
        self.batches.clear();
        self.current_layer = 0;
    }

    pub fn batch_count(&self) -> usize {
        self.batches.len()
    }

    pub fn total_commands(&self) -> usize {
        self.batches.iter().map(|b| b.len()).sum()
    }

    pub fn optimize_batches(&mut self) {
        // Merge compatible adjacent batches
        let mut i = 0;
        while i < self.batches.len().saturating_sub(1) {
            let can_merge = {
                let (left, right) = self.batches.as_slices();
                if i < left.len() && i + 1 < left.len() + right.len() {
                    if i + 1 < left.len() {
                        left[i].can_merge_with(&left[i + 1])
                    } else {
                        left[i].can_merge_with(&right[i + 1 - left.len()])
                    }
                } else {
                    false
                }
            };
            
            if can_merge {
                let next_batch = self.batches.remove(i + 1).unwrap();
                if let Some(current_batch) = self.batches.get_mut(i) {
                    current_batch.merge(next_batch);
                }
            } else {
                i += 1;
            }
        }
    }
}

impl Default for BatchRenderer {
    fn default() -> Self {
        Self::new()
    }
}