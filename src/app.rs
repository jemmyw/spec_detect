pub struct App {
    pub should_quit: bool,
}

impl App {
    pub fn new() -> App {
        App { should_quit: false }
    }

    pub fn on_quit(&mut self) {
        self.should_quit = true;
    }
}
