use std::cell::Cell;

pub struct HelpModule {
    pub ch: Cell<u8>,
    pub ch2: Cell<u8>,
}

impl HelpModule {
    pub fn new() -> Self {
        Self {
            ch: Cell::new(1),
            ch2: Cell::new(1),
        }
    }

    pub fn clearchs(&self) {
        self.ch.set(1);
        self.ch2.set(1);
    }
}

pub fn txtp(mut jokuluku: i32) -> Vec<u8> {
    if jokuluku != 0 {
        let mut str1: Vec<u8> = Vec::new();
        while jokuluku > 0 {
            str1.push((jokuluku % 10) as u8 + b'0');
            jokuluku /= 10;
        }
        str1.push(b'.');
        if str1.len() < 3 {
            str1.insert(0, b'0');
        }
        if jokuluku < 0 {
            str1.insert(0, b'-');
        }
        str1
    } else {
        vec![b'0', b'.', b'0']
    }
}

pub fn txt(mut jokuluku: i32) -> Vec<u8> {
    jokuluku.to_string().into_bytes()
}
