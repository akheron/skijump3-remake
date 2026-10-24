use crate::trace;
use std::cell::Cell;
use std::future::{poll_fn, Future};
use std::io::{BufRead, BufReader, IsTerminal, Read, Write};
use std::pin::pin;
use std::task::Poll;

pub fn read_line(f: &mut BufReader<impl Read>) -> std::io::Result<Vec<u8>> {
    let mut line = Vec::new();
    f.read_until(0xA, &mut line)?;
    while let Some(&b'\r') | Some(&b'\n') = line.last() {
        line.pop();
    }
    Ok(line)
}

/// `ReadLn` of a number in Free Pascal: skips blank lines, reads the text up
/// to the next space or line end, and gives 0 at the end of the file. An
/// error is the runtime error 106 of Pascal.
pub fn parse_line<T: PascalNumber>(f: &mut BufReader<impl Read>) -> eyre::Result<T> {
    let mut token = Vec::new();
    while token.len() < 255 {
        let Some(&c) = f.fill_buf()?.first() else {
            break;
        };
        if c > b' ' {
            token.push(c);
        } else if !token.is_empty() {
            break;
        }
        f.consume(1);
    }
    read_line(f)?;
    if token.is_empty() {
        return Ok(T::from_val(b"0").unwrap());
    }
    T::from_val(&token).ok_or_else(|| eyre::eyre!("Invalid numeric format"))
}

/// A text file that Pascal reads under `{$I-}`. A read error stays in
/// `in_out_res`, and each later read is skipped: a number reads as 0, a string
/// as empty, and `eof` is true.
pub struct UncheckedText<R: Read> {
    f: BufReader<R>,
    pub in_out_res: bool,
}

impl<R: Read> UncheckedText<R> {
    pub fn new(f: R) -> Self {
        Self {
            f: BufReader::new(f),
            in_out_res: false,
        }
    }

    pub fn readln_num<T: PascalNumber>(&mut self) -> T {
        if !self.in_out_res {
            if let Ok(value) = parse_line(&mut self.f) {
                return value;
            }
            self.in_out_res = true;
        }
        T::from_val(b"0").unwrap()
    }

    /// `ReadLn` into a `string[max_len]`
    pub fn readln_str(&mut self, max_len: usize) -> Vec<u8> {
        if self.in_out_res {
            return Vec::new();
        }
        match read_line(&mut self.f) {
            Ok(mut line) => {
                line.truncate(max_len);
                line
            }
            Err(_) => {
                self.in_out_res = true;
                Vec::new()
            }
        }
    }

    pub fn eof(&mut self) -> bool {
        self.in_out_res || self.f.fill_buf().map_or(true, |buf| buf.is_empty())
    }
}

pub trait PascalNumber: Sized {
    /// `Val` into this type, truncated as with `{$R-}`
    fn from_val(text: &[u8]) -> Option<Self>;
}

macro_rules! pascal_number {
    ($val:ident: $($t:ty),*) => {
        $(impl PascalNumber for $t {
            fn from_val(text: &[u8]) -> Option<Self> {
                $val(text).map(|value| value as $t)
            }
        })*
    };
}
pascal_number!(val_sint: i8, i16, i32, i64);
pascal_number!(val_uint: u8, u16, u32, u64);

/// `Val` of Pascal: 0 when the text is not a number
pub fn val(text: &[u8]) -> i32 {
    val_code(text).unwrap_or(0)
}

/// `Val` of Pascal into a longint: `None` when its error code is not 0
pub fn val_code(text: &[u8]) -> Option<i32> {
    val_sint(text).map(|value| value as i32)
}

/// `Val` of Free Pascal into a signed integer. The caller truncates the result
/// to its type, because Pascal does not check the range with `{$R-}`.
fn val_sint(text: &[u8]) -> Option<i64> {
    let (negative, base, digits) = init_val(text)?;
    let max_new_value = if base == 10 {
        i64::MAX as u64 + negative as u64
    } else {
        u64::MAX
    };
    let mut value: u64 = 0;
    for &c in digits {
        let u = digit_value(c);
        let prev = value;
        value = value.wrapping_mul(base);
        if u >= base || max_new_value - u < value || prev > u64::MAX / base {
            return None;
        }
        value += u;
    }
    let value = value as i64;
    Some(if negative {
        value.wrapping_neg()
    } else {
        value
    })
}

/// `Val` of Free Pascal into an unsigned integer
fn val_uint(text: &[u8]) -> Option<u64> {
    let (negative, base, digits) = init_val(text)?;
    if negative {
        return None;
    }
    let mut value: u64 = 0;
    for &c in digits {
        let u = digit_value(c);
        if u >= base || (u64::MAX - u) / base < value {
            return None;
        }
        value = value * base + u;
    }
    Some(value)
}

/// `InitVal` of Free Pascal: the sign, the base and the digits after the
/// leading zeros. `None` when there are no digits.
fn init_val(text: &[u8]) -> Option<(bool, u64, &[u8])> {
    let mut code = 0;
    while code < text.len() && (text[code] == b' ' || text[code] == b'\t') {
        code += 1;
    }
    let mut negative = false;
    match text.get(code) {
        Some(b'-') => {
            negative = true;
            code += 1;
        }
        Some(b'+') => code += 1,
        _ => {}
    }
    let mut base = 10;
    match text.get(code) {
        Some(b'$' | b'X' | b'x') => {
            base = 16;
            code += 1;
        }
        Some(b'%') => {
            base = 2;
            code += 1;
        }
        Some(b'&') => {
            base = 8;
            code += 1;
        }
        Some(b'0') if matches!(text.get(code + 1), Some(b'x' | b'X')) => {
            base = 16;
            code += 2;
        }
        _ => {}
    }
    while code + 1 < text.len() && text[code] == b'0' {
        code += 1;
    }
    if code >= text.len() {
        return None;
    }
    Some((negative, base, &text[code..]))
}

fn digit_value(c: u8) -> u64 {
    match c {
        b'0'..=b'9' => (c - b'0') as u64,
        b'A'..=b'F' => (c - b'A' + 10) as u64,
        b'a'..=b'f' => (c - b'a' + 10) as u64,
        _ => 16,
    }
}

/// `Write(text:width)` of Pascal: the text right-aligned in `width` characters
pub fn pad_left(text: &[u8], width: usize) -> Vec<u8> {
    let mut padded = vec![b' '; width.saturating_sub(text.len())];
    padded.extend_from_slice(text);
    padded
}

pub fn random(max: u32) -> u32 {
    if trace::active() {
        return trace::random(max);
    }
    (rand::random::<f64>() * (max as f64)) as u32
}

/// `Halt` of Pascal. A web page cannot end its program, so on the web the
/// game stops here and `until_halt` returns.
pub async fn halt() -> ! {
    exit(0).await
}

/// The `EInOutError` that `{$I+}` code raises when a `{$I-}` read has left a
/// read error in `InOutRes`. The game does not catch it, so the program ends
/// with exit code 217.
pub async fn invalid_input_error() -> ! {
    let text = "An unhandled exception occurred:\nEInOutError: Invalid input\n";
    #[cfg(target_family = "wasm")]
    web_console::write(text);
    let mut stdout = std::io::stdout();
    let _ = stdout.write_all(text.as_bytes());
    let _ = stdout.flush();
    exit(217).await
}

async fn exit(exitcode: i32) -> ! {
    if trace::active() {
        trace::finish(exitcode);
    }
    #[cfg(not(target_family = "wasm"))]
    std::process::exit(exitcode);
    #[cfg(target_family = "wasm")]
    {
        HALTED.with(|halted| halted.set(true));
        loop {
            std::future::pending::<()>().await;
        }
    }
}

/// Runs `program` until it ends or calls `halt`
pub async fn until_halt(program: impl Future<Output = ()>) {
    let mut program = pin!(program);
    poll_fn(|cx| {
        let poll = program.as_mut().poll(cx);
        if HALTED.with(|halted| halted.replace(false)) {
            Poll::Ready(())
        } else {
            poll
        }
    })
    .await
}

thread_local! {
    static HALTED: Cell<bool> = const { Cell::new(false) };
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

/// `TextColor` of the Crt unit. 16 and up blink.
pub fn textcolor(color: u8) {
    CRT.with(|crt| crt.text_color.set(color));
    crt_escape(&format!(
        "\x1b[0;{}{};{}m",
        if color & 16 != 0 { "5;" } else { "" },
        ansi_color(color, 30, 90),
        ansi_color(CRT.with(|crt| crt.text_background.get()), 40, 40)
    ));
}

/// `TextBackground` of the Crt unit
pub fn textbackground(color: u8) {
    CRT.with(|crt| crt.text_background.set(color & 7));
    textcolor(CRT.with(|crt| crt.text_color.get()));
}

/// `Window` of the Crt unit: `GotoXY` and `ClrScr` use this part of the
/// 80x25 screen.
pub fn window(x1: u8, y1: u8, x2: u8, y2: u8) {
    CRT.with(|crt| crt.window.set((x1, y1, x2, y2)));
    gotoxy(1, 1);
}

/// `GotoXY` of the Crt unit
pub fn gotoxy(x: u8, y: u8) {
    let (x1, y1, _, _) = CRT.with(|crt| crt.window.get());
    crt_escape(&format!("\x1b[{};{}H", y1 + y - 1, x1 + x - 1));
}

/// `ClrScr` of the Crt unit: fills the window with the background color
pub fn clrscr() {
    let (x1, y1, x2, y2) = CRT.with(|crt| crt.window.get());
    for y in 1..=y2 - y1 + 1 {
        gotoxy(1, y);
        crt_escape(&" ".repeat((x2 - x1 + 1) as usize));
    }
    gotoxy(1, 1);
}

struct Crt {
    window: Cell<(u8, u8, u8, u8)>,
    text_color: Cell<u8>,
    text_background: Cell<u8>,
}

thread_local! {
    static CRT: Crt = const {
        Crt {
            window: Cell::new((1, 1, 80, 25)),
            text_color: Cell::new(7),
            text_background: Cell::new(0),
        }
    };
}

/// The ANSI color number of a DOS color: dark colors from `dark`, bright from
/// `bright`
fn ansi_color(color: u8, dark: u8, bright: u8) -> u8 {
    const ANSI_ORDER: [u8; 8] = [0, 4, 2, 6, 1, 5, 3, 7];
    let base = if color & 8 != 0 { bright } else { dark };
    base + ANSI_ORDER[(color & 7) as usize]
}

/// The trace does not get the Crt output, so that it has only the text of
/// the Pascal game.
fn crt_escape(text: &str) {
    let mut stdout = std::io::stdout();
    if stdout.is_terminal() {
        let _ = stdout.write_all(text.as_bytes());
    }
}

/// The trace gets the bytes of the Pascal game, and the terminal gets UTF-8.
fn write_decoded(text: &[u8], decode: fn(u8) -> char) {
    if trace::active() {
        trace::console(text);
    }
    let utf8: String = text.iter().map(|&c| decode(c)).collect();
    #[cfg(target_family = "wasm")]
    web_console::write(&utf8);
    let mut stdout = std::io::stdout();
    let _ = stdout.write_all(utf8.as_bytes());
    let _ = stdout.flush();
}

/// The end text of the game comes after this. The web build does not show it.
pub fn start_end_text() {
    trace::start_end_text();
    #[cfg(target_family = "wasm")]
    web_console::hide();
}

/// Shows the console text again, for a new run of the game.
pub fn reset_console() {
    #[cfg(target_family = "wasm")]
    web_console::reset();
}

/// The web page has no console, so the browser console gets the text, one
/// line per `console.log`.
#[cfg(target_family = "wasm")]
mod web_console {
    use std::cell::{Cell, RefCell};
    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console)]
        fn log(s: &str);
    }

    thread_local! {
        static LINE: RefCell<String> = const { RefCell::new(String::new()) };
        static HIDDEN: Cell<bool> = const { Cell::new(false) };
    }

    pub fn write(text: &str) {
        if HIDDEN.with(Cell::get) {
            return;
        }
        LINE.with(|line| {
            let mut line = line.borrow_mut();
            for c in text.chars() {
                if c == '\n' {
                    log(&line);
                    line.clear();
                } else {
                    line.push(c);
                }
            }
        });
    }

    pub fn hide() {
        LINE.with(|line| {
            let mut line = line.borrow_mut();
            if !line.is_empty() {
                log(&line);
                line.clear();
            }
        });
        HIDDEN.with(|hidden| hidden.set(true));
    }

    pub fn reset() {
        LINE.with(|line| line.borrow_mut().clear());
        HIDDEN.with(|hidden| hidden.set(false));
    }
}

/// The name of a data file. DOS file names do not depend on case, and the data
/// files have upper-case names. A byte above 127 is a CP437 character.
pub fn dos_file_name(name: &[u8]) -> String {
    name.iter()
        .map(|&c| cp437_char(c.to_ascii_uppercase()))
        .collect()
}

/// The CP437 bytes of a file name, the inverse of `dos_file_name`. A character
/// that CP437 does not have becomes `?`.
pub fn dos_name_bytes(name: &str) -> Vec<u8> {
    name.chars()
        .map(|c| (0..=255).find(|&b| cp437_char(b) == c).unwrap_or(b'?'))
        .collect()
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
    use super::{
        cp437_char, dos_file_name, dos_name_bytes, parse_line, read_line, val_code, val_uint,
        UncheckedText,
    };
    use std::io::BufReader;

    #[test]
    fn cp437_upper_half() {
        assert_eq!(cp437_char(0x94), 'ö');
        assert_eq!(cp437_char(0xDB), '█');
        assert_eq!(cp437_char(0xFF), '\u{a0}');
    }

    #[test]
    fn dos_name_bytes_inverts_dos_file_name() {
        assert_eq!(
            dos_name_bytes(&dos_file_name(b"M\x8EKI.SJC")),
            b"M\x8EKI.SJC"
        );
        assert_eq!(dos_name_bytes("A\u{20ac}.SJC"), b"A?.SJC");
    }

    #[test]
    fn val_code_as_free_pascal() {
        assert_eq!(val_code(b"  -12"), Some(-12));
        assert_eq!(val_code(b"+007"), Some(7));
        assert_eq!(val_code(b"$1F"), Some(31));
        assert_eq!(val_code(b"0x1f"), Some(31));
        assert_eq!(val_code(b"%101"), Some(5));
        assert_eq!(val_code(b"&17"), Some(15));
        assert_eq!(val_code(b"$FFFFFFFF"), Some(-1));
        assert_eq!(val_code(b"4294967296"), Some(0));
        assert_eq!(val_code(b"-9223372036854775808"), Some(0));
        assert_eq!(val_code(b"9223372036854775808"), None);
        assert_eq!(val_code(b"12 "), None);
        assert_eq!(val_code(b"1x"), None);
        assert_eq!(val_code(b"$"), None);
        assert_eq!(val_code(b"-"), None);
        assert_eq!(val_code(b""), None);
        assert_eq!(val_uint(b"-1"), None);
    }

    #[test]
    fn unchecked_text_skips_reads_after_an_error() {
        let mut f = UncheckedText::new(&b"1\nx\n2\nname\n"[..]);
        assert_eq!(f.readln_num::<i16>(), 1);
        assert!(!f.eof());
        assert_eq!(f.readln_num::<i16>(), 0);
        assert!(f.in_out_res);
        assert_eq!(f.readln_num::<i16>(), 0);
        assert_eq!(f.readln_str(255), b"");
        assert!(f.eof());
    }

    #[test]
    fn parse_line_as_free_pascal_readln() {
        let mut f = BufReader::new(&b"\r\n  12 x\r\n300\n-1\nx 1\nnext"[..]);
        assert_eq!(parse_line::<i16>(&mut f).unwrap(), 12);
        assert_eq!(parse_line::<u8>(&mut f).unwrap(), 44);
        assert!(parse_line::<u8>(&mut f).is_err());
        assert!(parse_line::<i32>(&mut f).is_err());
        assert_eq!(read_line(&mut f).unwrap(), b"next");
        assert_eq!(parse_line::<i32>(&mut f).unwrap(), 0);
    }
}
