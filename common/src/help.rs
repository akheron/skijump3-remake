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

pub fn nsqrt(x: f32) -> f32 {
    let temp = f32::sqrt(f32::abs(x));
    if x < 0.0 {
        -temp
    } else {
        temp
    }
}

pub fn pcomp(score: i32, sij: i32) -> i32 {
    (score * 51) + 50 - sij
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
    let s = String::from_utf8_lossy(jokuluku);
    s.parse().unwrap_or(-1)
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
