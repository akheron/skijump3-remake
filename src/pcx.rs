use crate::maki::{MakiModule, SIVU_KOKO, X_SIZE};
use crate::sdlport::SDLPortModule;
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;

const NUM_SKIS: u16 = 4;
const NUM_SUITS: u16 = 8;

const REPLACE_MENU: [u8; 6 * 2 * 3] = [
    20, 20, 20, 26, 26, 26, 10, 10, 10, 15, 15, 15, 28, 8, 24, 34, 13, 28, 0, 24, 24, 6, 30, 30, 0,
    25, 0, 5, 30, 5, 47, 0, 0, 54, 10, 10,
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
    pub pelasta_alkuosa: [u8; 255 * 3],
}

impl<'m, 's, 'si> PcxModule<'m, 's, 'si> {
    pub fn new(m: &'m MakiModule, s: &'s SDLPortModule<'si>) -> Self {
        PcxModule {
            m,
            s,
            paletti: RefCell::new([[0; 3]; 256]),
            pelasta_alkuosa: [0; 255 * 3],
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
        unimplemented!()
    }
    pub fn muuta_logo(&self, col: u8) {
        unimplemented!()
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
        unimplemented!()
    }
    pub fn savyta_paletti(&self, alue: u8, bkbright: u8) {
        unimplemented!()
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
        unimplemented!()
    }
    pub fn takaisin_alkuosa(&self, alue: u8) {
        unimplemented!()
    }
    pub fn load_skis(&self, col: u8, phase: u8) {
        unimplemented!()
    }
    pub fn load_suit(&self, col: u8, phase: u8) {
        unimplemented!()
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
