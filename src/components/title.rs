use std::sync::Mutex;

use super::{Component, Event, Painter};
use crate::error::MyBarError;
use xcb_wm::ewmh;

pub struct Title<'a> {
    x: i16,
    y: i16,
    width: u16,
    height: u16,
    painter: &'a Painter<'a>,
    conn: &'a ewmh::Connection<'a>,
}

impl<'a> Title<'a> {
    pub fn new(painter: &'a Painter, conn: &'a ewmh::Connection) -> Self {
        Self {
            x: 0,
            y: 0,
            width: 300,
            height: 40,
            painter,
            conn,
        }
    }
}

impl<'a> Component for Title<'a> {
    fn draw(&self) -> Result<(), MyBarError> {
        let title = get_current_wm_title(self.conn)?;
        let tw = self.painter.text_width(&title)?;
        let w = tw + 30.0 * 2.0;

        self.painter
            .draw_rounded_background(self.x as f64, w, 10.0, "#475164")?;
        self.painter
            .draw_text(self.x as f64 + 30.0, 10.0, &title, "#ff3329")?;

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
                    println!("Title clicked at ({}, {})", x, y);
                    // TODO: 实现标题组件点击逻辑
                }
            }
            Event::KeyPress { keycode } => {
                // TODO: 实现标题组件键盘控制逻辑
            }
        }
        Ok(())
    }

    fn get_bounds(&self) -> (i16, i16, u16, u16) {
        (self.x, self.y, self.width, self.height)
    }
}

fn get_current_window(conn: &ewmh::Connection) -> Result<xcb::x::Window, MyBarError> {
    let reply = conn.wait_for_reply(conn.send_request(&ewmh::proto::GetActiveWindow))?;
    Ok(reply.window)
}

fn get_current_wm_title(conn: &ewmh::Connection) -> Result<String, MyBarError> {
    let window = get_current_window(conn)?;
    let reply = conn.wait_for_reply(conn.send_request(&ewmh::proto::GetWmName(window)))?;
    Ok(reply.name)
}

