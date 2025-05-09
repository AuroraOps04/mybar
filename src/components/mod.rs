pub mod bspwm;
pub mod date;
pub mod light;
pub mod painter;
pub mod title;
pub mod volume;

use crate::error::MyBarError;

pub trait Component {
    fn draw(&self) -> Result<(), MyBarError>;
    fn contains_point(&self, x: i16, y: i16) -> bool;
    fn handle_event(&self, event: &Event) -> Result<(), MyBarError>;
    fn get_bounds(&self) -> (i16, i16, u16, u16); // x, y, width, height
}

pub enum Event {
    MouseClick { x: i16, y: i16, button: u8 },
    KeyPress { keycode: u8 },
}

pub use bspwm::BspwmComponent;
pub use date::Date;
pub use light::Light;
pub use painter::Painter;
pub use title::Title;
pub use volume::Volume;
