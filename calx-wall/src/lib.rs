//! 2D drawing utilities
//!

#[macro_use]
extern crate glium;
extern crate image;
extern crate cgmath;
extern crate calx_alg;
extern crate calx_window;
extern crate calx_color;
extern crate calx_layout;
extern crate calx_cache;

pub use draw_util::DrawUtil;
pub use font::{Font, Fonter, Align};
pub use wall::{Wall, Vertex};

mod draw_util;
mod font;
mod wall;

/// UI Widget static identifier, unique for a specific site in source code.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WidgetId {
    filename: &'static str,
    line: u32,
    column: u32,
}

impl WidgetId {
    pub fn new(filename: &'static str, line: u32, column: u32) -> WidgetId {
        WidgetId {
            filename: filename,
            line: line,
            column: column,
        }
    }

    pub fn dummy() -> WidgetId {
        WidgetId {
            filename: "n/a",
            line: 666666,
            column: 666666,
        }
    }
}

#[macro_export]
/// Generate a static identifier for the current source code position. Used
/// with imgui API.
macro_rules! widget_id {
    () => {
        ::calx::backend::WidgetId::new(concat!(module_path!(), "/", file!()), line!(), column!())
    }
}
