use std::cell::RefCell;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub enum Payload {
    Keypress(u8, u8),
    Random(u32, u32),
    Bool(bool),
    I32(i32),
    F64(f64),
}

pub trait ToPayload {
    fn to_payload(self) -> Payload;
}

impl ToPayload for Payload {
    fn to_payload(self) -> Payload {
        self
    }
}

impl ToPayload for bool {
    fn to_payload(self) -> Payload {
        Payload::Bool(self)
    }
}

impl ToPayload for i32 {
    fn to_payload(self) -> Payload {
        Payload::I32(self)
    }
}

impl ToPayload for f64 {
    fn to_payload(self) -> Payload {
        Payload::F64(self)
    }
}

#[derive(Debug)]
struct Entry {
    trace_lineno: usize,
    pos: String,
    context: String,
    payload: Payload,
}

pub struct Trace {
    entries: Rc<RefCell<Option<VecDeque<Entry>>>>,
}

impl Trace {
    pub fn load(&self, filename: &str) {
        let f = BufReader::new(File::open(filename).unwrap());
        let entries = f
            .lines()
            .enumerate()
            .map(|(lineno, line)| {
                let line = line.unwrap();
                let mut parts = line.split_whitespace().collect::<Vec<_>>();
                let pos = parts.remove(0).to_string();
                let context = parts.remove(0).to_string();
                let payload = if context == "Keypress" {
                    assert_eq!(parts.len(), 2);
                    let ch1 = parts[0].parse::<u8>().unwrap();
                    let ch2 = parts[1].parse::<u8>().unwrap();
                    Payload::Keypress(ch1, ch2)
                } else if context == "Random" {
                    assert_eq!(parts.len(), 2);
                    let range = parts[0].parse::<u32>().unwrap();
                    let value = parts[1].parse::<u32>().unwrap();
                    Payload::Random(range, value)
                } else {
                    assert_eq!(parts.len(), 2);
                    match parts[0] {
                        "bool" => Payload::Bool(parts[1] == "true"),
                        "i32" => Payload::I32(parts[1].parse().unwrap()),
                        "f64" => Payload::F64(parts[1].parse().unwrap()),
                        _ => panic!("Unexpected line {} in KeyLog.txt: {}", lineno + 1, line),
                    }
                };
                Entry {
                    trace_lineno: lineno + 1,
                    pos,
                    context,
                    payload,
                }
            })
            .collect();

        self.entries.replace(Some(entries));
    }

    fn loaded(&self) -> bool {
        self.entries.borrow().is_some()
    }

    fn pop(&self) -> Entry {
        let mut entries = self.entries.borrow_mut();
        entries.as_mut().unwrap().pop_front().unwrap()
    }

    pub fn key_pressed(&self) -> Option<bool> {
        if self.loaded() {
            let entry = self.pop();
            if entry.context != "KeyPressed" {
                panic!("Expected KeyPressed, got {:?}", entry);
            }
            let Payload::Bool(state) = entry.payload else { panic!("foo") };
            Some(state)
        } else {
            None
        }
    }

    pub fn wait_for_key_press(&self) -> Option<(u8, u8)> {
        if self.loaded() {
            match self.pop() {
                Entry {
                    payload: Payload::Keypress(ch1, ch2),
                    ..
                } => Some((ch1, ch2)),
                entry => panic!("Expected WaitForKeyPress, got {:?}", entry),
            }
        } else {
            None
        }
    }

    pub fn expect(&self, expected_context: &str, expected_payload: impl ToPayload) {
        let expected_payload = expected_payload.to_payload();
        if self.loaded() {
            let actual = self.pop();
            if actual.context != expected_context || actual.payload != expected_payload {
                panic!(
                    "Computed {} {:?} != traced {} {:?} from {} trace:{}",
                    expected_context,
                    expected_payload,
                    actual.context,
                    actual.payload,
                    actual.pos,
                    actual.trace_lineno
                );
            }
        }
    }

    pub fn trace(&self, expected_context: &str, expected_payload: impl ToPayload) {
        let expected_payload = expected_payload.to_payload();
        if self.loaded() {
            let actual = self.pop();
            if actual.context != expected_context {
                panic!(
                    "Computed {} != traced {} from {} trace:{}",
                    expected_context, actual.context, actual.pos, actual.trace_lineno
                );
            }
            if actual.payload != expected_payload {
                panic!(
                    "Computed {} {:?} != traced {} {:?} from {} trace:{}",
                    expected_context,
                    expected_payload,
                    actual.context,
                    actual.payload,
                    actual.pos,
                    actual.trace_lineno
                );
            }
        }
    }
}

thread_local! {
    static TRACE: Rc<RefCell<Option<VecDeque<Entry >>>> = Rc::new(RefCell::new(None));
}

pub fn trace() -> Trace {
    let entries = TRACE.with(|t| t.clone());
    Trace { entries }
}
