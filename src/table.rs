pub fn parru_anim(kulmalaskuri: &mut i32) -> u8 {
    let mut jumper_anim: u8 = 0;
    match *kulmalaskuri {
        1000..=1012 => jumper_anim = 169,
        1013..=1033 => jumper_anim = 170,
        1034..=1046 => jumper_anim = 169,
        1047..=1059 => jumper_anim = 164,
        1060..=1072 => jumper_anim = 171,
        1073..=1093 => jumper_anim = 172,
        1094..=1106 => jumper_anim = 171,
        1107 => *kulmalaskuri = 0,

        2000..=2012 => jumper_anim = 173,
        2013..=2025 => jumper_anim = 174,
        2026..=2036 => jumper_anim = 175,
        2037..=2047 => jumper_anim = 174,
        2048..=2058 => jumper_anim = 175,
        2059..=2071 => jumper_anim = 174,
        2072..=2082 => jumper_anim = 173,
        2083 => *kulmalaskuri = 0,

        3000..=3024 => jumper_anim = 176,
        3025..=3037 => jumper_anim = 177,
        3038..=3200 => jumper_anim = 164,
        3201 => *kulmalaskuri = 0,

        _ => {}
    }

    if *kulmalaskuri == 0 {
        jumper_anim = 164;
    }

    jumper_anim
}

pub fn suksi_laskussa(kulma: i32) -> u8 {
    let value: u8;
    match kulma {
        4..=6 => value = 1,
        7..=9 => value = 2,
        10..=12 => value = 3,
        13..=16 => value = 4,
        17..=19 => value = 5,
        20..=24 => value = 6,
        25..=28 => value = 7,
        29..=33 => value = 8,
        34..=39 => value = 9,
        40..=48 => value = 10,
        49..=64 => value = 11,
        65..=90 => value = 12,
        _ => value = 0,
    }
    value + 71
}
