use crate::sdlport::TPalette;
use image::{Rgb, RgbImage};
use sdl2::pixels::Color;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
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
    entries: RefCell<Option<VecDeque<Entry>>>,
    frame_tracing_enabled: Cell<bool>,
    traced_frame_num: Cell<i32>,
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

    pub fn start_frame_tracing(&self) {
        self.frame_tracing_enabled.set(true);
    }

    pub fn expect_frame(&self, buffer: &[u8], palette: &[u8; 768]) {
        if !self.frame_tracing_enabled.get() {
            return;
        }
        let framecount = self.traced_frame_num.get() + 1;
        self.traced_frame_num.set(framecount);

        let filename = format!("frames/{:0>6}.dat", framecount);
        let mut f = File::open(filename).unwrap();

        let mut expected_buffer = vec![0; buffer.len()];
        f.read_exact(&mut expected_buffer).unwrap();

        let mut expected_palette = [0; 768];
        f.read_exact(&mut expected_palette).unwrap();

        let buffer_eq = buffer == &expected_buffer[..];
        let palette_eq = palette == &expected_palette[..];
        if !buffer_eq || !palette_eq {
            let expected_filename = "expected.png";
            let actual_filename = "actual.png";
            write_image(expected_filename, &expected_buffer, &expected_palette);
            write_image(actual_filename, buffer, palette);
            eprintln!("Frame {}:", framecount);
            eprintln!("Buffer {} equal", if buffer_eq { "IS" } else { "IS NOT" });
            if !buffer_eq {
                eprintln!("Buffer diff [(x, y): expected != actual]");
                for y in 0..200 {
                    for x in 0..320 {
                        let expected = expected_buffer[y * 320 + x];
                        let actual = buffer[y * 320 + x];
                        if expected != actual {
                            eprintln!("  ({}, {}): {} != {}", x, y, expected, actual);
                        }
                    }
                }
            }
            eprintln!("Palette {} equal", if palette_eq { "IS" } else { "IS NOT" });
            if !palette_eq {
                eprintln!("Palette diff [index: expected != actual]:");
                for i in 0..256 {
                    let expected = &expected_palette[i * 3..i * 3 + 3];
                    let actual = &palette[i * 3..i * 3 + 3];
                    if expected != actual {
                        eprintln!("  {:3}: {:?} != {:?}", i, expected, actual);
                    }
                }
            }
            eprintln!("Expected screen: {expected_filename}");
            eprintln!("Actual screen: {actual_filename}");
            panic!("Frame equality check failed");
        }
    }
}

fn write_image(filename: &str, buffer: &[u8], palette: &[u8; 768]) {
    let mut img = RgbImage::new(320, 200);
    for y in 0..200 {
        for x in 0..320 {
            let i = buffer[y * 320 + x] as usize;
            let r = palette[i * 3] * 4;
            let g = palette[i * 3 + 1] * 4;
            let b = palette[i * 3 + 2] * 4;
            img.put_pixel(x as u32, y as u32, Rgb([r, g, b]));
        }
    }
    img.save(filename).unwrap();
}

thread_local! {
    static TRACE: Rc<Trace> = Rc::new(Trace {
        entries: RefCell::new(None),
        frame_tracing_enabled: Cell::new(false),
        traced_frame_num: Cell::new(-1),
    });
}

pub fn trace() -> Rc<Trace> {
    TRACE.with(|t| t.clone())
}
