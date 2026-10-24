use crate::platform::Platform;
use crate::rs_util::{random, read_line, val};
use std::cell::RefCell;
use std::io::{BufRead, BufReader};
use std::rc::Rc;

const NUM_STR: usize = 599;

/// The length of `LangStr`
const LANG_STR_LEN: usize = 59;

pub struct LangModule<'s, P: Platform> {
    s: &'s P,
    pub lnames: Vec<Vec<u8>>,
    pub plstr: RefCell<Vec<Rc<Vec<u8>>>>,
}

impl<'s, P: Platform> LangModule<'s, P> {
    pub fn new(s: &'s P) -> Self {
        LangModule {
            s,
            lnames: Vec::new(),
            plstr: RefCell::new(vec![Rc::new(Vec::new()); NUM_STR + 1]),
        }
    }

    pub fn num_languages(&self) -> usize {
        self.lnames.len()
    }

    pub fn lstr(&self, index: u32) -> Rc<Vec<u8>> {
        match self.plstr.borrow().get(index as usize) {
            Some(str1) => str1.clone(),
            None => Rc::new(Vec::new()),
        }
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

    pub fn load_language(&self, languageindex: u8) {
        let mut plstr = self.plstr.borrow_mut();
        for temp in 0..=NUM_STR {
            plstr[temp] = Rc::new(b"?".to_vec());
        }

        let mut f1 = BufReader::new(self.s.open_file("LANGBASE.SKI"));
        let mut lines = std::iter::from_fn(|| match f1.fill_buf() {
            Ok(buf) if !buf.is_empty() => Some(read_line(&mut f1).unwrap()),
            _ => None,
        });

        let mut index = 0;

        for str1 in lines.by_ref() {
            if str1.first() == Some(&b'*') && str1.get(1) == Some(&languageindex.wrapping_add(64)) {
                //{ oikea kieli löytyi! }
                index = 1;
                break;
            }
        }

        if index > 0 {
            //{ kunhan oikea kieli on löytyynty. }
            for str1 in lines {
                if str1.first() == Some(&b'*') {
                    break; //{ kiitti, riittää }
                }

                index = 0;
                let mut first = true;

                for temp in 0..str1.len() {
                    if index > 0 && plstr[index].len() < LANG_STR_LEN {
                        Rc::make_mut(&mut plstr[index]).push(str1[temp]);
                    }

                    if str1[temp] == b':' && first {
                        first = false;
                        let code = val(&str1[..temp]);
                        index = if (1..=NUM_STR as i32).contains(&code) {
                            code as usize
                        } else {
                            0
                        };
                        if index > 0 {
                            plstr[index] = Rc::new(Vec::new()); //{ putsaa stringin }
                        }
                    }
                }
            }
        }
    }

    pub fn init(&mut self) {
        self.reset_language();
    }
}
