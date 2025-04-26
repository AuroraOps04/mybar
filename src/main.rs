use std::sync::Arc;
use xcb::{x};

mod error;
mod alsa;
mod util;
mod components;
mod x11;

use components::{Painter, Volume, Date, Title, Component, Event};
use x11::{create_window, setup_ewmh};

fn main() -> xcb::Result<()> {
    let (conn, screen_num) = xcb::Connection::connect(None)?;
    let conn = Arc::new(conn);
    let setup = conn.get_setup();
    let screen = setup.roots().nth(screen_num as usize).unwrap();
    
    let (window, visual_type) = create_window(&conn, screen);
    let ewmh_conn = setup_ewmh(&conn, window);
    
    conn.send_request(&x::MapWindow { window });
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

    let width = screen.width_in_pixels();
    let height = 40;
    let painter = Painter::new(&conn, window, visual_type, width as i32, height as i32).unwrap();
    let  audio = alsa::Audio::default();
    let volume = Volume::new(&painter, & audio);
    let date = Date::new(&painter);
    let title = Title::new(&painter, &ewmh_conn);
    
    let components: Vec<Box<dyn Component>> = vec![
        Box::new(volume),
        Box::new(date),
        Box::new(title),
    ];
    
    loop {
        match conn.wait_for_event()? {
            xcb::Event::X(x::Event::Expose(ev)) => {
                if ev.count() != 0 {
                    continue;
                }
                println!("expose event");
                for component in &components {
                    if let Err(e) = component.draw() {
                        eprintln!("Error drawing component: {}", e);
                    }
                }
                conn.flush()?;
            }
            xcb::Event::X(x::Event::ButtonPress(ev)) => {
                let x = ev.event_x();
                let y = ev.event_y();
                let button = ev.detail();
                for component in &components {
                    if component.contains_point(x, y) {
                        if let Err(e) = component.handle_event(&Event::MouseClick { x, y, button }) {
                            eprintln!("Error handling click event: {}", e);
                        }
                        break;
                    }
                }
            }
            xcb::Event::X(x::Event::KeyPress(ev)) => {
                let event = Event::KeyPress { keycode: ev.detail() };
                for component in &components {
                    if let Err(e) = component.handle_event(&event) {
                        eprintln!("Error handling key event: {}", e);
                    }
                }
            }
            _ => {
                println!("other event");
            }
        }
    }
}
