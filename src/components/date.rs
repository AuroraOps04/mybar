use super::{Component, Event, Painter};
use crate::error::MyBarError;
use chrono;

pub struct Date<'a> {
    x: i16,
    y: i16,
    width: u16,
    height: u16,
    painter: &'a Painter<'a>,
}

impl<'a> Date<'a> {
    pub fn new(painter: &'a Painter<'a>) -> Self {
        Self {
            x: 820,
            y: 0,
            width: 200,
            height: 40,
            painter,
        }
    }

    pub fn flush(&self) -> Result<(), MyBarError> {
        self.painter.flush()?;
        Ok(())
    }
}

impl Component for Date<'_> {
    fn draw(&self) -> Result<(), MyBarError> {
        let now = chrono::Local::now();
        let text = now.format("%a %b %e %T %Y").to_string();
        let tw = self.painter.text_width(&text)?;
        let w = tw + 30.0 * 2.0;

        self.painter
            .draw_rounded_background(self.x as f64, w, 10.0, "#475164")?;
        self.painter
            .draw_text(self.x as f64 + 30.0, 10.0, &text, "#ff3329")?;

        Ok(())
    }

    fn contains_point(&self, x: i16, y: i16) -> bool {
        x >= self.x
            && x <= self.x + self.width as i16
            && y >= self.y
            && y <= self.y + self.height as i16
    }

    fn handle_event(&self, event: &Event) -> Result<(), MyBarError> {
        match event {
            Event::MouseClick { x, y, button } => {
                if self.contains_point(*x, *y) {
                    println!("Date clicked at ({}, {})", x, y);
                    // TODO: 实现日期组件点击逻辑
                }
            }
            Event::KeyPress { keycode } => {
                // TODO: 实现日期组件键盘控制逻辑
            }
        }
        Ok(())
    }

    fn get_bounds(&self) -> (i16, i16, u16, u16) {
        (self.x, self.y, self.width, self.height)
    }
}
