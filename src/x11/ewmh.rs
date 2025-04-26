use xcb::{x, Xid};
use xcb_wm::ewmh;

pub fn setup_ewmh(conn: &xcb::Connection, window: x::Window) -> ewmh::Connection {
    let ewmh_con = ewmh::Connection::connect(&conn);

    // set dock
    let req = ewmh::proto::SetWmWindowType::new(
        window,
        vec![ewmh_con.atoms._NET_WM_WINDOW_TYPE_DOCK]
    );
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

    ewmh_con
} 