use limine_mini::terminal;

pub struct Limine {
    pub terminal: &'static limine_mini::terminal::Response,
}

impl Limine {
    pub fn gather() -> Self {
        Self {
            terminal: req::TERMINAL.response().unwrap(),
        }
    }
}

mod req {
    use super::*;

    pub static TERMINAL: terminal::Request = terminal::Request::new(0);
}
