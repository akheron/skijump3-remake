use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};

pub struct LangModule {
    pub lnames: Vec<String>,
    pub plstr: HashMap<u32, Vec<u8>>,
}

impl LangModule {
    pub fn new() -> Self {
        LangModule {
            lnames: Vec::new(),
            plstr: HashMap::new(),
        }
    }

    pub fn init(&mut self) {
        self.reset_language();
    }

    pub fn load_language(&mut self, languageindex: u8) {
        let f = File::open("LANGBASE.SKI").unwrap();
        let mut lines = BufReader::new(f).split(b'\n');
        loop {
            let Some(Ok(line)) = lines.next() else {
                panic!("Language with index {} not found", languageindex)
            };
            if line.starts_with(b"*") && line[1] - b'A' == languageindex {
                break;
            };
        }

        self.plstr.clear();
        loop {
            let Some(Ok(line)) = lines.next() else {
                break
            };
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
            self.plstr.insert(index, parts[1].to_vec());
        }
    }

    // Seems to be unused.
    //
    // pub fn lnstr(&self, index: i32) -> &str {
    //     unimplemented!()
    // }

    pub fn lstr(&self, index: u32) -> &[u8] {
        return self.plstr.get(&index).unwrap();
    }

    pub fn lch(&self, index: u32, num: u32) -> char {
        self.lstr(index)[num as usize] as char
    }

    pub fn lrstr(&self, start: u32, end: u32) -> &[u8] {
        let index = start + (rand::random::<f64>() * ((end - start + 1) as f64)) as u32;
        self.lstr(index)
    }

    fn reset_language(&mut self) {
        let f = BufReader::new(File::open("LANGBASE.SKI").unwrap());
        let mut lines = f.split(b'\n');
        loop {
            let Some(Ok(mut line)) = lines.next() else { return };
            if line.starts_with(b"*") {
                line = lines.next().unwrap().unwrap();
                self.lnames.push(String::from_utf8(line).unwrap());
            }
        }
    }
}
