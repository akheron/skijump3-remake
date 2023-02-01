use std::cell::Cell;

pub struct HelpModule {
    pub ch1: Cell<u8>,
    pub ch2: Cell<u8>,
}

impl HelpModule {
    pub fn new() -> Self {
        Self {
            ch1: Cell::new(1),
            ch2: Cell::new(1),
        }
    }

    pub fn clearchs(&self) {
        self.ch1.set(1);
        self.ch2.set(1);
    }
}
