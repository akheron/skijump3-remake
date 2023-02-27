use crate::graph::GraphModule;
use crate::help::{txtp, HelpModule};
use std::cell::Cell;
use std::f32::consts::PI;

pub struct TuuliModule<'g, 'h, 'm, 's, 'si> {
    pub voim: Cell<i32>,
    pub windy: Cell<i32>,
    pub value: Cell<i32>,

    g: &'g GraphModule<'m, 's, 'si>,
    h: &'h HelpModule,

    tsuun: Cell<bool>,
    traja1: Cell<i32>,
    traja2: Cell<i32>,
    tkulma: Cell<f32>,
    tuulix: Cell<i32>,
    tuuliy: Cell<i32>,
    tpaikka: Cell<u8>,
}

impl<'g, 'h, 'm, 's, 'si> TuuliModule<'g, 'h, 'm, 's, 'si> {
    pub fn new(g: &'g GraphModule<'m, 's, 'si>, h: &'h HelpModule) -> Self {
        Self {
            voim: Cell::new(0),
            windy: Cell::new(0),
            value: Cell::new(0),

            g,
            h,

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
        let mut s: Vec<u8> = Vec::new();
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
                self.tuulix.get() as u16 + 22,
                self.tuuliy.get() as u16 + 1,
                self.tuulix.get() as u16 + 22 + self.value.get() as u16 / 3,
                self.tuuliy.get() as u16 + 2,
                236,
            );
        }
        if self.value.get() < 0 {
            self.g.fill_box(
                self.tuulix.get() as u16 + 20 + self.value.get() as u16 / 3,
                self.tuuliy.get() as u16 + 1,
                self.tuulix.get() as u16 + 20,
                self.tuuliy.get() as u16 + 2,
                237,
            );
        }
        let s = txtp(self.value.get().abs());
        if self.value.get() < 0 {
            self.g
                .write_font(self.tuulix.get() + 10, self.tuuliy.get() + 5, &vec![b'-']);
        }

        /*
        writefont(tuulix+15,tuuliy+5,s[1]);
        writefont(tuulix+24,tuuliy+5,s[3]);

        TODO:Does the previous 2 lines equal to the following 2 lines?
        */
        self.g
            .write_font(self.tuulix.get() + 15, self.tuuliy.get() + 5, &[s[0]]);
        self.g
            .write_font(self.tuulix.get() + 24, self.tuuliy.get() + 5, &[s[2]]);
    }

    pub fn hae(&self) {
        self.siirra();
        self.value
            .set(((PI * self.tkulma.get() / 180.0).cos().round() * self.voim.get() as f32) as i32);
    }

    pub fn alusta(&self, place: u8) {
        let temp1 = (rand::random::<f64>() * 180.0) as i32; // Tuulen rajoja
        let temp2 = (rand::random::<f64>() * 120.0) as i32;
        self.windy.set(temp2); // ns. tuulisuusindeksi

        self.traja1.set(temp1 - temp2);
        self.traja2.set(temp1 + temp2);
        self.tkulma
            .set((rand::random::<f64>() * (temp2 * 2) as f64) as f32 + self.traja1.get() as f32);
        self.voim.set((rand::random::<f64>() * 50 as f64) as i32);
        if rand::random::<f64>() < 0.5 {
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
        if rand::random::<f64>() < 1.0 / 50.0 {
            self.tsuun.set(!self.tsuun.get());
        }
        if self.tsuun.get() {
            self.tkulma
                .set(self.tkulma.get() + (rand::random::<f64>() * 4.0 / 5.0) as f32);
        } else {
            self.tkulma
                .set(self.tkulma.get() - (rand::random::<f64>() * 4.0 / 5.0) as f32);
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
