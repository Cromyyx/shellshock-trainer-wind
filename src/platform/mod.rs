// src/platform/mod.rs

#[cfg(windows)]
pub mod windows;

// Trait defining platform-specific window/input interactions
pub trait Handle {
    /// Checks if a specific abstract key is currently pressed.
    fn is_key_pressed(&self, vk: VK) -> bool;

    /// Gets the client area dimensions of the window.
    fn get_window_rect(&self) -> Rect;

    /// Gets the mouse cursor position relative to the window's client area (0,0 upper-left).
    fn get_mouse_position_in_window(&self) -> Cursor;
}

/// Abstract Virtual Key representations for trainer actions.
#[derive(Debug, Clone, Copy)] // Added Clone, Copy for convenience
pub enum VK {
    Key1, // Set source position
    Key2, // Set target position
    Key3, // Get/Set Wind Input
    Key4, // Calculate Hits (using stored wind and dimensions)
    Key5, // Clear Positions and Wind (keeps cached dimensions)
    Key6, // Switch calculation mode (Angle/Velocity)
    Key7, // Cache current Game Window Dimensions
}

/// Represents the dimensions of a rectangle (like the window client area).
#[derive(Debug, Clone)] // Added Clone for caching
pub struct Rect {
    width: i32,
    height: i32,
}

impl Rect {
    pub fn new(width: i32, height: i32) -> Self {
        Rect { width, height }
    }

    pub fn get_width(&self) -> i32 {
        self.width
    }

    pub fn get_height(&self) -> i32 {
        self.height
    }
}

/// Represents a cursor position (like the mouse).
// Make Cursor clonable if needed for more complex state, though not strictly needed here yet
#[derive(Debug, Clone)]
pub struct Cursor {
    x: i32,
    y: i32,
}

impl Cursor {
    pub fn new(x: i32, y: i32) -> Self {
        Cursor { x, y }
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }
}