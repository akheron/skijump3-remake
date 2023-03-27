use std::cell::{Cell, RefCell};

pub const X_SIZE: u32 = 1024;
pub const Y_SIZE: u32 = 512;
pub const ALUE: u32 = X_SIZE * Y_SIZE;
pub const SIVUJA: u32 = 16;
pub const SIVU_KOKO: u32 = ALUE / SIVUJA;

pub struct MakiModule {
    pub linjan_pituus: RefCell<[u16; Y_SIZE as usize]>,
    pub profiili_y: RefCell<[i16; 1301]>,

    pub x: Cell<i32>,
    pub y: Cell<i32>,

    pub graffa: RefCell<[u8; (ALUE * 2 + 1024) as usize]>, //{ osoite m�en grafiikkaan }
    pub video: RefCell<[u8; 64000]>,                       //{ osoite v�lipuskuriin }

    siirto_osoite: Cell<u32>,
}

impl MakiModule {
    pub fn new() -> Self {
        MakiModule {
            linjan_pituus: RefCell::new([0; Y_SIZE as usize]),
            profiili_y: RefCell::new([0; 1301]),

            x: Cell::new(0),
            y: Cell::new(0),

            siirto_osoite: Cell::new(0),
            graffa: RefCell::new([0; (ALUE * 2 + 1024) as usize]),
            video: RefCell::new([0; 64000]),
        }
    }

    pub fn lue(&self, osoite: i32) -> u8 {
        self.graffa.borrow()[osoite as usize]
    }

    pub fn kirjoita(&self, osoite: i32, arvo: u8) {
        self.graffa.borrow_mut()[osoite as usize] = arvo;
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
        // The original code just allocates some buffers and sets some function pointers.
        true
    }
    pub fn lopeta(&self) {
        // Empty in the original sources
    }

    fn kopioi_maki(&self, osoite: i32, delta: i32) {
        let linjan_pituus = self.linjan_pituus.borrow();
        let mut video = self.video.borrow_mut();
        let graffa = self.graffa.borrow();

        let mut output: usize = 0;
        let mut d: i32;

        for y_index in 0..200 {
            let mut input = osoite + y_index * X_SIZE as i32 + self.x.get();
            let line = linjan_pituus[(y_index + self.y.get()) as usize];

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
        /*{ linjojen pituudet voisi ladata nopeammin levylt� ... }*/
        *keula_x = 0;
        let mut former_y: i32 = 0;

        let mut linjan_pituus = self.linjan_pituus.borrow_mut();
        for y in 0..Y_SIZE {
            linjan_pituus[y as usize] = 0;
            for x in 0..X_SIZE {
                if self.lue((y * X_SIZE + x) as i32) != 0 {
                    linjan_pituus[y as usize] = (x + 1) as u16;
                }
            }
        }

        let mut profiili_y = self.profiili_y.borrow_mut();
        for x in 0..X_SIZE {
            for y in 0..Y_SIZE {
                profiili_y[x as usize] = y as i16;
                if linjan_pituus[y as usize] > x as u16 {
                    break;
                }
            }
        }

        for x in X_SIZE..1300 {
            profiili_y[x as usize] = profiili_y[X_SIZE as usize - 1];
        }

        for x in 0..X_SIZE {
            let y = profiili_y[x as usize];
            if y as i32 - former_y > 3 {
                *keula_x = x as i32; //{ etsit��n keulan paikka }
            }
            former_y = y as i32;
        }

        *keula_x -= 1; //{ se mennee yhden liian pitk�ksi }

        for x in *keula_x..(X_SIZE - 10) as i32 {
            let x2 = x - *keula_x; //{ suhteellinen keulan alap��h�n X }
            let y2 = profiili_y[x as usize] as i32 - profiili_y[*keula_x as usize] as i32; //{ suhteellinen Y }
            let hp = f32::round(f32::sqrt((x2 * x2 + y2 * y2) as f32) * pk * 0.5) as i32 * 5; //{ +10? }
            if hp >= (2 * kr / 3) * 10 && hp <= kr * 12 {
                let c = if hp < kr * 10 { 238 } else { 239 };
                for y in 0..=2 {
                    self.kirjoita(
                        (profiili_y[x as usize] as i32 + y + 1) * X_SIZE as i32 + x,
                        c,
                    );
                }
            }
        }
    }
    pub fn aseta_moodi(&self, m: u16) {
        unimplemented!()
    }
    pub fn profiili(&self, x: i32) -> i32 {
        let mut temp: i32 = 0;
        if (x > 0) && (x < 1300) {
            temp = self.profiili_y.borrow()[x as usize] as i32;
        }
        temp
    }
}
