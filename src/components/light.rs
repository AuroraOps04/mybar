use super::{Component, Event, Painter};
use crate::error::MyBarError;
use crate::light;

pub struct Light<'a> {
    x: i16,
    y: i16,
    width: u16,
    height: u16,
    painter: &'a Painter<'a>,
}

impl<'a> Light<'a> {
    pub fn new(painter: &'a Painter) -> Self {
        Self {
            x: 1120,
            y: 0,
            width: 100,
            height: 40,
            painter,
        }
    }
}

impl<'a> Component for Light<'a> {
    fn draw(&self) -> Result<(), MyBarError> {
        let brightness = light::get_light() as f64 / 100.0;
        let icon = "";
        let color = "#ffcc00";
        let te = self.painter.text_width(&icon)?;
        let iw = te + self.width as f64 + 5.0;

        self.painter
            .draw_rounded_background(self.x as f64, iw + 10.0 * 2., 10.0, "#475164")?;
        self.painter
            .draw_text(self.x as f64 + 10.0, 10.0, &icon, color)?;

        let rw = self.width as f64 * brightness;
        self.painter.set_hex_color(&color)?;
        self.painter
            .cairo_conn
            .move_to(self.x as f64 + 10.0 + te + 5.0, 20.0);
        self.painter
            .cairo_conn
            .line_to(self.x as f64 + 10.0 + te + 5.0 + rw, 20.0);
        self.painter.cairo_conn.stroke()?;
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
                let button = *button;
                if button == 4 {
                    // 滚轮上
                    let current_brightness = light::get_light();
                    light::set_light((current_brightness + 5).min(100));
                    self.draw()?;
                } else if button == 5 {
                    // 滚轮下
                    let current_brightness = light::get_light();
                    light::set_light((current_brightness as i32 - 5).max(0) as u16);
                    self.draw()?;
                }
            }
            Event::KeyPress { keycode } => {
                // TODO: 实现键盘控制亮度逻辑
            }
        }
        self.painter.flush()?;
        Ok(())
    }

    fn get_bounds(&self) -> (i16, i16, u16, u16) {
        (self.x, self.y, self.width, self.height)
    }
} 