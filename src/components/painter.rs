use cairo;
use xcb::{x, Xid};
use crate::error::MyBarError;

use crate::util;

pub struct Painter<'a> {
    width: i32,
    height: i32,
    conn: &'a xcb::Connection,
    pub cairo_conn: cairo::Context,
}

impl<'a> Painter<'a> {
    pub fn new(
        conn: &'a xcb::Connection,
        window: xcb::x::Window,
        visual_type: xcb::x::Visualtype,
        width: i32,
        height: i32,
    ) -> Result<Self, MyBarError> {
        let surface = create_surface(conn, window, visual_type, width, height)?;
        let cairo_conn = cairo::Context::new(surface)?;
        cairo_conn.select_font_face(
            "Maple Mono NL NF CN",
            cairo::FontSlant::Normal,
            cairo::FontWeight::Normal,
        );
        cairo_conn.set_font_size(14.0);
        Ok(Painter {
            width,
            height,
            conn,
            cairo_conn,
        })
    }
    pub fn flush(&self) -> Result<(), MyBarError> {
        self.conn.flush().map_err(|_| MyBarError::Other("Flush error".to_string()))?;
        Ok(())
    }
    pub fn text_width(&self, text: &str) -> Result<f64, MyBarError> {
        Ok(self.cairo_conn.text_extents(text)?.width())
    }

    pub fn set_hex_color(&self, color: &str) -> Result<(), MyBarError> {
        let (a, r, g, b) = util::hex_to_argb(color).map_err(|_| MyBarError::Other("Invalid hex color".to_string()))?;
        self.cairo_conn.set_source_rgba(r, g, b, a);
        Ok(())
    }

    pub fn draw_rectangle(&self, x: f64, width: f64, color: &str) -> Result<(), MyBarError> {
        let height: f64 = self.height as f64 - 10.0;
        let y = (self.height as f64 - height) / 2.0;
        self.set_hex_color(color)?;
        self.cairo_conn.rectangle(x, y, width, height);
        self.cairo_conn.fill()?;
        Ok(())
    }

    pub fn draw_text(&self, x: f64, y: f64, text: &str, color: &str) -> Result<(), MyBarError> {
        self.set_hex_color(color)?;
        let fe = self.cairo_conn.font_extents()?;
        let y = self.height as f64 / 2.0 + (fe.ascent() - fe.descent()) / 2.0;
        self.cairo_conn.move_to(x, y);
        self.cairo_conn.show_text(text)?;
        Ok(())
    }

    pub fn draw_rounded_background(
        &self,
        x: f64,
        width: f64,
        radius: f64,
        color: &str,
    ) -> Result<(), MyBarError> {
        let height: f64 = self.height as f64 - 10.0;
        let y = (self.height as f64 - height) / 2.0;
        self.set_hex_color(color)?;
        self.cairo_conn.move_to(x + radius, y);
        self.cairo_conn.line_to(x + width - radius, y);
        self.cairo_conn.arc(
            x + width - radius,
            y + radius,
            radius,
            -90.0_f64.to_radians(),
            0.0_f64.to_radians(),
        );
        self.cairo_conn.line_to(x + width, y + height - radius);
        self.cairo_conn.arc(
            x + width - radius,
            y + height - radius,
            radius,
            0.0_f64,
            90.0_f64.to_radians(),
        );
        self.cairo_conn.line_to(x + radius, y + height);
        self.cairo_conn.arc(
            x + radius,
            y + height - radius,
            radius,
            90.0_f64.to_radians(),
            180_f64.to_radians(),
        );
        self.cairo_conn.line_to(x, y + radius);
        self.cairo_conn.arc(
            x + radius,
            y + radius,
            radius,
            180_f64.to_radians(),
            270_f64.to_radians(),
        );
        self.cairo_conn.close_path();
        self.cairo_conn.fill()?;
        Ok(())
    }
}

fn create_surface(
    conn: &xcb::Connection,
    window: xcb::x::Window,
    visual_type: xcb::x::Visualtype,
    width: i32,
    height: i32,
) -> Result<cairo::XCBSurface, MyBarError> {
    unsafe {
        let conn = cairo::XCBConnection::from_raw_none(
            conn.get_raw_conn() as *mut cairo_sys::xcb_connection_t
        );
        let mut visual_type = visual_type;
        let surface = cairo::XCBSurface::create(
            &conn,
            &cairo::XCBDrawable(window.resource_id()),
            &cairo::XCBVisualType::from_raw_none(
                (&mut visual_type) as *mut xcb::x::Visualtype as *mut cairo_sys::xcb_visualtype_t,
            ),
            width,
            height,
        )?;
        Ok(surface)
    }
} 