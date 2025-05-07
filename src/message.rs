use crate::error;

pub enum Message {
    Date,
    BspwmUpdate,
}

impl From<xcb::x::ClientMessageData> for Message {
    fn from(value: xcb::x::ClientMessageData) -> Self {
        match value {
            xcb::x::ClientMessageData::Data32(data) => {
                let v = data[0];
                match v {
                    1 => Message::BspwmUpdate,
                    _ => Message::Date,
                }
            }
            _ => Message::Date,
        }
    }
}

impl From<Message> for xcb::x::ClientMessageData {
    fn from(value: Message) -> Self {
        let m = value as u32;
        let mut data = [0; 5];
        data[0] = m;
        Self::Data32(data)
    }
}

impl Message {
    pub fn send(self, conn: &xcb::Connection, window: xcb::x::Window) -> error::MyResult<()> {
        let d: xcb::x::ClientMessageData = self.into();
        let e = xcb::x::SendEvent {
            propagate: false,
            destination: xcb::x::SendEventDest::Window(window),
            event_mask: xcb::x::EventMask::NO_EVENT,
            event: &xcb::x::ClientMessageEvent::new(window, xcb::x::ATOM_STRING, d),
        };
        let c = conn.send_request_checked(&e);
        conn.check_request(c)?;
        Ok(())
    }
}
