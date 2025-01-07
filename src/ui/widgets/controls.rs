use crate::error::Result;
use super::{Widget, WidgetEvent};

pub struct Button {
    label: String,
    position: (f32, f32),
    size: (f32, f32),
    is_enabled: bool,
    is_pressed: bool,
    on_click: Option<Box<dyn Fn() -> Result<()>>>,
}

impl Button {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            position: (0.0, 0.0),
            size: (100.0, 30.0), // Default size
            is_enabled: true,
            is_pressed: false,
            on_click: None,
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position = (x, y);
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        self.size = (width, height);
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.is_enabled = enabled;
    }

    pub fn on_click<F>(&mut self, callback: F)
    where
        F: Fn() -> Result<()> + 'static,
    {
        self.on_click = Some(Box::new(callback));
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

impl Widget for Button {
    fn render(&self) -> Result<()> {
        // Basic rendering for now
        Ok(())
    }

    fn handle_interaction(&self, _event: WidgetEvent) -> Result<()> {
        // Basic interaction handling for now
        Ok(())
    }
}

pub struct Slider {
    value: f32,
    range: (f32, f32),
    position: (f32, f32),
    size: (f32, f32),
    is_enabled: bool,
    is_dragging: bool,
    on_change: Option<Box<dyn Fn(f32) -> Result<()>>>,
}

impl Slider {
    pub fn new(min: f32, max: f32) -> Self {
        Self {
            value: min,
            range: (min, max),
            position: (0.0, 0.0),
            size: (200.0, 20.0), // Default size
            is_enabled: true,
            is_dragging: false,
            on_change: None,
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position = (x, y);
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        self.size = (width, height);
    }

    pub fn set_value(&mut self, value: f32) -> Result<()> {
        let (min, max) = self.range;
        self.value = value.clamp(min, max);
        
        if let Some(callback) = &self.on_change {
            callback(self.value)?;
        }
        
        Ok(())
    }

    pub fn on_change<F>(&mut self, callback: F)
    where
        F: Fn(f32) -> Result<()> + 'static,
    {
        self.on_change = Some(Box::new(callback));
    }

    pub fn value(&self) -> f32 {
        self.value
    }
}

impl Widget for Slider {
    fn render(&self) -> Result<()> {
        // Basic rendering for now
        Ok(())
    }

    fn handle_interaction(&self, _event: WidgetEvent) -> Result<()> {
        // Basic interaction handling for now
        Ok(())
    }
}

pub struct Checkbox {
    label: String,
    is_checked: bool,
    position: (f32, f32),
    size: (f32, f32),
    is_enabled: bool,
    on_change: Option<Box<dyn Fn(bool) -> Result<()>>>,
}

impl Checkbox {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            is_checked: false,
            position: (0.0, 0.0),
            size: (20.0, 20.0), // Default size
            is_enabled: true,
            on_change: None,
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position = (x, y);
    }

    pub fn set_checked(&mut self, checked: bool) -> Result<()> {
        self.is_checked = checked;
        
        if let Some(callback) = &self.on_change {
            callback(self.is_checked)?;
        }
        
        Ok(())
    }

    pub fn on_change<F>(&mut self, callback: F)
    where
        F: Fn(bool) -> Result<()> + 'static,
    {
        self.on_change = Some(Box::new(callback));
    }

    pub fn is_checked(&self) -> bool {
        self.is_checked
    }
}

impl Widget for Checkbox {
    fn render(&self) -> Result<()> {
        // Basic rendering for now
        Ok(())
    }

    fn handle_interaction(&self, _event: WidgetEvent) -> Result<()> {
        // Basic interaction handling for now
        Ok(())
    }
} 