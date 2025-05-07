use super::{Component, Event, Painter};
use crate::error::MyBarError;

pub struct Volume<'a> {
    x: i16,
    y: i16,
    width: u16,
    height: u16,
    painter: &'a Painter<'a>,
    audio: &'a crate::alsa::Audio,
}

impl<'a> Volume<'a> {
    pub fn new(painter: &'a Painter, audio: &'a crate::alsa::Audio) -> Self {
        Self {
            x: 1420,
            y: 0,
            width: 100,
            height: 40,
            painter,
            audio,
        }
    }
}

impl<'a> Component for Volume<'a> {
    fn draw(&self) -> Result<(), MyBarError> {
        let v = self.audio.get_current_volume();
        let unmuted = self.audio.is_unmuted();
        // 根据静音状态选择不同的图标和颜色
        let (icon, color) = if unmuted {
            ("", "#ff3399") // 未静音时使用粉色
        } else {
            ("", "#666666") // 静音时使用灰色
        };
        let te = self.painter.text_width(&icon)?;
        let iw = te + self.width as f64 + 5.0;

        self.painter
            .draw_rounded_background(self.x as f64, iw + 10.0 * 2., 10.0, "#475164")?;
        self.painter
            .draw_text(self.x as f64 + 10.0, 10.0, &icon, color)?;

        let rw = self.width as f64 * v;
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
                if button == 1 {
                    // 左键
                    self.audio.toggle_mute();
                    self.draw()?;
                } else if button == 2 {
                    // 中键
                    println!("Volume up clicked at ({}, {})", x, y);
                } else if button == 3 {
                    // 右键
                    println!("Volume down clicked at ({}, {})", x, y);
                } else if button == 4 {
                    // 滚轮上
                    let current_volume = self.audio.get_current_volume();
                    self.audio
                        .set_current_volumn((current_volume + 0.05).min(1.0));
                    self.draw()?;
                } else if button == 5 {
                    // 滚轮下
                    let current_volume = self.audio.get_current_volume();
                    self.audio
                        .set_current_volumn((current_volume - 0.05).max(0.0));
                    self.draw()?;
                }
            }
            Event::KeyPress { keycode } => {
                // TODO: 实现键盘控制音量逻辑
            }
        }
        self.painter.flush()?;
        Ok(())
    }

    fn get_bounds(&self) -> (i16, i16, u16, u16) {
        (self.x, self.y, self.width, self.height)
    }
}
