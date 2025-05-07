use message::Message;
use std::{sync::Arc, thread::sleep, time::Duration};
use xcb::x;

mod alsa;
mod bspwm;
mod components;
mod error;
mod light;
mod message;
mod util;
mod x11;

use components::{BspwmComponent, Component, Date, Event, Light, Painter, Title, Volume, title};
use x11::{create_window, setup_ewmh};

fn main() -> error::MyResult<()> {
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
    let painter = Painter::new(&conn, window, visual_type, width as i32, height)?;
    let audio = alsa::Audio::default();
    let volume = Volume::new(&painter, &audio);
    let light = Light::new(&painter);
    let date = Date::new(&painter);
    let title = Title::new(&painter, &ewmh_conn);
    let bspwm: Arc<std::sync::Mutex<bspwm::Bspwm>> = bspwm::Bspwm::new(&conn, window);
    let bspwm_component = BspwmComponent::new(&painter, bspwm);

    let components: Vec<Box<dyn Component>> = vec![
        Box::new(light),
        Box::new(volume),
        Box::new(date),
        Box::new(title),
        Box::new(bspwm_component),
    ];
    let conn_clone = Arc::clone(&conn);
    std::thread::spawn(move || {
        loop {
            message::Message::Date.send(&conn_clone, window).unwrap();
            sleep(Duration::from_micros(100));
        }
    });

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
                        if let Err(e) = component.handle_event(&Event::MouseClick { x, y, button })
                        {
                            eprintln!("Error handling click event: {}", e);
                        }
                        break;
                    }
                }
            }
            xcb::Event::X(xcb::x::Event::PropertyNotify(e)) => {
                println!(
                    "titlte change, w: {:?}, w2: {:?}",
                    e.window(),
                    title::get_current_window(&ewmh_conn)?
                );
                if e.atom() == ewmh_conn.atoms._NET_ACTIVE_WINDOW
                    || e.atom() == ewmh_conn.atoms._NET_WM_NAME
                {
                    components[3].draw()?;
                }
                if e.atom() == ewmh_conn.atoms._NET_ACTIVE_WINDOW
                    && e.window() == title::get_current_window(&ewmh_conn)?
                {
                    println!("window: {:?}", e.window());
                    //
                }
            }
            xcb::Event::X(x::Event::KeyPress(ev)) => {
                let event = Event::KeyPress {
                    keycode: ev.detail(),
                };
                for component in &components {
                    if let Err(e) = component.handle_event(&event) {
                        eprintln!("Error handling key event: {}", e);
                    }
                }
            }
            xcb::Event::X(x::Event::ClientMessage(ev)) => {
                let m: message::Message = ev.data().into();
                match m {
                    message::Message::Date => {
                        components[2].draw()?;
                    }
                    message::Message::BspwmUpdate => {
                        components[4].draw()?;
                    }
                }
            }
            _ => {
                println!("other event");
            }
        }
    }
}
