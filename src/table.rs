pub fn find_landing(kulma: i32) -> i32 {
    //{ korkeus, jonka j�lkeen alastulo on aloitettava }
    match kulma {
        0..=9 => 50,
        10..=20 => 50,
        21..=24 => 50,
        25 => 48,
        26 => 45,
        27 => 40,
        28 => 36,
        29 => 32,
        30 => 28,
        31 => 24,
        32 => 22,
        33..=39 => 20,
        40..=60 => 15,
        _ => 25,
    }
}

pub fn jump_risk(kulma: i32) -> i32 {
    1 + match kulma {
        31 => 1,
        30 => 2,
        29 => 4,
        28 => 7,
        27 => 12,
        26 => 19,
        25 => 29,
        24 => 41,
        23 => 54,
        22 => 70,
        21 => 90,
        20 => 120,
        19 => 200,
        18 => 300,
        17 => 500,
        16 => 700,
        0..=15 => 950,
        _ => 0,
    }
}

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
    71 + match kulma {
        4..=6 => 1,
        7..=9 => 2,
        10..=12 => 3,
        13..=16 => 4,
        17..=19 => 5,
        20..=24 => 6,
        25..=28 => 7,
        29..=33 => 8,
        34..=39 => 9,
        40..=48 => 10,
        49..=64 => 11,
        65..=90 => 12,
        _ => 0,
    }
}

pub fn lasku_asento(suksi: u8) -> u8 {
    101 + match suksi - 71 {
        2..=4 => 1,
        5..=6 => 2,
        7..=9 => 3,
        10..=12 => 4,
        _ => 0,
    }
}

pub fn lasku_anim(mut kulma: i32, style: u8) -> u8 {
    if kulma > 70 {
        kulma -= 71; //{ jos se on suksianim }
    }

    let value: u8 = match kulma {
        0..=4 => 127,
        5..=6 => 126,
        7 => 125,
        8..=12 => 124,
        _ => 127,
    };
    if style == 2 {
        value + 6
    } else {
        value
    }
}

pub fn ponn_anim(ponnphase: &mut u8) -> u8 {
    let value = match *ponnphase {
        4..=6 => 118,
        7..=9 => 119,
        10..=13 => 120,
        14..=17 => 121,
        18..=20 => 122,
        21..=23 => 123,
        24..=50 => 112, //{ TARKISTA T�M�!!! }
        _ => 117,
    };
    *ponnphase += 1;
    value
}

pub fn lento_anim(kulma1: i32) -> u8 {
    106 + match kulma1 {
        50..=61 => 1,
        62..=76 => 2,
        77..=95 => 3,
        96..=119 => 4,
        120..=149 => 5,
        150..=186 => 6,
        187..=1000 => 7,
        _ => 0,
    }
}

pub fn suksi_lennossa(kulmas: i32) -> u8 {
    //{ k�rki yl�sp�in olevia suksia }
    71 + match kulmas {
        -900..=-258 => 19,
        -257..=-216 => 18,
        -215..=-176 => 17,
        -175..=-136 => 16,
        -135..=-97 => 15,
        -96..=-59 => 14,
        -58..=-20 => 13,

        20..=58 => 1,
        59..=96 => 2,
        97..=135 => 3,
        136..=175 => 4,
        176..=215 => 5,
        216..=257 => 6,
        _ => 0,
    }
}
