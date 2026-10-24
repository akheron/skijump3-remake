use crate::platform::Platform;
use crate::rs_util::{clrscr, textbackground, textcolor, val_code, window, write, writeln};

/// The original `round` takes a single, so its argument is rounded to f32 first.
/// Halves are rounded away from zero.
pub fn round(r: f32) -> i32 {
    r.round() as i32
}

/// The built-in `Round` of Turbo Pascal, used by the units that do not use SJ3HELP.
/// The x87 FPU rounds halves to even.
pub fn round_half_even(x: f64) -> i32 {
    x.round_ties_even() as i32
}

/// The SDL port has no sound, so a beep is only its wait.
pub fn beep(s: &impl Platform, _beeptapa: u8) {
    s.wait(2);
}

pub fn nsqrt(x: f32) -> f32 {
    let temp = f32::sqrt(f32::abs(x));
    if x < 0.0 {
        -temp
    } else {
        temp
    }
}

pub fn cwrite(col: u8, str1: &[u8]) {
    textcolor(col);
    write(str1);
}

pub fn cwriteln(col: u8, str1: &[u8]) {
    textcolor(col);
    writeln(str1);
}

pub fn dowindow(x1: u8, y1: u8, x2: u8, y2: u8, col: u8) {
    window(x1, y1, x2, y2);
    textbackground(col);
    clrscr();
}

pub fn pcomp(score: i32, sij: i32) -> i32 {
    (score * 51) + 50 - sij
}

pub fn paraletter(str1: &[u8]) -> u8 {
    let mut tempch = 0;
    let mut temp = 1;

    while str1.get(temp - 1) != Some(&b')') && temp < str1.len() {
        temp += 1;
    }

    if temp > 1 {
        tempch = str1[temp - 2];
    }

    tempch
}

pub fn txtp(mut jokuluku: i32) -> Vec<u8> {
    if jokuluku == 0 {
        b"0.0".to_vec()
    } else {
        let mut s = format!("{}", jokuluku.abs());
        s.insert(s.len() - 1, '.');
        if s.len() < 3 {
            s.insert(0, '0');
        }
        if jokuluku < 0 {
            s.insert(0, '-');
        }
        s.into_bytes()
    }
}

pub fn txt(mut jokuluku: i32) -> Vec<u8> {
    jokuluku.to_string().into_bytes()
}

pub fn num(jokuluku: &[u8]) -> i32 {
    val_code(jokuluku).unwrap_or(-1)
}

pub fn upstr(str1: &[u8]) -> Vec<u8> {
    let mut outstr = str1.to_vec();

    for index in 1..=outstr.len() {
        outstr[index - 1] = outstr[index - 1].to_ascii_uppercase();

        outstr[index - 1] = match outstr[index - 1] {
            0x86 => 0x8f, // å
            0x84 => 0x8e, // ä
            0x94 => 0x99, // ö
            0x81 => 0x9a, // ü
            0x9b => 0x9d, // ¢
            0x91 => 0x92, // æ
            c => c,
        };
    }

    outstr
}

pub fn toansi(str1: &[u8]) -> Vec<u8> {
    let mut outstr = str1.to_vec();

    for index in 1..=outstr.len() {
        outstr[index - 1] = match outstr[index - 1] {
            0x8f => 0xc2, // Å
            0x8e => 0xc4, // Ä
            0x99 => 0xd6, // Ö
            0x9a => 0xdc, // Ü
            0x9d => 0xd8, // ¥
            0x92 => 0xc6, // Æ
            0x86 => 0xe5, // å
            0x84 => 0xe4, // ä
            0x94 => 0xf6, // ö
            0x81 => 0xfc, // ü
            0x9b => 0xf8, // ¢
            0x91 => 0xe6, // æ
            0xe1 => 0xdf, //{ beta? }
            c => c,
        };
    }

    outstr
}

pub fn nmstr(str1: &[u8]) -> Vec<u8> {
    let mut outstr = upstr(str1);

    for index in 1..=outstr.len() {
        if index > 1 && outstr[index - 2] != b' ' {
            outstr[index - 1] = match outstr[index - 1] {
                c @ b'A'..=b'Z' => c + 32,
                0x8f => 0x86, // Å
                0x8e => 0x84, // Ä
                0x99 => 0x94, // Ö
                0x9a => 0x81, // Ü
                0x9d => 0x9b, // ¥
                0x92 => 0x91, // Æ
                c => c,
            };
        }
    }

    outstr
}
