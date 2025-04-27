use std::env;

pub struct Desktop {}

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
        io::{Read, Write},
        os::unix::net::UnixStream,
    };

    use crate::bspwm::get_bspwm_socket;

    #[test]
    fn t1() {
        let sock = get_bspwm_socket();
        let b = b"subscribe\0report";

        assert_eq!(0, "".as_bytes().len());
        assert_eq!(sock, "/tmp/bspwm_0_0-socket".to_string());
        println!("sock: {sock}");
        let mut sock = UnixStream::connect(sock).expect("conn");
        println!("conn ok");
        sock.write_all(b"subscribe\0report\0").expect("send");
        println!("write ok");
        //
        let mut s = String::new();
        sock.read_to_string(&mut s).expect("read");
        // println!("{s}")
        // let s = std::thread::spawn(move || {
        //     loop {
        //     }
        // });
        // s.join().unwrap();
    }
}
