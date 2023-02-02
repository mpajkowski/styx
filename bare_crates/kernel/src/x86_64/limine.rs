use limine_mini::terminal::Response as TerminalResponse;

pub struct Limine {
    pub terminal: &'static TerminalResponse,
}

impl Limine {
    pub fn gather() -> Self {
        Self {
            terminal: req::TERMINAL.response().unwrap(),
        }
    }
}

mod req {
    pub static TERMINAL: limine_mini::terminal::Request = limine_mini::terminal::Request::new(0);
}
