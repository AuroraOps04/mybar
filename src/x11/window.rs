use xcb::x;

pub fn create_window(conn: &xcb::Connection, screen: &x::Screen) -> (x::Window, x::Visualtype) {
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
            x::Cw::EventMask(
                x::EventMask::EXPOSURE | x::EventMask::BUTTON_PRESS | x::EventMask::KEY_PRESS,
            ),
            x::Cw::Colormap(colormap),
        ],
    });
    conn.check_request(cookie).expect("failed create window");
    // set stack mode
    let cookie = conn.send_request_checked(&xcb::x::ConfigureWindow {
        window: wid,
        value_list: &[xcb::x::ConfigWindow::StackMode(xcb::x::StackMode::Below)],
    });
    conn.check_request(cookie)
        .expect("failed set window stack mode");

    (wid, *visual)
}

