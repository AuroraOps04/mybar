use std::{fmt, ptr::write};

#[derive(Debug)]
pub enum MyBarError {
    Cairo(cairo::Error),
    Xcb(xcb::Error),
    XcbConn(xcb::ConnError),
    XcbProto(xcb::ProtocolError),
    Other(String),
}

pub type MyResult<T> = std::result::Result<T, MyBarError>;

impl fmt::Display for MyBarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MyBarError::Cairo(e) => write!(f, "Cairo error: {}", e),
            MyBarError::Xcb(e) => write!(f, "XCB error: {}", e),
            MyBarError::Other(s) => write!(f, "Other error: {}", s),
            MyBarError::XcbConn(e) => write!(f, "Xcb connect error: {}", e),
            MyBarError::XcbProto(e) => write!(f, "Xcb protocol error: {}", e),
        }
    }
}

impl std::error::Error for MyBarError {}

impl From<cairo::Error> for MyBarError {
    fn from(err: cairo::Error) -> Self {
        MyBarError::Cairo(err)
    }
}

impl From<xcb::Error> for MyBarError {
    fn from(err: xcb::Error) -> Self {
        MyBarError::Xcb(err)
    }
}

impl From<xcb::ConnError> for MyBarError {
    fn from(value: xcb::ConnError) -> Self {
        MyBarError::XcbConn(value)
    }
}

impl From<xcb::ProtocolError> for MyBarError {
    fn from(value: xcb::ProtocolError) -> Self {
        MyBarError::XcbProto(value)
    }
}
