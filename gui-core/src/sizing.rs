#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unit {
    Fixed(f32),
    Perc(f32),
}

impl Unit {
    pub fn fixed(value: f32) -> Self {
        Unit::Fixed(value)
    }

    pub fn perc(value: f32) -> Self {
        Unit::Perc(value)
    }

    pub fn resolve(&self, available_space: f32) -> f32 {
        match self {
            Unit::Fixed(value) => *value,
            Unit::Perc(percentage) => available_space * (percentage / 100.0),
        }
    }

    pub fn is_percentage(&self) -> bool {
        matches!(self, Unit::Perc(_))
    }

    pub fn is_fixed(&self) -> bool {
        matches!(self, Unit::Fixed(_))
    }

    pub fn value(&self) -> f32 {
        match self {
            Unit::Fixed(value) => *value,
            Unit::Perc(value) => *value,
        }
    }
}

impl From<f32> for Unit {
    fn from(value: f32) -> Self {
        Unit::Fixed(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: Unit,
    pub height: Unit,
}

impl Size {
    pub fn new(width: Unit, height: Unit) -> Self {
        Self { width, height }
    }

    pub fn fixed(width: f32, height: f32) -> Self {
        Self {
            width: Unit::Fixed(width),
            height: Unit::Fixed(height),
        }
    }

    pub fn perc(width: f32, height: f32) -> Self {
        Self {
            width: Unit::Perc(width),
            height: Unit::Perc(height),
        }
    }

    pub fn resolve(&self, available_width: f32, available_height: f32) -> (f32, f32) {
        (
            self.width.resolve(available_width),
            self.height.resolve(available_height),
        )
    }
}

impl From<(f32, f32)> for Size {
    fn from((width, height): (f32, f32)) -> Self {
        Size::fixed(width, height)
    }
}

impl From<(Unit, Unit)> for Size {
    fn from((width, height): (Unit, Unit)) -> Self {
        Size::new(width, height)
    }
}