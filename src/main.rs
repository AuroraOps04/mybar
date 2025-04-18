use xcb::Xid;
use xcb::x;

fn setupwindow(window: x::Window) -> xcb::Result<()> {
    let (conn, screen_num) = xcb::Connection::connect(None)?;
    let setup = conn.get_setup();
    let screen = setup.roots().nth(screen_num as usize).unwrap();
}

fn main() {}
