use crate::rs_util::{random_i32, random_u16};
use std::f64::consts::PI;

#[derive(Clone, Copy)]
struct Lumi {
    x: i32,
    y: i32,
    gravity: i32,
    sin_pos: u16,
    c1: u16,
    c2: u16,
    style: u16,
}

const LUMI_MAX: u16 = 256;
const SINE_LGT: u16 = 512; //{ tиtyy olla 2^n }
const TAUSTA_MIN: u8 = 64; //{ taustakuvan vвit }
const TAUSTA_MAX: u8 = 215;
const G_VAIHTELU: i32 = 300; //{ putoamisnopeuden vaihtelu }
const SIVU_LIIKE: i32 = 50; //{ sivuttaisliikkeen suuruus }

pub struct LumiModule {
    lh: [Lumi; LUMI_MAX as usize],
    sine: [i32; SINE_LGT as usize],
    max: u16,
    perus_g: u16, //{ nopeus, jolla hiutaleet putoaa }
    g_vaihtelu: u16,
    sivu_liike: u16,
    sleet: bool,
}

impl LumiModule {
    pub fn init() -> LumiModule {
        let mut l = LumiModule {
            lh: [Lumi {
                x: 0,
                y: 0,
                gravity: 0,
                sin_pos: 0,
                c1: 0,
                c2: 0,
                style: 0,
            }; LUMI_MAX as usize],
            sine: [0; SINE_LGT as usize],
            max: 0,
            perus_g: 0,
            g_vaihtelu: 0,
            sivu_liike: 0,
            sleet: false,
        };
        l.reset();
        l
    }

    fn reset(&mut self) {
        for i in 0..SINE_LGT {
            self.sine[i as usize] =
                f64::round(f64::sin(i as f64 * PI * 2f64 / SINE_LGT as f64) * SIVU_LIIKE as f64)
                    as i32;
        }
        for i in 0..LUMI_MAX {
            let lumi = &mut self.lh[i as usize];
            lumi.x = random_i32(320) << 10;
            lumi.x = random_i32(320) << 10;
            lumi.y = random_i32(200) << 10;
            lumi.sin_pos = random_u16(SINE_LGT);
            lumi.gravity = random_i32(G_VAIHTELU) + self.perus_g as i32 - self.g_vaihtelu as i32;
            lumi.style = random_u16(2);
            if (self.sleet) && (lumi.style == 1) {
                lumi.style = random_u16(2);
            }
            lumi.c1 = get_color();
            lumi.c2 = get_color();
        }
    }

    pub fn vie_lmaara(&mut self, in_lmaara: u16) {
        self.perus_g = 600;
        self.g_vaihtelu = 300;
        self.sivu_liike = 50;
        self.sleet = false;

        self.max = in_lmaara;

        if self.max > 1000 {
            self.sleet = true;
        }

        if self.sleet {
            //{ r�tт }
            self.perus_g = 875;
            self.g_vaihtelu = 100;
            self.sivu_liike = 50;
            self.max = in_lmaara - 1000;
        }

        self.reset();
    }

    pub fn update(
        &mut self,
        buffer: &mut [u8],
        delta_x: i32,
        delta_y: i32,
        tuuli: i32,
        draw: bool,
    ) {
        if self.max >= LUMI_MAX {
            self.max = LUMI_MAX - 1;
        }

        for i in 0..=self.max
        //with LH[i]
        {
            let mut lumi = &mut self.lh[i as usize];
            if draw {
                lumi.x += self.sine[lumi.sin_pos as usize] + (delta_x << 9) + tuuli;
                lumi.sin_pos += 1;
                lumi.sin_pos &= SINE_LGT - 1;
                lumi.y += lumi.gravity + (delta_y << 8);
            }
            let mut offset = (lumi.x >> 10) + (lumi.y >> 10) * 320;
            if (offset < 63679)
                && (buffer[offset as usize] >= TAUSTA_MIN)
                && (buffer[offset as usize + 1] >= TAUSTA_MIN)
                && (buffer[offset as usize] < TAUSTA_MAX)
                && (buffer[offset as usize + 1] < TAUSTA_MAX)
            {
                if lumi.style == 1 {
                    buffer[offset as usize] = lumi.c1 as u8;
                    buffer[offset as usize + 1] = (lumi.c1 >> 8) as u8;
                    buffer[offset as usize + 320] = lumi.c2 as u8;
                    buffer[offset as usize + 320 + 1] = (lumi.c2 >> 8) as u8;
                } else {
                    buffer[offset as usize] = lumi.c1 as u8;
                }
            }
        }
    }
}

fn get_color() -> u16 {
    random_u16(4) + 232 + ((random_u16(4) + 232) << 8)
}
