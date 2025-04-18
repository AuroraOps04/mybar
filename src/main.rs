use xcb::x;
use xcb_wm::ewmh;

fn main() -> xcb::Result<()> {
    let (conn, screen_num) = xcb::Connection::connect(None)?;
    let setup = conn.get_setup();
    let screen = setup.roots().nth(screen_num as usize).unwrap();
    let wid = conn.generate_id();

    conn.send_request(&x::CreateWindow {
        depth: screen.root_depth(),
        wid,
        parent: screen.root(),
        x: 0,
        y: (screen.height_in_pixels() - 30) as i16,
        width: screen.width_in_pixels(),
        height: 30,
        border_width: 0,
        class: x::WindowClass::CopyFromParent,
        visual: screen.root_visual(),
        value_list: &[
            x::Cw::BackPixel(screen.white_pixel()),
            x::Cw::EventMask(x::EventMask::EXPOSURE),
        ],
    });

    let ewmh_con = ewmh::Connection::connect(&conn);

    // set dock
    let window = wid;
    let req =
        ewmh::proto::SetWmWindowType::new(window, vec![ewmh_con.atoms._NET_WM_WINDOW_TYPE_DOCK]);
    ewmh_con.send_request_checked(&req);

    ewmh_con.send_request_checked(&ewmh::proto::SetWmName::new(window, "mybar"));

    let mut arr: [u32; 4] = [0; 4];
    arr[3] = 30;
    let cookie = conn.send_request_checked(&x::ChangeProperty {
        mode: x::PropMode::Replace,
        window,
        property: ewmh_con.atoms._NET_WM_STRUT,
        r#type: x::ATOM_CARDINAL,
        data: &arr,
    });
    conn.check_request(cookie).expect("failed set struct");
    conn.send_request(&x::MapWindow { window: wid });
    conn.flush()?;

    loop {
        match conn.wait_for_event()? {
            xcb::Event::X(x::Event::Expose(ev)) => {
                if (ev.count() != 0) {
                    continue;
                }
            }
            _ => {
                println!("other event");
            }
        }
    }

    // Ok(())
}
