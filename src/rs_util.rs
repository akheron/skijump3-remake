use std::cell::Cell;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

pub fn read_line(f: &mut BufReader<File>) -> std::io::Result<Vec<u8>> {
    let mut line = Vec::new();
    f.read_until(0xA, &mut line)?;
    while let Some(&b'\r') | Some(&b'\n') = line.last() {
        line.pop();
    }
    Ok(line)
}

pub fn parse_line<T>(f: &mut BufReader<File>) -> eyre::Result<T>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    let mut line = String::new();
    f.read_line(&mut line)?;
    Ok(line.trim().parse()?)
}

// Linear Congruential Generator
// m = 2^32, a = 134775813, c = 1

thread_local!(static RAND_SEED: Cell<u32> = Cell::new(0));

pub fn randomize(seed: u32) {
    RAND_SEED.with(|r| r.set(seed));
}

pub fn random(max: u32) -> u32 {
    let state = RAND_SEED.with(|r| {
        r.set(((r.get() as u64) * 134775813 + 1) as u32);
        r.get()
    });
    ((state as u64 * max as u64) >> 32) as u32
}
