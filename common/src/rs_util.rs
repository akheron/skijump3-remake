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

/// `Val` of Pascal: 0 when the text is not a number
pub fn val(text: &[u8]) -> i32 {
    let text = text.trim_ascii_start();
    std::str::from_utf8(text)
        .ok()
        .and_then(|text| text.parse().ok())
        .unwrap_or(0)
}

pub fn random(max: u32) -> u32 {
    if trace::active() {
        return trace::random(max);
    }
    (rand::random::<f64>() * (max as f64)) as u32
}

/// `Write` to the console. The terminal gets the text in UTF-8, decoded from
/// CP437, the code page of the DOS console.
pub fn write(text: &[u8]) {
    write_decoded(text, cp437_char);
}

/// `WriteLn` to the console
pub fn writeln(text: &[u8]) {
    write(&[text, b"\n"].concat());
}

/// `WriteLn` to the console of text in Latin-1
pub fn writeln_latin1(text: &[u8]) {
    write_decoded(&[text, b"\n"].concat(), |c| c as char);
}

/// The trace gets the bytes of the Pascal game, and the terminal gets UTF-8.
fn write_decoded(text: &[u8], decode: fn(u8) -> char) {
    if trace::active() {
        trace::console(text);
    }
    let utf8: String = text.iter().map(|&c| decode(c)).collect();
    let mut stdout = std::io::stdout();
    let _ = stdout.write_all(utf8.as_bytes());
    let _ = stdout.flush();
}

fn cp437_char(c: u8) -> char {
    const CP437_HIGH: &str = "\
        ÇüéâäàåçêëèïîìÄÅÉæÆôöòûùÿÖÜ¢£¥₧ƒáíóúñÑªº¿⌐¬½¼¡«»\
        ░▒▓│┤╡╢╖╕╣║╗╝╜╛┐└┴┬├─┼╞╟╚╔╩╦╠═╬╧╨╤╥╙╘╒╓╫╪┘┌█▄▌▐▀\
        αßΓπΣσµτΦΘΩδ∞φε∩≡±≥≤⌠⌡÷≈°∙·√ⁿ²■\u{a0}";
    match c {
        0..=127 => c as char,
        _ => CP437_HIGH.chars().nth(c as usize - 128).unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use super::cp437_char;

    #[test]
    fn cp437_upper_half() {
        assert_eq!(cp437_char(0x94), 'ö');
        assert_eq!(cp437_char(0xDB), '█');
        assert_eq!(cp437_char(0xFF), '\u{a0}');
    }
}
