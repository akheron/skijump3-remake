use crate::trace;
use std::io::{BufRead, BufReader, Read, Write};
use std::str::FromStr;

pub fn read_line(f: &mut BufReader<impl Read>) -> std::io::Result<Vec<u8>> {
    let mut line = Vec::new();
    f.read_until(0xA, &mut line)?;
    while let Some(&b'\r') | Some(&b'\n') = line.last() {
        line.pop();
    }
    Ok(line)
}

pub fn parse_line<T>(f: &mut BufReader<impl Read>) -> eyre::Result<T>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    let mut line = String::new();
    f.read_line(&mut line)?;
    Ok(line.trim().parse()?)
}

pub fn random(max: u32) -> u32 {
    if trace::active() {
        return trace::random(max);
    }
    (rand::random::<f64>() * (max as f64)) as u32
}

/// `Write` to the console
pub fn write(text: &[u8]) {
    if trace::active() {
        trace::console(text);
    }
    let mut stdout = std::io::stdout();
    let _ = stdout.write_all(text);
    let _ = stdout.flush();
}

/// `WriteLn` to the console
pub fn writeln(text: &[u8]) {
    write(&[text, b"\n"].concat());
}
