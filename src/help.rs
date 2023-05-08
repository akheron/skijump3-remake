pub fn nsqrt(x: f64) -> f64 {
    let temp = f64::sqrt(f64::abs(x));
    if x < 0.0 {
        -temp
    } else {
        temp
    }
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
        s.into_bytes()
    }
}

pub fn txt(mut jokuluku: i32) -> Vec<u8> {
    jokuluku.to_string().into_bytes()
}

pub fn pcomp(score: i32, sij: i32) -> i32 {
    (score * 51) + 50 - sij
}
