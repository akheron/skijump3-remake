use std::cell::RefCell;

pub struct HelpModule {
    pub ch1: RefCell<u8>,
    pub ch2: RefCell<u8>,
}

impl HelpModule {
    pub fn new() -> Self {
        Self {
            ch1: RefCell::new(1),
            ch2: RefCell::new(1),
        }
    }

    pub fn clearchs(&self) {
        *self.ch1.borrow_mut() = 1;
        *self.ch2.borrow_mut() = 1;
    }
}
