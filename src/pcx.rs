use crate::maki::{MakiModule, SIVU_KOKO, X_SIZE};
use crate::sdlport::SDLPortModule;
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;

pub const NUM_SKIS: u16 = 4;
pub const NUM_SUITS: u16 = 8;

const REPLACE_MENU: [u8; 6 * 2 * 3] = [
    20, 20, 20, 26, 26, 26, 10, 10, 10, 15, 15, 15, 28, 8, 24, 34, 13, 28, 0, 24, 24, 6, 30, 30, 0,
    25, 0, 5, 30, 5, 47, 0, 0, 54, 10, 10,
];

const REPLACE_LOGO: [u8; 6 * 4] = [
    46, 46, 63, 32, 32, 63, //{ sininen logo }
    51, 51, 51, 38, 38, 38, //{ harmaa logo }
    54, 10, 10, 47, 0, 0, //{ punainen valo! }
    10, 54, 10, 0, 47, 0, //{ vihre� valo! }
];

const SUITS: [[u8; 4]; NUM_SUITS as usize] = [
    [0, 53, 17, 53], //{ Violetti }
    [0, 55, 33, 11], //{ Oranssi }
    [0, 11, 48, 18], //{ Vihre� }
    [0, 24, 28, 63], //{ Sininen }
    [0, 63, 17, 17], //{ Punainen }
    [0, 33, 33, 33], //{ Harmaa }
    [1, 10, 10, 10], //{ Musta, feidi yl�s }
    [0, 45, 17, 63], //{ Lila }
];

const SKIS: [[u8; 3]; NUM_SKIS as usize] = [
    [63, 63, 32], //{ keltainen default }
    [60, 60, 60], //{ valkoiset }
    [33, 60, 33], //{ vihre�t }
    [63, 43, 43], //{ punaiset }
];

const STANDARDI_PALETTI: [u8; 40 * 3] = [
    53, 17, 53, 63, 0, 0, 43, 12, 43, 63, 0, 0, 49, 45, 0, 34, 31, 0, 63, 0, 0, 56, 54, 54, 63, 63,
    21, 54, 52, 10, 42, 42, 42, 42, 20, 10, 21, 21, 21, 57, 45, 38, 63, 0, 0, 63, 63, 32, 40, 40,
    41, 48, 48, 49, 55, 55, 56, 63, 63, 63, 56, 13, 13, 13, 53, 13, 23, 23, 63, 63, 23, 23, 63, 63,
    63, 44, 44, 44, 0, 0, 0, 18, 13, 34, 34, 13, 18, 20, 20, 20, 63, 57, 9, 9, 57, 63, 23, 16, 43,
    43, 16, 23, 26, 26, 26, 52, 47, 0, 0, 47, 52, 51, 51, 51, 38, 38, 38, 63, 63, 63,
];

pub struct PcxModule<'m, 's, 'si> {
    m: &'m MakiModule,
    s: &'s SDLPortModule<'si>,
    pub paletti: RefCell<[[u8; 3]; 256]>,
    pub pelasta_alkuosa: RefCell<[[u8; 3]; 256]>,
}

impl<'m, 's, 'si> PcxModule<'m, 's, 'si> {
    pub fn new(m: &'m MakiModule, s: &'s SDLPortModule<'si>) -> Self {
        PcxModule {
            m,
            s,
            paletti: RefCell::new([[0; 3]; 256]),
            pelasta_alkuosa: RefCell::new([[0; 3]; 256]),
        }
    }

    pub fn aseta_paletti(&self) {
        self.s.set_palette(&self.paletti.borrow());
    }

    pub fn siirra_standardi_paletti(&self) {
        let mut paletti = self.paletti.borrow_mut();
        for temp1 in 0..=39 {
            for temp2 in 0..=2 {
                paletti[216 + temp1][temp2] = STANDARDI_PALETTI[temp1 * 3 + temp2];
            }
        }
    }
    pub fn siirra_liivi_pois(&self) {
        let mut paletti = self.paletti.borrow_mut();
        for temp in 0..=2 {
            paletti[220][temp] = paletti[216][temp];
        }
        for temp in 0..=2 {
            paletti[221][temp] = paletti[218][temp];
        }
    }
    pub fn muuta_logo(&self, col: u8) {
        let mut paletti = self.paletti.borrow_mut();
        for temp1 in 0..=1 {
            for temp2 in 0..=2 {
                paletti[253 + temp1][temp2] = REPLACE_LOGO[(col as usize + temp1) * 3 + temp2];
            }
        }
    }
    pub fn muuta_replay(&self, mode: u8) {
        unimplemented!()
    }
    pub fn muuta_menu(&self, index: u8, col: u8) {
        let mut paletti = self.paletti.borrow_mut();
        for temp in 0..=2 {
            paletti[(242 + index) as usize][temp] = REPLACE_MENU[((col * 2) * 3) as usize + temp];
        }

        for temp in 0..2 {
            paletti[(247 + index) as usize][temp] =
                REPLACE_MENU[(((col * 2) + 1) * 3) as usize + temp];
        }
    }
    pub fn tumma_lumi(&self) {
        let mut paletti = self.paletti.borrow_mut();
        for temp in 232..=235 {
            for temp2 in 0..=2 {
                paletti[temp as usize][temp2 as usize] -=
                    f32::round(0.4 * (paletti[temp as usize][temp2 as usize] as f32 - 32.0)) as u8;
            }
        }
    }
    pub fn savyta_paletti(&self, alue: u8, bkbright: u8) {
        let mut start: i32 = 0;
        let mut fin: i32 = 239;

        if alue == 1 {
            start = 64;
            fin = 215;
        }

        let r1 = bkbright as f32 / 100.0;

        let mut paletti = self.paletti.borrow_mut();
        for temp1 in start..=fin {
            for temp2 in 0..=2 {
                paletti[temp1 as usize][temp2 as usize] =
                    (paletti[temp1 as usize][temp2 as usize] as f32 * r1) as u8;
                if paletti[temp1 as usize][temp2 as usize] > 63 {
                    paletti[temp1 as usize][temp2 as usize] = 63;
                }
            }
        }
    }
    pub fn special_main_paletti(&self) {
        {
            let mut paletti = self.paletti.borrow_mut();
            paletti[243][0] = 18;
            paletti[243][1] = 13;
            paletti[243][2] = 34;

            paletti[248][0] = 23;
            paletti[248][1] = 16;
            paletti[248][2] = 43;
        }
        self.muuta_menu(3, 0);
    }
    pub fn tallenna_alkuosa(&self, alue: u8) {
        let mut len = 240;
        if alue == 1 {
            len = 64;
        }
        self.pelasta_alkuosa.borrow_mut()[0..len].copy_from_slice(&self.paletti.borrow()[0..len]);
    }
    pub fn takaisin_alkuosa(&self, alue: u8) {
        let mut len = 240;
        if alue == 1 {
            len = 64;
        }
        self.paletti.borrow_mut()[0..len].copy_from_slice(&self.pelasta_alkuosa.borrow()[0..len]);
        self.paletti.borrow_mut()[0][0] = 0;
    }
    pub fn load_skis(&self, mut col: u8, phase: u8) {
        let mut target = 231;
        if phase > 0 {
            target = phase * 5;
        }
        if col > (NUM_SKIS - 1) as u8 {
            col = 0;
        }

        let mut paletti = self.paletti.borrow_mut();
        paletti[target as usize][0] = SKIS[col as usize][0];
        paletti[target as usize][1] = SKIS[col as usize][1];
        paletti[target as usize][2] = SKIS[col as usize][2];
    }
    pub fn load_suit(&self, mut col: u8, phase: u8) {
        const NUMS: [[f32; 4]; 2] = [[1.0, 0.87, 0.75, 0.63], [1.0, 1.5, 2.0, 2.5]];
        let mut target = 215;
        if phase > 0 {
            target = phase * 5;
        }
        if col > (NUM_SUITS - 1) as u8 {
            col = 0;
        }
        let w1 = SUITS[col as usize][1];
        let w2 = SUITS[col as usize][2];
        let w3 = SUITS[col as usize][3];

        let mut paletti = self.paletti.borrow_mut();
        for temp in 0..=3 {
            paletti[(target + temp) as usize][0] =
                f32::round(NUMS[SUITS[col as usize][0] as usize][temp as usize] * w1 as f32) as u8;
            paletti[(target + temp) as usize][1] =
                f32::round(NUMS[SUITS[col as usize][0] as usize][temp as usize] * w2 as f32) as u8;
            paletti[(target + temp) as usize][2] =
                f32::round(NUMS[SUITS[col as usize][0] as usize][temp as usize] * w3 as f32) as u8;
        }
    }

    pub fn lataa_pcx(&self, name: &str, picsize: i32, mut page: u32, mirror: u8) -> bool {
        let mut wp = 0; //{ "write position" }
        let mut pc = 0; //{ "page counter" }

        let put_byte = |b: u8, pos: u32| {
            let mut video = self.m.video.borrow_mut();
            if mirror > 0 {
                let temp = pos % X_SIZE;
                let mut x = 671_i32 - temp as i32;
                if x < 0 {
                    x += 1024
                };
                video[(((pos / X_SIZE) * X_SIZE) + x as u32) as usize] = b;
            } else {
                video[pos as usize] = b;
            }
        };

        let mut file = File::open(name).unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();

        let mut offset = 128;
        self.m.lukitse_kirjoitus_sivu(page);

        while wp < picsize {
            let b1 = buf[offset];
            offset += 1;
            if b1 >= 192 {
                let b2 = buf[offset];
                offset += 1;
                for i in 0..(b1 - 192) {
                    put_byte(b2, pc);

                    wp += 1;
                    pc += 1;
                    if pc >= SIVU_KOKO {
                        pc = 0;
                        self.m.paivita_kirjoitus_sivu();
                        page += 1;
                        self.m.lukitse_kirjoitus_sivu(page);
                    }
                }
            } else {
                put_byte(b1, pc);

                wp += 1;
                pc += 1;
                if pc >= SIVU_KOKO {
                    pc = 0;
                    self.m.paivita_kirjoitus_sivu();
                    page += 1;
                    self.m.lukitse_kirjoitus_sivu(page);
                }
            }
        }

        {
            offset += 1;
            let mut paletti = self.paletti.borrow_mut();
            for pc in 0..=767 {
                paletti[pc / 3][pc % 3] = buf[offset] >> 2;
                offset += 1
            }
        }

        self.m.paivita_kirjoitus_sivu();
        true
    }
}
