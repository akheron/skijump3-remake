use crate::platform::Platform;
use crate::rs_util::random;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::rc::Rc;

pub struct LangModule<'s, P: Platform> {
    s: &'s P,
    pub lnames: Vec<Vec<u8>>,
    pub plstr: RefCell<HashMap<u32, Rc<Vec<u8>>>>,
}

impl<'s, P: Platform> LangModule<'s, P> {
    pub fn new(s: &'s P) -> Self {
        LangModule {
            s,
            lnames: Vec::new(),
            plstr: RefCell::new(HashMap::new()),
        }
    }

    pub fn num_languages(&self) -> usize {
        self.lnames.len()
    }

    pub fn init(&mut self) {
        self.reset_language();
    }

    pub fn load_language(&self, languageindex: u8) {
        let f = self.s.open_file("LANGBASE.SKI");
        let mut lines = BufReader::new(f).split(b'\n');
        loop {
            let Some(Ok(line)) = lines.next() else {
                panic!("Language with index {} not found", languageindex)
            };
            // @ = 64, A = 65 ==> languageindex 1 = A
            if line.starts_with(b"*") && line[1] - 64 == languageindex {
                break;
            };
        }

        let mut plstr = self.plstr.borrow_mut();
        plstr.clear();
        loop {
            let Some(Ok(line)) = lines.next() else { break };
            if line.starts_with(b"*") {
                break;
            }

            let parts = line.splitn(2, |c| *c == b':').collect::<Vec<_>>();
            if parts.len() != 2 || !parts[0].iter().all(|c| b'0' <= *c && *c <= b'9') {
                continue;
            }
            let index = parts[0]
                .iter()
                .fold(0, |acc, c| acc * 10 + (c - b'0') as u32);
            plstr.insert(index, Rc::new(parts[1].to_vec()));
        }
    }

    pub fn lstr(&self, index: u32) -> Rc<Vec<u8>> {
        return self.plstr.borrow().get(&index).unwrap().clone();
    }

    pub fn lch(&self, index: u32, num: u32) -> u8 {
        // In Pascal, strings are 1-indexed => subtract 1.
        assert!(num >= 1);
        self.lstr(index)[(num - 1) as usize]
    }

    pub fn lrstr(&self, start: u32, stop: u32) -> Rc<Vec<u8>> {
        self.lstr(start + random(stop - start + 1))
    }

    fn reset_language(&mut self) {
        let f = BufReader::new(self.s.open_file("LANGBASE.SKI"));
        let mut lines = f.split(b'\n');
        loop {
            let Some(Ok(mut line)) = lines.next() else {
                return;
            };
            if line.starts_with(b"*") {
                line = lines.next().unwrap().unwrap();
                self.lnames.push(line);
            }
        }
    }
}
