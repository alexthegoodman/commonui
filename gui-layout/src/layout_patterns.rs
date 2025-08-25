use taffy::{
    Style, FlexDirection, JustifyContent, AlignItems, AlignContent, FlexWrap,
    Dimension, LengthPercentage, LengthPercentageAuto, Position, Display
};
use gui_reactive::Signal;
use crate::ReactiveLayoutManager;

/// Builder for flexbox layouts with common patterns
#[derive(Debug, Clone)]
pub struct FlexLayoutBuilder {
    style: Style,
}

impl FlexLayoutBuilder {
    pub fn new() -> Self {
        Self {
            style: Style {
                display: Display::Flex,
                ..Default::default()
            }
        }
    }

    /// Create a horizontal (row) flex container
    pub fn row() -> Self {
        Self::new().direction(FlexDirection::Row)
    }

    /// Create a vertical (column) flex container
    pub fn column() -> Self {
        Self::new().direction(FlexDirection::Column)
    }

    /// Set flex direction
    pub fn direction(mut self, direction: FlexDirection) -> Self {
        self.style.flex_direction = direction;
        self
    }

    /// Set main axis alignment (justify-content)
    pub fn justify_content(mut self, justify: JustifyContent) -> Self {
        self.style.justify_content = Some(justify);
        self
    }

    /// Set cross axis alignment (align-items)
    pub fn align_items(mut self, align: AlignItems) -> Self {
        self.style.align_items = Some(align);
        self
    }

    /// Set content alignment (align-content)
    pub fn align_content(mut self, align: AlignContent) -> Self {
        self.style.align_content = Some(align);
        self
    }

    /// Set flex wrap
    pub fn flex_wrap(mut self, wrap: FlexWrap) -> Self {
        self.style.flex_wrap = wrap;
        self
    }

    /// Set gap between items
    pub fn gap(mut self, gap: f32) -> Self {
        self.style.gap.width = LengthPercentage::Length(gap);
        self.style.gap.height = LengthPercentage::Length(gap);
        self
    }

    /// Set row gap
    pub fn row_gap(mut self, gap: f32) -> Self {
        self.style.gap.height = LengthPercentage::Length(gap);
        self
    }

    /// Set column gap
    pub fn column_gap(mut self, gap: f32) -> Self {
        self.style.gap.width = LengthPercentage::Length(gap);
        self
    }

    /// Set padding
    pub fn padding(mut self, padding: f32) -> Self {
        self.style.padding.left = LengthPercentage::Length(padding);
        self.style.padding.right = LengthPercentage::Length(padding);
        self.style.padding.top = LengthPercentage::Length(padding);
        self.style.padding.bottom = LengthPercentage::Length(padding);
        self
    }

    /// Set padding for specific sides
    pub fn padding_sides(mut self, left: f32, right: f32, top: f32, bottom: f32) -> Self {
        self.style.padding.left = LengthPercentage::Length(left);
        self.style.padding.right = LengthPercentage::Length(right);
        self.style.padding.top = LengthPercentage::Length(top);
        self.style.padding.bottom = LengthPercentage::Length(bottom);
        self
    }

    /// Set margin
    pub fn margin(mut self, margin: f32) -> Self {
        self.style.margin.left = LengthPercentageAuto::Length(margin);
        self.style.margin.right = LengthPercentageAuto::Length(margin);
        self.style.margin.top = LengthPercentageAuto::Length(margin);
        self.style.margin.bottom = LengthPercentageAuto::Length(margin);
        self
    }

    /// Set width
    pub fn width(mut self, width: f32) -> Self {
        self.style.size.width = Dimension::Length(width);
        self
    }

    /// Set height
    pub fn height(mut self, height: f32) -> Self {
        self.style.size.height = Dimension::Length(height);
        self
    }

    /// Set percentage width
    pub fn width_percent(mut self, percent: f32) -> Self {
        self.style.size.width = Dimension::Percent(percent / 100.0);
        self
    }

    /// Set percentage height
    pub fn height_percent(mut self, percent: f32) -> Self {
        self.style.size.height = Dimension::Percent(percent / 100.0);
        self
    }

    /// Build the style
    pub fn build(self) -> Style {
        self.style
    }

    /// Build as a reactive signal
    pub fn build_signal(self) -> Signal<Style> {
        Signal::new(self.style)
    }
}

/// Builder for flex item properties
#[derive(Debug, Clone)]
pub struct FlexItemBuilder {
    style: Style,
}

impl FlexItemBuilder {
    pub fn new() -> Self {
        Self {
            style: Style::default()
        }
    }

    /// Set flex grow
    pub fn flex_grow(mut self, grow: f32) -> Self {
        self.style.flex_grow = grow;
        self
    }

    /// Set flex shrink
    pub fn flex_shrink(mut self, shrink: f32) -> Self {
        self.style.flex_shrink = shrink;
        self
    }

    /// Set flex basis
    pub fn flex_basis(mut self, basis: f32) -> Self {
        self.style.flex_basis = Dimension::Length(basis);
        self
    }

    /// Set flex basis as percentage
    pub fn flex_basis_percent(mut self, percent: f32) -> Self {
        self.style.flex_basis = Dimension::Percent(percent / 100.0);
        self
    }

    /// Shorthand for flex: 1 (grow to fill available space)
    pub fn flex_1(mut self) -> Self {
        self.style.flex_grow = 1.0;
        self.style.flex_shrink = 1.0;
        self.style.flex_basis = Dimension::Percent(0.0);
        self
    }

    /// Set align self (override container's align-items for this item)
    pub fn align_self(mut self, align: Option<AlignItems>) -> Self {
        self.style.align_self = align;
        self
    }

    /// Set position
    pub fn position(mut self, position: Position) -> Self {
        self.style.position = position;
        self
    }

    /// Set absolute positioning
    pub fn absolute(mut self) -> Self {
        self.style.position = Position::Absolute;
        self
    }

    /// Set width and height
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.style.size.width = Dimension::Length(width);
        self.style.size.height = Dimension::Length(height);
        self
    }

    /// Build the style
    pub fn build(self) -> Style {
        self.style
    }

    /// Build as a reactive signal
    pub fn build_signal(self) -> Signal<Style> {
        Signal::new(self.style)
    }
}

/// Simplified grid-like layout builder using flexbox (for compatibility)
#[derive(Debug, Clone)]
pub struct GridLayoutBuilder {
    style: Style,
}

impl GridLayoutBuilder {
    pub fn new() -> Self {
        Self {
            style: Style {
                display: Display::Flex,
                flex_wrap: FlexWrap::Wrap,
                ..Default::default()
            }
        }
    }

    /// Create a grid-like layout with specified columns (using flex-basis)
    pub fn columns(mut self, _count: usize) -> Self {
        // For now, just use flexbox with wrap
        self.style.flex_direction = FlexDirection::Row;
        self.style.flex_wrap = FlexWrap::Wrap;
        self
    }

    /// Create a grid-like layout with specified rows
    pub fn rows(mut self, _count: usize) -> Self {
        // For now, just use flexbox with wrap
        self.style.flex_direction = FlexDirection::Column;
        self.style.flex_wrap = FlexWrap::Wrap;
        self
    }

    /// Set gap between grid items
    pub fn gap(mut self, gap: f32) -> Self {
        self.style.gap.width = LengthPercentage::Length(gap);
        self.style.gap.height = LengthPercentage::Length(gap);
        self
    }

    /// Set row gap
    pub fn row_gap(mut self, gap: f32) -> Self {
        self.style.gap.height = LengthPercentage::Length(gap);
        self
    }

    /// Set column gap
    pub fn column_gap(mut self, gap: f32) -> Self {
        self.style.gap.width = LengthPercentage::Length(gap);
        self
    }

    /// Set padding
    pub fn padding(mut self, padding: f32) -> Self {
        self.style.padding.left = LengthPercentage::Length(padding);
        self.style.padding.right = LengthPercentage::Length(padding);
        self.style.padding.top = LengthPercentage::Length(padding);
        self.style.padding.bottom = LengthPercentage::Length(padding);
        self
    }

    /// Set align items
    pub fn align_items(mut self, align: AlignItems) -> Self {
        self.style.align_items = Some(align);
        self
    }

    /// Set justify content
    pub fn justify_content(mut self, justify: JustifyContent) -> Self {
        self.style.justify_content = Some(justify);
        self
    }

    /// Set align content
    pub fn align_content(mut self, align: AlignContent) -> Self {
        self.style.align_content = Some(align);
        self
    }

    /// Build the style
    pub fn build(self) -> Style {
        self.style
    }

    /// Build as a reactive signal
    pub fn build_signal(self) -> Signal<Style> {
        Signal::new(self.style)
    }
}

/// Builder for flex item properties (simplified grid item)
#[derive(Debug, Clone)]
pub struct GridItemBuilder {
    style: Style,
}

impl GridItemBuilder {
    pub fn new() -> Self {
        Self {
            style: Style::default()
        }
    }

    /// Set flex basis to simulate grid column width
    pub fn column_span(mut self, _start: i16, _end: i16) -> Self {
        // Simplified: just use flex properties
        self.style.flex_grow = 1.0;
        self
    }

    /// Set flex direction for row spanning (simplified)
    pub fn row_span(mut self, _start: i16, _end: i16) -> Self {
        // Simplified: just use flex properties
        self.style.flex_grow = 1.0;
        self
    }

    /// Set a flex-based grid area (simplified)
    pub fn grid_area(mut self, _row_start: i16, _col_start: i16, _row_end: i16, _col_end: i16) -> Self {
        self.style.flex_grow = 1.0;
        self
    }

    /// Set align self
    pub fn align_self(mut self, align: Option<AlignItems>) -> Self {
        self.style.align_self = align;
        self
    }

    /// Set size for the item
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.style.size.width = Dimension::Length(width);
        self.style.size.height = Dimension::Length(height);
        self
    }

    /// Build the style
    pub fn build(self) -> Style {
        self.style
    }

    /// Build as a reactive signal
    pub fn build_signal(self) -> Signal<Style> {
        Signal::new(self.style)
    }
}

/// Common layout patterns implemented as functions
pub struct LayoutPatterns;

impl LayoutPatterns {
    /// Center content horizontally and vertically
    pub fn center() -> Style {
        FlexLayoutBuilder::new()
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center)
            .build()
    }

    /// Stack items vertically with spacing
    pub fn vertical_stack(gap: f32) -> Style {
        FlexLayoutBuilder::column()
            .gap(gap)
            .build()
    }

    /// Arrange items horizontally with spacing
    pub fn horizontal_stack(gap: f32) -> Style {
        FlexLayoutBuilder::row()
            .gap(gap)
            .build()
    }

    /// Create a responsive grid that adapts to content (simplified version)
    pub fn responsive_grid(columns: usize, gap: f32) -> Style {
        GridLayoutBuilder::new()
            .columns(columns)
            .gap(gap)
            .build()
    }

    /// Create a sidebar layout (sidebar + content)
    pub fn sidebar_layout(_sidebar_width: f32, gap: f32) -> Style {
        FlexLayoutBuilder::row()
            .gap(gap)
            .build()
    }

    /// Create a header-content-footer layout
    pub fn header_content_footer_layout() -> Style {
        FlexLayoutBuilder::column()
            .build()
    }

    /// Create a card layout with padding
    pub fn card_layout(padding: f32) -> Style {
        FlexLayoutBuilder::column()
            .padding(padding)
            .build()
    }

    /// Create a toolbar layout (horizontal with items at the ends)
    pub fn toolbar_layout(gap: f32) -> Style {
        FlexLayoutBuilder::row()
            .justify_content(JustifyContent::SpaceBetween)
            .align_items(AlignItems::Center)
            .gap(gap)
            .build()
    }
}

/// Trait for creating reactive layout containers with common patterns
pub trait ReactiveLayoutPatterns {
    /// Create a centered container
    fn create_centered_container(&self, node_id: u64) -> Result<(), taffy::TaffyError>;

    /// Create a vertical stack container
    fn create_vertical_stack(&self, node_id: u64, gap: f32) -> Result<(), taffy::TaffyError>;

    /// Create a horizontal stack container
    fn create_horizontal_stack(&self, node_id: u64, gap: f32) -> Result<(), taffy::TaffyError>;

    /// Create a grid container
    fn create_grid_container(&self, node_id: u64, columns: usize, gap: f32) -> Result<(), taffy::TaffyError>;

    /// Create a responsive grid container
    fn create_responsive_grid(&self, node_id: u64, columns: usize, gap: f32) -> Result<(), taffy::TaffyError>;
}

impl ReactiveLayoutPatterns for ReactiveLayoutManager {
    fn create_centered_container(&self, node_id: u64) -> Result<(), taffy::TaffyError> {
        let style_signal = Signal::new(LayoutPatterns::center());
        self.create_reactive_node(node_id, style_signal)
    }

    fn create_vertical_stack(&self, node_id: u64, gap: f32) -> Result<(), taffy::TaffyError> {
        let style_signal = Signal::new(LayoutPatterns::vertical_stack(gap));
        self.create_reactive_node(node_id, style_signal)
    }

    fn create_horizontal_stack(&self, node_id: u64, gap: f32) -> Result<(), taffy::TaffyError> {
        let style_signal = Signal::new(LayoutPatterns::horizontal_stack(gap));
        self.create_reactive_node(node_id, style_signal)
    }

    fn create_grid_container(&self, node_id: u64, columns: usize, gap: f32) -> Result<(), taffy::TaffyError> {
        let style = GridLayoutBuilder::new()
            .columns(columns)
            .gap(gap)
            .build();
        let style_signal = Signal::new(style);
        self.create_reactive_node(node_id, style_signal)
    }

    fn create_responsive_grid(&self, node_id: u64, columns: usize, gap: f32) -> Result<(), taffy::TaffyError> {
        let style_signal = Signal::new(LayoutPatterns::responsive_grid(columns, gap));
        self.create_reactive_node(node_id, style_signal)
    }
}

impl Default for FlexLayoutBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for FlexItemBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for GridLayoutBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for GridItemBuilder {
    fn default() -> Self {
        Self::new()
    }
}