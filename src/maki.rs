use std::cell::{Cell, RefCell};

pub const X_SIZE: u32 = 1024;
pub const Y_SIZE: u32 = 512;
pub const ALUE: u32 = X_SIZE * Y_SIZE;
pub const SIVUJA: u32 = 16;
pub const SIVU_KOKO: u32 = ALUE / SIVUJA;

pub struct MakiModule {
    pub linjan_pituus: [u16; Y_SIZE as usize],
    pub profiili_y: [i16; 1301],

    pub x: Cell<i32>,
    pub y: Cell<i32>,

    pub graffa: RefCell<[u8; (ALUE * 2 + 1024) as usize]>, //{ osoite m�en grafiikkaan }
    pub video: RefCell<[u8; 64000]>,                       //{ osoite v�lipuskuriin }

    siirto_osoite: Cell<u32>,
}

impl MakiModule {
    pub fn new() -> Self {
        MakiModule {
            linjan_pituus: [0; Y_SIZE as usize],
            profiili_y: [0; 1301],

            x: Cell::new(0),
            y: Cell::new(0),

            siirto_osoite: Cell::new(0),
            graffa: RefCell::new([0; (ALUE * 2 + 1024) as usize]),
            video: RefCell::new([0; 64000]),
        }
    }

    pub fn lue(&self, osoite: i32) -> u8 {
        unimplemented!()
    }

    pub fn kirjoita(&self, osoite: i32, arvo: u8) {
        unimplemented!()
    }

    pub fn paivita_kirjoitus_sivu(&self) {
        let mut graffa = self.graffa.borrow_mut();
        let video = self.video.borrow();
        let siirto_osoite = self.siirto_osoite.get() as usize;
        for index in 0..=((SIVU_KOKO - 1) as usize) {
            graffa[siirto_osoite + index] = video[index];
        }
    }

    pub fn tulosta(&self) {
        let a: i32 = self.y.get() * X_SIZE as i32;
        let b: i32 = ALUE as i32 - (self.x.get() >> 1) - (self.y.get() >> 1) * X_SIZE as i32;
        self.kopioi_maki(a, b);
    }
    pub fn alusta(&self) -> bool {
        unimplemented!()
    }
    pub fn lopeta(&self) {
        unimplemented!()
    }

    fn kopioi_maki(&self, osoite: i32, delta: i32) {
        let mut video = self.video.borrow_mut();
        let graffa = self.graffa.borrow();

        let mut output: usize = 0;
        let mut d: i32;
        for y_index in 0..200i32 {
            let mut input = osoite + y_index * X_SIZE as i32 + self.x.get();
            let line = self.linjan_pituus[(y_index + self.y.get()) as usize];

            for x_index in 0..320 {
                if x_index + self.x.get() >= line as i32 {
                    d = delta;
                } else {
                    d = 0;
                }

                video[output] = graffa[(input + d) as usize];
                input += 1;
                output += 1;
            }
        }
    }

    pub fn lukitse_kirjoitus_sivu(&self, sivu: u32) {
        if sivu < SIVUJA * 2 {
            self.siirto_osoite.set(sivu * SIVU_KOKO);
        }
    }

    pub fn laske_linjat(&self, keula_x: &mut i32, kr: i32, pk: f32) {
        unimplemented!()
    }
    pub fn aseta_moodi(&self, m: u16) {
        unimplemented!()
    }
    pub fn profiili(&self, x: i32) -> i32 {
        let mut temp: i32 = 0;
        if (x > 0) && (x < 1300) {
            temp = self.profiili_y[x as usize] as i32;
        }
        temp
    }
}
