use crate::graph::GraphModule;
use crate::help::txtp;
use crate::rs_util::random;
use std::cell::Cell;
use std::f32::consts::PI;

pub struct TuuliModule<'g, 'm, 's, 'si> {
    pub voim: Cell<i32>,
    pub windy: Cell<i32>,
    pub value: Cell<i32>,

    g: &'g GraphModule<'m, 's, 'si>,

    tsuun: Cell<bool>,
    traja1: Cell<i32>,
    traja2: Cell<i32>,
    tkulma: Cell<f32>,
    tuulix: Cell<i32>,
    tuuliy: Cell<i32>,
    tpaikka: Cell<u8>,
}

impl<'g, 'h, 'm, 's, 'si> TuuliModule<'g, 'm, 's, 'si> {
    pub fn new(g: &'g GraphModule<'m, 's, 'si>) -> Self {
        Self {
            voim: Cell::new(0),
            windy: Cell::new(0),
            value: Cell::new(0),

            g,

            tsuun: Cell::new(false),
            traja1: Cell::new(0),
            traja2: Cell::new(0),
            tkulma: Cell::new(0.0),
            tuulix: Cell::new(0),
            tuuliy: Cell::new(0),
            tpaikka: Cell::new(0),
        }
    }

    pub fn piirra(&self) {
        self.g.fill_box(
            self.tuulix.get() as u16 + 4,
            self.tuuliy.get() as u16 + 1,
            self.tuulix.get() as u16 + 38,
            self.tuuliy.get() as u16 + 2,
            248,
        );
        self.g.fill_box(
            self.tuulix.get() as u16 + 21,
            self.tuuliy.get() as u16 + 1,
            self.tuulix.get() as u16 + 21,
            self.tuuliy.get() as u16 + 2,
            240,
        );
        self.g
            .put_pixel(self.tuulix.get() + 21, self.tuuliy.get() + 9, 247); //{ piste; oli 252 }

        if self.value.get() > 0 {
            self.g.fill_box(
                (self.tuulix.get() + 22) as u16,
                (self.tuuliy.get() + 1) as u16,
                (self.tuulix.get() + 22 + self.value.get() / 3) as u16,
                (self.tuuliy.get() + 2) as u16,
                236,
            );
        }
        if self.value.get() < 0 {
            self.g.fill_box(
                (self.tuulix.get() + 20 + self.value.get() / 3) as u16,
                (self.tuuliy.get() + 1) as u16,
                (self.tuulix.get() + 20) as u16,
                (self.tuuliy.get() + 2) as u16,
                237,
            );
        }
        let s = txtp(self.value.get().abs());
        if self.value.get() < 0 {
            self.g
                .write_font(self.tuulix.get() + 10, self.tuuliy.get() + 5, b"-");
        }

        self.g
            .write_font(self.tuulix.get() + 15, self.tuuliy.get() + 5, &[s[0]]);
        self.g
            .write_font(self.tuulix.get() + 24, self.tuuliy.get() + 5, &[s[2]]);
    }

    pub fn hae(&self) {
        self.siirra();
        self.value.set(
            f32::round(f32::cos(PI * self.tkulma.get() / 180.0) * self.voim.get() as f32) as i32,
        );
    }

    pub fn alusta(&self, place: u8) {
        let temp1 = random(180) as i32; //{ Tuulen rajoja }
        let temp2 = random(120) as i32;
        self.windy.set(temp2); //{ ns. tuulisuusindeksi }

        self.traja1.set(temp1 - temp2);
        self.traja2.set(temp1 + temp2);
        self.tkulma
            .set((random(temp2 as u32 * 2) as i32 + self.traja1.get()) as f32);
        self.voim.set(random(50) as i32);
        if random(2) == 0 {
            self.tsuun.set(true);
        } else {
            self.tsuun.set(false);
        }

        self.aseta_paikka(place);
    }

    pub fn siirra(&self) {
        if (self.tsuun.get()) && (self.tkulma.get() > self.traja2.get() as f32) {
            self.tsuun.set(false);
        }
        if (self.tsuun.get() == false) && (self.tkulma.get() < self.traja1.get() as f32) {
            self.tsuun.set(true);
        }
        if random(50) == 0 {
            self.tsuun.set(!self.tsuun.get());
        }
        if self.tsuun.get() {
            self.tkulma
                .set(self.tkulma.get() + (random(4) as f32 / 5.0));
        } else {
            self.tkulma
                .set(self.tkulma.get() - (random(4) as f32 / 5.0));
        }
    }

    pub fn tuo(&self, mut x: i32, mut y: i32) {
        match self.tpaikka.get() {
            11 => {
                x += 10;
                y += 20;
            }
            12 => {
                x += 15;
                y += 5;
            }
            13 => {
                x -= 10;
                y += 12;
            }
            _ => {}
        }
        self.tuulix.set(x);
        self.tuuliy.set(y);
    }

    pub fn aseta_paikka(&self, place: u8) {
        self.tpaikka.set(place);

        let mut tuulix = 10;
        let mut tuuliy = 180;

        match self.tpaikka.get() {
            2 => {
                tuuliy = 97;
            }
            3 => {
                tuulix = 268;
            }
            4 => {
                tuulix = 150;
            }
            5 => {
                tuulix = 268;
                tuuliy = 97;
            }
            6 => {
                tuulix = 268;
                tuuliy = 21;
            }
            7 => {
                tuulix = 150;
                tuuliy = 21;
            }
            8 => {
                tuulix = 56;
                tuuliy = 33;
            }
            _ => {}
        }
        self.tuulix.set(tuulix);
        self.tuuliy.set(tuuliy);
    }
}
