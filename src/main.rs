use std::thread;
use std::time::Duration;
use std::{panic::UnwindSafe, sync::Arc};
use xcb::{Xid, x};
use xcb_wm::ewmh;
mod alsa;
mod util;

fn create_surface(
    conn: &xcb::Connection,
    window: xcb::x::Window,
    visual_type: xcb::x::Visualtype,
    width: i32,
    height: i32,
) -> Result<cairo::XCBSurface, cairo::Error> {
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

pub struct Painter {
    width: i32,
    height: i32,
    audio: alsa::Audio,
    pub cairo_conn: cairo::Context,
}

impl Painter {
    fn new(
        conn: &xcb::Connection,
        window: xcb::x::Window,
        visual_type: xcb::x::Visualtype,
        width: i32,
        height: i32,
    ) -> Self {
        let surface = create_surface(conn, window, visual_type, width, height)
            .expect("failed to create surface");
        let cairo_conn = cairo::Context::new(surface).expect("faield create cairo context");
        cairo_conn.select_font_face(
            "Maple Mono NL NF CN",
            cairo::FontSlant::Normal,
            cairo::FontWeight::Normal,
        );
        cairo_conn.set_font_size(14.0);
        let audio = alsa::Audio::default();
        Painter {
            width,
            height,
            audio,
            cairo_conn,
        }
    }
    fn draw_volumn(&self) {
        let v = self.audio.get_current_volume();
        let unmuted = self.audio.unmuted;
        let x = 1700.;
        let rw = 100.;
        let gap = 10.;
        let te = self.cairo_conn.text_extents("").unwrap();
        let iw = te.width() + rw + 5.0;
        self.draw_rounded_background(x, iw + gap * 2., 10.0, "#475164")
            .unwrap();
        self.draw_text(x + gap, gap, "", "#ff3399").unwrap();
        let rw = rw * v;
        self.set_hex_color("#ff3399").unwrap();
        self.cairo_conn.move_to(x + gap + te.width() + 5.0, 20.);
        self.cairo_conn
            .line_to(x + gap + te.width() + 5.0 + rw, 20.);
        self.cairo_conn.stroke().unwrap();
    }

    fn text_width(&self, text: &str) -> Result<f64, cairo::Error> {
        self.cairo_conn.text_extents(text).map(|te| te.width())
    }
    fn set_hex_color(&self, color: &str) -> Result<(), cairo::Error> {
        let (a, r, g, b) = util::hex_to_argb(color).map_err(|_| cairo::Error::DwriteError)?;
        self.cairo_conn.set_source_rgba(r, g, b, a);
        Ok(())
    }

    fn draw_rectangle(&self, x: f64, width: f64, color: &str) -> Result<(), cairo::Error> {
        let height: f64 = self.height as f64 - 10.0;
        let y = (self.height as f64 - height) / 2.0;
        self.set_hex_color(color)?;
        self.cairo_conn.rectangle(x, y, width, height);
        self.cairo_conn.fill()?;
        Ok(())
    }

    fn draw_text(&self, x: f64, y: f64, text: &str, color: &str) -> Result<(), cairo::Error> {
        self.set_hex_color(color)?;
        let fe = self.cairo_conn.font_extents()?;
        let y = self.height as f64 / 2.0 + (fe.ascent() - fe.descent()) / 2.0;
        self.cairo_conn.move_to(x, y);
        self.cairo_conn.show_text(text)?;
        Ok(())
    }

    fn draw_rounded_background(
        &self,
        x: f64,
        width: f64,
        radius: f64,
        color: &str,
    ) -> Result<(), cairo::Error> {
        let height: f64 = self.height as f64 - 10.0;
        let y = (self.height as f64 - height) / 2.0;
        self.set_hex_color(color)?;
        self.cairo_conn.move_to(x + radius, y);
        self.cairo_conn.line_to(x + width - radius, y);
        // 270 -> 360
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

fn main() -> xcb::Result<()> {
    let (conn, screen_num) = xcb::Connection::connect(None)?;
    let conn = Arc::new(conn);
    let setup = conn.get_setup();
    let screen = setup.roots().nth(screen_num as usize).unwrap();
    let wid = conn.generate_id();
    // 查找支持 32 位深度的视觉（用于透明度）
    let mut visual = None;
    for depth in screen.allowed_depths() {
        if depth.depth() == 32 {
            for v in depth.visuals() {
                if v.class() == x::VisualClass::TrueColor {
                    visual = Some(v);
                }
            }
        }
    }

    let visual = visual.unwrap_or_else(|| {
        panic!("未找到 32 位视觉，请确保运行合成器（如 Picom）");
    });

    let colormap = conn.generate_id();

    let cookie = conn.send_request_checked(&x::CreateColormap {
        alloc: x::ColormapAlloc::None,
        mid: colormap,
        window: screen.root(),
        visual: visual.visual_id(),
    });
    conn.check_request(cookie).expect("failed create colormap");
    let width = screen.width_in_pixels();
    let height = 40;
    let cookie = conn.send_request_checked(&x::CreateWindow {
        depth: 32,
        wid,
        parent: screen.root(),
        x: 0,
        y: (screen.height_in_pixels() - 40) as i16,
        width,
        height,
        border_width: 0,
        class: x::WindowClass::CopyFromParent,
        visual: visual.visual_id(),
        value_list: &[
            x::Cw::BackPixel(0x0),
            x::Cw::BorderPixel(0x0),
            x::Cw::EventMask(x::EventMask::EXPOSURE),
            x::Cw::Colormap(colormap),
        ],
    });
    conn.check_request(cookie).expect("failed create window");

    let ewmh_con = ewmh::Connection::connect(&conn);

    // set dock
    let window = wid;
    let req =
        ewmh::proto::SetWmWindowType::new(window, vec![ewmh_con.atoms._NET_WM_WINDOW_TYPE_DOCK]);
    ewmh_con.send_request_checked(&req);
    ewmh_con.send_request_checked(&ewmh::proto::SetWmName::new(window, "mybar"));

    let mut arr: [u32; 12] = [0; 12];
    arr[3] = 40;
    let cookie = conn.send_request_checked(&x::ChangeProperty {
        mode: x::PropMode::Replace,
        window,
        property: ewmh_con.atoms._NET_WM_STRUT_PARTIAL,
        r#type: x::ATOM_CARDINAL,
        data: &arr,
    });
    conn.check_request(cookie).expect("failed set struct");
    conn.send_request(&x::MapWindow { window: wid });
    conn.flush()?;

    let gc = conn.generate_id();

    let cookie = conn.send_request_checked(&x::CreateGc {
        cid: gc,
        drawable: x::Drawable::Window(window),
        value_list: &[
            x::Gc::Foreground(0xFFFF9900),
            x::Gc::Background(0x00000000),
            x::Gc::GraphicsExposures(false),
        ],
    });
    conn.check_request(cookie).expect("create gc error");

    // 创建一个新的线程来定期更新时间
    let p = Painter::new(&conn, window, *visual, width as i32, height as i32);
    // let p = Arc::new(p);

    p.set_hex_color("#475164").unwrap();

    // let p_clone = Arc::clone(&p);
    // thread::spawn(move || {
    //     loop {
    //         thread::sleep(Duration::from_secs(1));
    //         if let Err(e) = draw_date(&p_clone) {
    //             eprintln!("Error updating time: {}", e);
    //         }
    //     }
    // });
    draw_title(&p, &ewmh_con);
    p.draw_volumn();
    loop {
        match conn.wait_for_event()? {
            xcb::Event::X(x::Event::Expose(ev)) => {
                if ev.count() != 0 {
                    continue;
                }
                println!("expose event");
                draw_date(&p).unwrap();
                conn.flush()?;
            }
            _ => {
                println!("other event");
            }
        }
    }

    // Ok(())
}

fn get_current_window(conn: &ewmh::Connection) -> Result<xcb::x::Window, xcb::Error> {
    let reply = conn.wait_for_reply(conn.send_request(&ewmh::proto::GetActiveWindow))?;
    Ok(reply.window)
}

fn get_current_wm_title(conn: &ewmh::Connection) -> Result<String, xcb::Error> {
    let window = get_current_window(conn)?;
    let reply = conn.wait_for_reply(conn.send_request(&ewmh::proto::GetWmName(window)))?;
    Ok(reply.name)
}

fn get_bspwm_socket() -> Option<String> {
    let socket = std::env::var("BSPWM_SOCKET");
    if let Ok(socket) = socket {
        return Some(socket);
    }
    xcb::parse_display("").map(|dis_info| {
        let host = dis_info.host;
        let dis = dis_info.display;
        let sc = dis_info.screen;
        format!("/tmp/bspwm{host}_{dis}_{sc}-socket")
    })
}

fn draw_date(p: &Painter) -> Result<(), cairo::Error> {
    let now = chrono::Local::now();
    let text = now.format("%Y/%m/%d %H:%M").to_string();
    let tw = p.text_width(&text)?;
    let gap = 30.0;
    let w = tw + gap * 2.0;
    p.draw_rounded_background(100.0, w, 10.0, "#475164")?;

    p.draw_text(100.0 + gap, 10.0, &text, "#ff3329")?;
    Ok(())
}

fn draw_title(p: &Painter, conn: &ewmh::Connection) {
    let title = get_current_wm_title(conn).unwrap();
    let tw = p.text_width(&title).unwrap();
    let gap = 30.0;
    let w = tw + 2.0 * gap;
    p.draw_rounded_background(900.0, w, 10.0, "#475164")
        .unwrap();
    p.draw_text(900.0 + gap, 20.0, &title, "#3cffdd").unwrap();
}
