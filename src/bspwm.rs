use std::{
    env,
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    sync::{Arc, Mutex},
};

use crate::message;

#[derive(Debug, Clone)]
pub enum DesktopEnum {
    FREE,
    FOCUSED,
    OCCUPIED,
    URGENT,
}

impl DesktopEnum {
    fn from_char(c: char) -> Option<Self> {
        match c {
            'o' => Some(DesktopEnum::OCCUPIED),
            'u' => Some(DesktopEnum::URGENT),
            'F' | 'U' | 'O' => Some(DesktopEnum::FOCUSED),
            _ => Some(DesktopEnum::FREE),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Desktop {
    pub state: DesktopEnum,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Monitor {
    pub name: String,
    pub is_active: bool,
    pub desktops: Vec<Desktop>,
}

#[derive(Debug, Clone)]
pub struct Bspwm {
    pub monitors: Vec<Monitor>,
}

impl Bspwm {
    pub fn new(conn: &Arc<xcb::Connection>, window: xcb::x::Window) -> Arc<Mutex<Bspwm>> {
        let bspwm = Arc::new(Mutex::new(Bspwm { monitors: vec![] }));
        let b = Arc::clone(&bspwm);
        let conn_c = Arc::clone(conn);
        std::thread::spawn(move || {
            let sock = get_bspwm_socket();
            let mut sock = UnixStream::connect(sock).unwrap();
            sock.write_all(b"subscribe\0report\0").unwrap();
            let mut reader = BufReader::new(sock);
            let mut line = String::new();
            loop {
                if let Err(e) = reader.read_line(&mut line) {
                    eprintln!("read bspwm err: {e}");
                    break;
                }
                if let Ok(mut bspwm) = b.lock() {
                    bspwm.parse_report(&line);
                    message::Message::BspwmUpdate
                        .send(conn_c.as_ref(), window)
                        .unwrap();
                }
                line.clear();
            }
        });

        bspwm
    }

    fn parse_report(&mut self, report: &str) {
        let mut cur_monitor: Option<&mut Monitor> = None;
        let mut i = 0;
        let chars: Vec<char> = report.chars().collect();
        let len = chars.len();

        while i < len {
            match chars[i] {
                'M' | 'm' => {
                    let is_active = chars[i] == 'M';
                    i += 1;
                    let start = i;
                    while i < len && chars[i] != ':' {
                        i += 1;
                    }
                    let name: String = chars[start..i].iter().collect();

                    // 查找或创建显示器
                    cur_monitor = self.monitors.iter_mut().find(|m| m.name == name);
                    if cur_monitor.is_none() {
                        self.monitors.push(Monitor {
                            name: name.clone(),
                            is_active,
                            desktops: vec![],
                        });
                        cur_monitor = self.monitors.last_mut();
                    } else {
                        if let Some(mon) = cur_monitor.as_mut() {
                            mon.is_active = is_active;
                        }
                    }
                }
                'o' | 'O' | 'f' | 'F' | 'u' | 'U' => {
                    if let Some(monitor) = cur_monitor.as_mut() {
                        let state = DesktopEnum::from_char(chars[i]).unwrap();
                        i += 1;
                        let start = i;
                        while i < len && chars[i] != ':' {
                            i += 1;
                        }
                        let name: String = chars[start..i].iter().collect();

                        // 查找或创建桌面
                        if let Some(desktop) = monitor.desktops.iter_mut().find(|d| d.name == name)
                        {
                            desktop.state = state;
                        } else {
                            monitor.desktops.push(Desktop { state, name });
                        }
                    }
                }
                'L' | 'T' | 'G' => {
                    // 跳过这些标记
                    i += 1;
                    while i < len && chars[i] != ':' && chars[i] != '\n' {
                        i += 1;
                    }
                }
                _ => i += 1,
            }
        }
    }
}

fn get_bspwm_socket() -> String {
    match env::var("BSPWM_SOCKET") {
        Ok(sock) => sock,
        Err(_) => {
            let di = xcb::parse_display("").unwrap();
            format!("/tmp/bspwm{}_{}_{}-socket", di.host, di.display, di.screen).to_string()
        }
    }
}
#[cfg(test)]
mod test {
    use std::{
        io::{BufRead, BufReader, Write},
        os::unix::net::UnixStream,
    };

    use crate::bspwm::get_bspwm_socket;

    #[test]
    fn t1() {
        let sock = get_bspwm_socket();

        let mut sock = UnixStream::connect(sock).expect("conn");
        sock.write_all(b"subscribe\0report\0").expect("send");
        let mut reader = BufReader::new(sock);
        // let mut buffer = Vec::new();
        let mut s = String::new();
        reader.read_line(&mut s).unwrap();
    }
}
