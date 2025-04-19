use std::sync::Arc;
use std::thread;
use std::time::Duration;
use xcb::x;
use xcb_wm::ewmh;

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

    let cookie = conn.send_request_checked(&x::CreateWindow {
        depth: 32,
        wid,
        parent: screen.root(),
        x: 0,
        y: (screen.height_in_pixels() - 30) as i16,
        width: screen.width_in_pixels(),
        height: 30,
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

fn draw_date(conn: &xcb::Connection, window: x::Window, gc: x::Gcontext) -> xcb::Result<()> {
    let now = chrono::Local::now();
    let text = now.format("%Y/%m/%d %H:%M");
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
