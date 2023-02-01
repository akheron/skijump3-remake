use std::cell::{Cell, RefCell};

pub const X_SIZE: u32 = 1024;
pub const Y_SIZE: u32 = 512;
pub const ALUE: u32 = X_SIZE * Y_SIZE;
pub const SIVUJA: u32 = 16;
pub const SIVU_KOKO: u32 = ALUE / SIVUJA;

pub struct MakiModule {
    pub linjan_pituus: [u16; Y_SIZE as usize],
    pub profiili_y: [i16; 1301],

    pub x: i32,
    pub y: i32,

    pub graffa: RefCell<[u8; (ALUE * 2 + 1024) as usize]>, //{ osoite m�en grafiikkaan }
    pub video: RefCell<[u8; 64000]>,                       //{ osoite v�lipuskuriin }

    siirto_osoite: Cell<u32>,
}

impl MakiModule {
    pub fn new() -> Self {
        MakiModule {
            linjan_pituus: [0; Y_SIZE as usize],
            profiili_y: [0; 1301],

            x: 0,
            y: 0,

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
        unimplemented!()
    }
    pub fn alusta(&self) -> bool {
        unimplemented!()
    }
    pub fn lopeta(&self) {
        unimplemented!()
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
        unimplemented!()
    }
}
