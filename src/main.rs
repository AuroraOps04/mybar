use std::sync::Arc;
use std::thread;
use std::time::Duration;
use xcb::{Xid, x};
use xcb_wm::ewmh;

fn t1(conn: &xcb::Connection, window: xcb::x::Window, vis: &mut xcb::x::Visualtype) {
    unsafe {
        let conn = cairo::XCBConnection::from_raw_none(
            conn.get_raw_conn() as *mut cairo_sys::xcb_connection_t
        );
        let surface = cairo::XCBSurface::create(
            &conn,
            &cairo::XCBDrawable(window.resource_id()),
            &cairo::XCBVisualType::from_raw_none(
                vis as *mut xcb::x::Visualtype as *mut cairo_sys::xcb_visualtype_t,
            ),
            30,
            30,
        )
        .expect("surface error");

        let c = cairo::Context::new(surface).unwrap();
        c.set_line_width(4.0);
        c.set_source_rgb(255.0, 0.0, 0.0);
        c.rectangle(0.0, 0.0, 10.0, 10.0);
        c.stroke().unwrap();
    }
}

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
    cairo_conn: cairo::Context,
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
        Painter {
            width,
            height,
            cairo_conn,
        }
    }

    fn draw_rectangle(&self, x: f64, y: f64, size: f64) -> Result<(), cairo::Error> {
        self.cairo_conn.set_source_rgb(255.0, 0.0, 0.0);
        self.cairo_conn.rectangle(x, y, size, self.height as f64);
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
    let height = 30;
    let cookie = conn.send_request_checked(&x::CreateWindow {
        depth: 32,
        wid,
        parent: screen.root(),
        x: 0,
        y: (screen.height_in_pixels() - 30) as i16,
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
    arr[3] = 30;
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
            x::Gc::Foreground(0x80FF9900),
            x::Gc::Background(0x00000000),
            x::Gc::GraphicsExposures(false),
        ],
    });
    conn.check_request(cookie).expect("create gc error");

    // 创建一个新的线程来定期更新时间
    let conn_clone = Arc::clone(&conn);
    let window_clone = window;
    let gc_clone = gc;
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));
            if let Err(e) = draw_date(&conn_clone, window_clone, gc_clone) {
                eprintln!("Error updating time: {}", e);
            }
        }
    });
    let title = get_current_wm_title(&ewmh_con)?;
    println!("window title {title:?}");
    let p = Painter::new(&conn, window, *visual, width as i32, height as i32);
    p.draw_rectangle(20.0, 10.0, 200.0).expect("faield draw");

    loop {
        match conn.wait_for_event()? {
            xcb::Event::X(x::Event::Expose(ev)) => {
                if ev.count() != 0 {
                    continue;
                }
                println!("expose event");
                draw_date(&conn, window, gc)?;
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

fn draw_date(conn: &xcb::Connection, window: x::Window, gc: x::Gcontext) -> xcb::Result<()> {
    let now = chrono::Local::now();
    let text = now.format("%Y/%m/%d %H:%M");
    // let font = conn.generate_id();
    // conn.send_request_checked(&x::QueryFont { font });
    let cookie = conn.send_request_checked(&x::ImageText8 {
        drawable: x::Drawable::Window(window),
        gc,
        x: 20,
        y: 15,
        string: text.to_string().as_bytes(),
    });
    conn.check_request(cookie).expect("draw text error");
    conn.flush()?;
    Ok(())
}
