use std::fmt;

#[derive(Debug)]
pub enum MyBarError {
    Cairo(cairo::Error),
    Xcb(xcb::Error),
    Other(String),
}

impl fmt::Display for MyBarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MyBarError::Cairo(e) => write!(f, "Cairo error: {}", e),
            MyBarError::Xcb(e) => write!(f, "XCB error: {}", e),
            MyBarError::Other(s) => write!(f, "Other error: {}", s),
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