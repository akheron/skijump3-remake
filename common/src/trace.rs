//! Replays a trace written by the tracing build of the Pascal SDL port and
//! checks that this port computes the same state. Keypresses, random numbers,
//! the clock and the files that `FindFirst` finds are taken from the trace;
//! records, written files, console text and frame CRCs are compared against it.
//!
//! Trace lines look like `SJ3.PAS:1234 Context payload...`. The order of the
//! lines must match the order of the calls here exactly.

#[cfg(feature = "trace")]
mod imp {
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::fs::{self, File};
    use std::io::{BufRead, BufReader, Write};
    use std::path::Path;

    struct Entry {
        trace_lineno: usize,
        line: String,
    }

    impl Entry {
        fn context(&self) -> &str {
            self.line.split(' ').nth(1).unwrap_or("")
        }
        fn payload(&self) -> Vec<&str> {
            self.line.split(' ').skip(2).collect()
        }
        fn rest(&self) -> &str {
            self.line.splitn(3, ' ').nth(2).unwrap_or("")
        }
    }

    /// The Pascal run that wrote the trace
    #[derive(Clone)]
    struct Run {
        dir: String,
        seed: String,
        keys: String,
    }

    #[derive(Default)]
    struct State {
        entries: Option<VecDeque<Entry>>,
        run: Option<Run>,
        frame_dir: Option<String>,
        rendered_frames: i32,
        consumed_lines: usize,
        checked_frames: usize,
        console_untraced: bool,
        exitcode: usize,
    }

    thread_local! {
        static STATE: RefCell<State> = RefCell::new(State::default());
    }

    pub fn load(trace_path: &str, frame_dir: Option<String>) {
        let f = BufReader::new(File::open(trace_path).unwrap());
        let mut entries: VecDeque<Entry> = f
            .lines()
            .enumerate()
            .map(|(i, line)| Entry {
                trace_lineno: i + 1,
                line: line.unwrap(),
            })
            .collect();
        let start = entries
            .pop_front()
            .filter(|entry| entry.context() == "Start")
            .unwrap_or_else(|| panic!("{trace_path} does not begin with a Start line"));
        let (seed, keys) = start
            .rest()
            .strip_prefix("seed=")
            .and_then(|rest| rest.split_once(" keys="))
            .unwrap_or_else(|| panic!("Invalid Start line: {}", start.line));
        // The run scripts write the trace to <run dir>/run/Trace.txt
        let dir = Path::new(trace_path)
            .canonicalize()
            .unwrap()
            .parent()
            .and_then(Path::parent)
            .unwrap()
            .display()
            .to_string();
        if let Some(frame_dir) = &frame_dir {
            if !Path::new(frame_dir).is_dir() {
                panic!("SJ3_FRAMES={frame_dir} is not a directory");
            }
        }
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            s.entries = Some(entries);
            s.run = Some(Run {
                dir,
                seed: seed.to_string(),
                keys: keys.to_string(),
            });
            s.frame_dir = frame_dir;
            s.consumed_lines = 1;
        });
    }

    pub fn active() -> bool {
        STATE.with(|s| s.borrow().entries.is_some())
    }

    fn pop(expected_context: &str) -> Entry {
        let entry = STATE.with(|s| s.borrow_mut().entries.as_mut().unwrap().pop_front());
        let Some(entry) = entry else {
            panic!("Expected {expected_context}, but the trace has no End line");
        };
        if entry.context() == "End" {
            end(expected_context, &entry);
        }
        if entry.context() != expected_context {
            panic!(
                "Expected {expected_context}, trace:{} has: {}",
                entry.trace_lineno, entry.line
            );
        }
        STATE.with(|s| s.borrow_mut().consumed_lines += 1);
        entry
    }

    /// The Pascal run exits when the key script runs out, when it reaches
    /// SJ3_MAXFRAMES, or at the end of the game.
    fn end(expected_context: &str, entry: &Entry) -> ! {
        if !["KeyPressed", "Keypress", "Frame", "End"].contains(&expected_context) {
            panic!(
                "Expected {expected_context}, but the Pascal run exits at trace:{}: {}",
                entry.trace_lineno, entry.line
            );
        }
        let field = |name: &str| {
            entry
                .payload()
                .iter()
                .find_map(|field| field.strip_prefix(name)?.strip_prefix('='))
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or_else(|| panic!("No {name} in trace:{}", entry.trace_lineno))
        };
        let (frames, records, exitcode) = (field("frames"), field("records"), field("exitcode"));
        let (checked_frames, consumed_lines, lines_after, expected_exitcode) = STATE.with(|s| {
            let s = s.borrow();
            let lines_after = s.entries.as_ref().unwrap().len();
            (s.checked_frames, s.consumed_lines, lines_after, s.exitcode)
        });
        if lines_after > 0 {
            panic!(
                "The trace has {lines_after} lines after the End line on line {}",
                entry.trace_lineno
            );
        }
        if exitcode != expected_exitcode || frames != checked_frames || records != consumed_lines {
            panic!(
                "Replayed {consumed_lines} records and {checked_frames} frames with exit code {expected_exitcode}, but trace:{} has: {}",
                entry.trace_lineno, entry.line
            );
        }
        println!("Trace replay complete: {records} records and {frames} frames match");
        std::process::exit(0);
    }

    /// Checks that the Pascal run also ends here, with this exit code.
    pub fn finish(exitcode: i32) {
        STATE.with(|s| s.borrow_mut().exitcode = exitcode as usize);
        pop("End");
    }

    pub fn key_pressed() -> bool {
        let entry = pop("KeyPressed");
        entry.payload() == ["bool", "true"]
    }

    pub fn keypress() -> (u8, u8) {
        let entry = pop("Keypress");
        let p = entry.payload();
        (p[0].parse().unwrap(), p[1].parse().unwrap())
    }

    pub fn random(range: u32) -> u32 {
        let entry = pop("Random");
        let p = entry.payload();
        let traced_range: u32 = p[0].parse().unwrap();
        if traced_range != range {
            panic!(
                "Random({range}) but trace:{} has: {}",
                entry.trace_lineno, entry.line
            );
        }
        p[1].parse().unwrap()
    }

    pub fn find_files(pattern: &str) -> Vec<String> {
        let entry = pop("FindFirst");
        let (traced_pattern, first) = match entry.rest().split_once(' ') {
            Some((traced_pattern, name)) => (traced_pattern, Some(name)),
            None => (entry.rest(), None),
        };
        if traced_pattern != pattern {
            panic!(
                "FindFirst {pattern}, trace:{} has: {}",
                entry.trace_lineno, entry.line
            );
        }
        let mut names = Vec::new();
        let Some(first) = first else { return names };
        names.push(first.to_string());
        loop {
            let entry = pop("FindNext");
            if entry.rest().is_empty() {
                return names;
            }
            names.push(entry.rest().to_string());
        }
    }

    pub fn file_written(name: &str, content: &[u8]) {
        let entry = pop("FileWritten");
        let written = format!("{} {:08X} {name}", content.len(), crc32(&[content]));
        if entry.rest() != written {
            panic!(
                "Wrote {written}, trace:{} has: {}",
                entry.trace_lineno, entry.line
            );
        }
    }

    pub fn file_removed(name: &str) {
        let entry = pop("FileRemoved");
        if entry.rest() != name {
            panic!(
                "Removed {name}, trace:{} has: {}",
                entry.trace_lineno, entry.line
            );
        }
    }

    /// The Pascal run traces the end text only if SJ3_TRACE_ENDTEXT is set.
    pub fn start_end_text() {
        let traced = std::env::var_os("SJ3_TRACE_ENDTEXT").is_some_and(|value| !value.is_empty());
        STATE.with(|s| s.borrow_mut().console_untraced = !traced);
    }

    pub fn console(text: &[u8]) {
        if STATE.with(|s| s.borrow().console_untraced) {
            return;
        }
        let entry = pop("Console");
        let escaped: String = text
            .iter()
            .map(|&c| match c {
                b'\\' => "\\\\".to_string(),
                b' '..=b'~' => (c as char).to_string(),
                _ => format!("\\x{c:02X}"),
            })
            .collect();
        if entry.rest() != escaped {
            panic!(
                "Wrote {escaped} to the console, trace:{} has: {}",
                entry.trace_lineno, entry.line
            );
        }
    }

    pub fn date() -> (u16, u16, u16, u16) {
        clock(pop("Date"))
    }

    pub fn time() -> (u16, u16, u16, u16) {
        clock(pop("Time"))
    }

    fn clock(entry: Entry) -> (u16, u16, u16, u16) {
        let p: Vec<u16> = entry.payload().iter().map(|v| v.parse().unwrap()).collect();
        (p[0], p[1], p[2], p[3])
    }

    pub fn value(context: &str, payload: &str) {
        let entry = pop(context);
        if entry.payload().join(" ") != payload {
            panic!(
                "Computed {context} {payload}, trace:{} has: {}",
                entry.trace_lineno, entry.line
            );
        }
    }

    pub fn record(context: &str, fields: &[(&str, super::Value)]) {
        let entry = pop(context);
        let computed: Vec<String> = fields
            .iter()
            .map(|(name, value)| format!("{name}={}", value.encode()))
            .collect();
        let traced = entry.payload();
        if computed
            .iter()
            .map(String::as_str)
            .ne(traced.iter().copied())
        {
            let mut msg = format!("{context} differs from trace:{}\n", entry.trace_lineno);
            for (i, c) in computed.iter().enumerate() {
                let t = traced.get(i).copied().unwrap_or("<missing>");
                let mark = if c == t { "  " } else { "!=" };
                msg += &format!("  {mark} computed {:28} traced {:28}", c, t);
                if c != t {
                    msg += &format!(" ({} vs {})", super::decode(c), super::decode(t));
                }
                msg += "\n";
            }
            panic!("{msg}");
        }
    }

    pub fn rendered_frames() -> i32 {
        STATE.with(|s| s.borrow().rendered_frames)
    }

    pub fn frame(buffer: &[u8], palette: &[[u8; 3]; 256]) {
        let frame_num = STATE.with(|s| {
            let mut s = s.borrow_mut();
            let n = s.rendered_frames;
            s.rendered_frames += 1;
            n
        });
        let entry = pop("Frame");
        let p = entry.payload();
        if p[0].parse::<i32>().unwrap() != frame_num {
            panic!(
                "Rendered frame {frame_num}, trace:{} has: {}",
                entry.trace_lineno, entry.line
            );
        }
        let palette: Vec<u8> = palette.iter().flatten().copied().collect();
        if crc32(&[buffer, &palette]) != u32::from_str_radix(p[1], 16).unwrap() {
            frame_differs(frame_num, &entry, buffer, &palette);
        }
        STATE.with(|s| s.borrow_mut().checked_frames += 1);
    }

    fn frame_differs(frame_num: i32, entry: &Entry, buffer: &[u8], palette: &[u8]) -> ! {
        let mut out = File::create("actual.dat").unwrap();
        out.write_all(buffer).unwrap();
        out.write_all(palette).unwrap();
        let (frame_dir, run) = STATE.with(|s| {
            let s = s.borrow();
            (s.frame_dir.clone(), s.run.clone().unwrap())
        });
        let expected = frame_dir.and_then(|dir| fs::read(format!("{dir}/{frame_num:06}.dat")).ok());
        let Some(expected) = expected else {
            let dump_dir = format!("{}-frame{frame_num}", run.dir);
            panic!(
                "Frame {frame_num} differs from trace:{}. Actual frame written to actual.dat. \
                 To dump the Pascal frame and compare it, run:\n  \
                 SJ3_FRAMES_FROM={frame_num} SJ3_FRAMES_TO={frame_num} SJ3_MAXFRAMES={} \
                 trace/run-sdl.sh {} {dump_dir} {}\n  \
                 trace/run-rust.sh {dump_dir} {dump_dir}-rust frames",
                entry.trace_lineno,
                frame_num + 1,
                run.keys,
                run.seed,
            );
        };
        let (expected_buffer, expected_palette) = expected.split_at(buffer.len());
        let pixels = buffer
            .iter()
            .zip(expected_buffer)
            .filter(|(a, e)| a != e)
            .count();
        let colors = (0..256)
            .filter(|&i| palette[i * 3..i * 3 + 3] != expected_palette[i * 3..i * 3 + 3])
            .collect::<Vec<_>>();
        panic!(
            "Frame {frame_num} differs: {pixels} pixels, palette entries {colors:?}. Actual frame written to actual.dat"
        );
    }

    /// CRC-32 of zlib
    pub fn crc32(chunks: &[&[u8]]) -> u32 {
        const TABLE: [u32; 256] = {
            let mut table = [0; 256];
            let mut n = 0;
            while n < 256 {
                let mut c = n as u32;
                let mut k = 0;
                while k < 8 {
                    c = if c & 1 != 0 {
                        0xEDB88320 ^ (c >> 1)
                    } else {
                        c >> 1
                    };
                    k += 1;
                }
                table[n] = c;
                n += 1;
            }
            table
        };
        let mut crc = !0u32;
        for chunk in chunks {
            for &b in *chunk {
                crc = TABLE[((crc ^ b as u32) & 0xFF) as usize] ^ (crc >> 8);
            }
        }
        !crc
    }
}

#[cfg(feature = "trace")]
pub use imp::*;

#[cfg(not(feature = "trace"))]
mod noop {
    pub fn active() -> bool {
        false
    }
    pub fn key_pressed() -> bool {
        unreachable!()
    }
    pub fn keypress() -> (u8, u8) {
        unreachable!()
    }
    pub fn random(_range: u32) -> u32 {
        unreachable!()
    }
    pub fn find_files(_pattern: &str) -> Vec<String> {
        unreachable!()
    }
    pub fn file_written(_name: &str, _content: &[u8]) {}
    pub fn file_removed(_name: &str) {}
    pub fn start_end_text() {}
    pub fn console(_text: &[u8]) {}
    pub fn date() -> (u16, u16, u16, u16) {
        unreachable!()
    }
    pub fn time() -> (u16, u16, u16, u16) {
        unreachable!()
    }
    pub fn frame(_buffer: &[u8], _palette: &[[u8; 3]; 256]) {}
    pub fn value(_context: &str, _payload: &str) {}
    pub fn record(_context: &str, _fields: &[(&str, super::Value)]) {}
    pub fn crc32(_chunks: &[&[u8]]) -> u32 {
        0
    }
    pub fn rendered_frames() -> i32 {
        0
    }
    pub fn finish(_exitcode: i32) {}
}

#[cfg(not(feature = "trace"))]
pub use noop::*;

pub enum Value {
    F32(f32),
    I32(i32),
    Bool(bool),
    Str(Vec<u8>),
}

impl Value {
    #[cfg_attr(not(feature = "trace"), allow(dead_code))]
    fn encode(&self) -> String {
        match self {
            Value::F32(v) => format!("f32:{:08X}", v.to_bits()),
            Value::I32(v) => format!("i32:{v}"),
            Value::Bool(v) => format!("bool:{v}"),
            Value::Str(v) => {
                let mut s = String::from("str:");
                for &c in v {
                    match c {
                        b'\\' => s += "\\\\",
                        b'!'..=b'~' => s.push(c as char),
                        _ => s += &format!("\\x{c:02X}"),
                    }
                }
                s
            }
        }
    }
}

#[cfg_attr(not(feature = "trace"), allow(dead_code))]
fn decode(field: &str) -> String {
    match field.split_once("=f32:") {
        Some((_, bits)) => u32::from_str_radix(bits, 16)
            .map(|b| format!("{:e}", f32::from_bits(b)))
            .unwrap_or_default(),
        None => String::new(),
    }
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Value::F32(v)
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value::I32(v)
    }
}

impl From<u8> for Value {
    fn from(v: u8) -> Self {
        Value::I32(v as i32)
    }
}

impl From<&[u8]> for Value {
    fn from(v: &[u8]) -> Self {
        Value::Str(v.to_vec())
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}

#[cfg(all(test, feature = "trace"))]
mod tests {
    use super::*;

    #[test]
    fn finds_files_in_the_trace() {
        let path = std::env::temp_dir().join(format!("sj3-trace-{}.txt", std::process::id()));
        std::fs::write(
            &path,
            "TRACE.PAS:1 Start seed=1 keys=k\n\
             SJ3UNIT.PAS:2 FindFirst *.SJH A.SJH\n\
             SJ3UNIT.PAS:3 FindNext MY HILL.SJH\n\
             SJ3UNIT.PAS:3 FindNext\n\
             SJ3INFO.PAS:4 FindFirst *.SJC\n",
        )
        .unwrap();
        load(path.to_str().unwrap(), None);
        std::fs::remove_file(&path).unwrap();
        assert_eq!(find_files("*.SJH"), ["A.SJH", "MY HILL.SJH"]);
        assert!(find_files("*.SJC").is_empty());
    }
}
