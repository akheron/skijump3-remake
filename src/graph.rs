use crate::maki::MakiModule;
use crate::sdlport::SDLPortModule;
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;

pub struct GraphModule<'m, 's, 'si> {
    m: &'m MakiModule,
    s: &'s SDLPortModule<'si>,

    anim: RefCell<Vec<Vec<u8>>>,
    anim_p: [[u8; 2]; 200],
    num_anim: u8,
}

fn read_byte(file: &mut File) -> u8 {
    let mut buf = [0; 1];
    file.read_exact(&mut buf).unwrap();
    buf[0]
}

impl<'m, 's, 'si> GraphModule<'m, 's, 'si> {
    pub fn new(m: &'m MakiModule, s: &'s SDLPortModule<'si>) -> Self {
        GraphModule {
            m,
            s,
            anim: RefCell::new(vec![Vec::new(); 200]),
            anim_p: [[0; 2]; 200],
            num_anim: 0,
        }
    }

    pub fn draw_screen(&self) {
        self.s.wait_raster();
        self.s.render(self.m.video.borrow().as_ref());
    }
    pub fn draw_hill_screen(&self) {
        self.m.tulosta();
        self.draw_screen();
    }
    pub fn close_graph(&self) {
        unimplemented!()
    }

    pub fn sprite(&self, sprite_data: &[u8], x: u16, y: u16) {
        let mut video = self.m.video.borrow_mut();

        let ysize: u16 = sprite_data[2] as u16 + ((sprite_data[3] as u16) << 8);
        let xsize: u16 = ((sprite_data.len() as u16) - 4) / ysize;
        let mut offset = 4;

        for yindex in 0..=(ysize - 1) {
            for xindex in 0..=(xsize - 1) {
                if (sprite_data[offset] != 0) && (x + xindex < 320) {
                    video[(x + xindex + (y + yindex) * 320) as usize] = sprite_data[offset];
                }
                offset += 1;
            }
        }
    }

    pub fn draw_anim(&self, x: i32, y: i32, num: u8) {
        let anim = self.anim.borrow();

        let x = x - self.anim_p[num as usize][0] as i32;
        let y = y - self.anim_p[num as usize][1] as i32;
        let mut ysize: i32 = 0;

        if (num > 0) && (num <= self.num_anim) {
            ysize = anim[num as usize][2] as i32 + ((anim[num as usize][3] as i32) << 8);

            if (x >= 0) && (y >= 0) && (x < 320) && (y < 200 - ysize) {
                self.sprite(&anim[num as usize], x as u16, y as u16);
            }
        }
    }

    pub fn load_anim(&mut self, filename: &str) {
        let mut anim = self.anim.borrow_mut();
        let mut f1 = File::open(filename).unwrap();

        self.num_anim = 0;

        let mut x = read_byte(&mut f1);
        let mut y = read_byte(&mut f1);

        loop {
            // TODO: it seems that anim[0] is unused
            self.num_anim += 1;

            anim[self.num_anim as usize] = vec![0; x as usize * y as usize + 4];

            anim[self.num_anim as usize][0] = x;
            anim[self.num_anim as usize][1] = 0; //x >> 8;
            anim[self.num_anim as usize][2] = y;
            anim[self.num_anim as usize][3] = 0; //y >> 8;

            for yy in 0..=(y - 1) {
                for xx in 0..=(x - 1) {
                    let mut tempb = read_byte(&mut f1);
                    if tempb == 9 {
                        tempb = 15; // suksen keskityspiste (kai?)
                    }
                    anim[self.num_anim as usize][yy as usize * x as usize + xx as usize + 4] =
                        tempb;
                }
            }

            self.anim_p[self.num_anim as usize][0] = read_byte(&mut f1); // keskittämisplussa x
            self.anim_p[self.num_anim as usize][1] = read_byte(&mut f1); // -"- y

            x = read_byte(&mut f1);
            y = read_byte(&mut f1);

            if self.num_anim == 83 {
                // invert skis!
                for temp in 72..=83 {
                    self.num_anim += 1;
                    let x2: u16 = (anim[temp][0] as u16) + ((anim[temp][1] as u16) << 8);
                    let y2: u16 = (anim[temp][2] as u16) + ((anim[temp][3] as u16) << 8);

                    anim[self.num_anim as usize] = vec![0; (x2 * y2 + 4) as usize];

                    anim[self.num_anim as usize][0] = (x2 & 0xff) as u8;
                    anim[self.num_anim as usize][1] = ((x2 >> 8) & 0xff) as u8;
                    anim[self.num_anim as usize][2] = (y2 & 0xff) as u8;
                    anim[self.num_anim as usize][3] = ((y2 >> 8) & 0xff) as u8;

                    for yy in 0..=(y2 - 1) {
                        for xx in 0..=(x2 - 1) {
                            anim[self.num_anim as usize][(yy * x2 + xx + 4) as usize] =
                                anim[temp][((y2 - yy - 1) * x2 + xx + 4) as usize];
                            self.anim_p[self.num_anim as usize][0] = self.anim_p[temp][0];
                            self.anim_p[self.num_anim as usize][1] =
                                (y2 - 1 - (self.anim_p[temp][1] as u16)) as u8;
                        }
                    }
                }
            }

            if (x == 255) && (y == 255) {
                break;
            }
        }
    }

    pub fn put_pixel(&self, x: i32, y: i32, c: u8) {
        let mut video = self.m.video.borrow_mut();
        if (0..320).contains(&x) && (0..200).contains(&y) {
            video[((y * 320) + x) as usize] = c;
        }
    }

    pub fn get_pixel(&self, x: u16, y: u16) -> u8 {
        unimplemented!()
    }

    pub fn e_write_font(&self, xx: i32, yy: i32, s: &[u8]) {
        self.write_font(xx - self.font_len(s), yy, s);
    }

    pub fn write_font(&self, xx: i32, yy: i32, s: &[u8]) {
        if (xx > 0) && (yy > 0) {
            self.do_font(xx, yy, s, true);
        }
    }

    fn do_font(&self, xx: i32, yy: i32, s: &[u8], draw: bool) -> i32 {
        let mut p = 0; //{ siirtym� }

        // for i in 1..=s.len() {
        //     s[i] = upcase(s[i]);
        // }

        for i in 0..s.len() {
            let mut t = 100;
            let chh = s[i];

            match chh {
                b' ' => {
                    p += 4; //{ vaan space! }
                }
                b'$' => {
                    p += 5; //{ % numeron pituinen space }
                }
                b'0' => {
                    t = 29;
                }
                _ if (b'1'..=b'9').contains(&chh) => {
                    t = chh - 19;
                }
                _ if (b'A'..=b'Z').contains(&chh) => {
                    t = chh - 65;
                }
                _ if (b'a'..=b'z').contains(&chh) => {
                    t = chh - 97;
                }
                0x86 | 0x8f => {
                    t = 26;
                }
                0x84 | 0x8e => {
                    t = 27;
                }
                0x94 | 0x99 => {
                    t = 28;
                }
                b':' => {
                    t = 39;
                }
                b'.' => {
                    t = 40;
                }
                b'?' => {
                    t = 41;
                }
                b'!' => {
                    t = 42;
                }
                b'*' => {
                    t = 43;
                }
                b'-' => {
                    t = 44;
                }
                b'+' => {
                    t = 45;
                }
                b',' => {
                    t = 46;
                }
                b'(' => {
                    t = 47;
                }
                b')' => {
                    t = 48;
                }
                0xab => {
                    t = 49; //{ pieni m }
                }
                b'"' => {
                    t = 50; //{ tuplaheittomerkki }
                }
                b'\'' => {
                    t = 51; //{ yksi heittomerkki }
                }
                b'#' => {
                    t = 52;
                }
                0x9b | 0x9d => {
                    t = 53; //{ norja � eli o ja viiva halki }
                }
                0x81 | 0x9a => {
                    t = 54; //{ �ber y }
                }
                0xe1 => {
                    t = 55; //{ stuit staffel }
                }
                b'/' => {
                    t = 56;
                }
                0x92 | 0x91 => {
                    t = 57; //{ AE:t }
                }
                b'%' => {
                    t = 58;
                }
                _ => {
                    println!("Unknown char: {}", chh);
                }
            }

            if t != 100 {
                t += 1;
                if draw {
                    self.draw_anim(xx + p, yy, t); //{ t+? riippuu muista animeista! }
                }
                let anim = self.anim.borrow();
                p += ((anim[t as usize][0] as u16) + ((anim[t as usize][1] as u16) << 8)) as i32;
            }
        }

        p
    }

    pub fn font_len(&self, s: &[u8]) -> i32 {
        self.do_font(0, 0, s, false)
    }

    pub fn font_color(&self, col: u8) {
        let mut anim = self.anim.borrow_mut();

        for temp in 1..=60 {
            let x: u16 = anim[temp][0] as u16 + ((anim[temp][1] as u16) << 8);
            let y: u16 = anim[temp][2] as u16 + ((anim[temp][3] as u16) << 8);

            for yy in 0..=(y - 1) {
                for xx in 0..=(x - 1) {
                    let temp_b = anim[temp][((yy * x) + xx + 4) as usize];
                    if (temp_b != 242) && (temp_b != 0) {
                        anim[temp][((yy * x) + xx + 4) as usize] = col;
                    }
                }
            }
        }
    }

    pub fn fill_area(&self, x1: u16, y1: u16, x2: u16, y2: u16, thing: i32) {
        let index = 63;
        let sizex = 19;
        let sizey = 13;

        let mut video = self.m.video.borrow_mut();
        let anim = self.anim.borrow();
        for temp1 in y1..=y2 {
            for temp2 in x1..=x2 {
                let scr = (temp1 * 320) + temp2;

                let col = video[scr as usize];
                let mut new = col;

                let mut ax = (scr % 320) % sizex;
                let mut ay = (scr / 320) % sizey;

                if thing == 64 {
                    ax = ((scr % 320) + 2) % sizex;
                    ay = ((scr / 320) + 7) % sizey;
                }

                let count = (ay * sizex) + ax;

                if (new > 242) && (new < 246) && (anim[index][count as usize + 4] != 0) {
                    new += 5;
                }

                video[scr as usize] = new;
            }
        }
    }

    pub fn fill_box(&self, x: u16, y: u16, x2: u16, y2: u16, col: u8) {
        let mut video = self.m.video.borrow_mut();
        for yy in y..=y2 {
            for xx in x..=x2 {
                video[(xx + (yy << 8) + (yy << 6)) as usize] = col;
            }
        }
    }

    pub fn box_(&self, x: u16, y: u16, x2: u16, y2: u16, col: u8) {
        for xx in x..=x2 {
            self.put_pixel(xx as i32, y as i32, col);
            self.put_pixel(xx as i32, y2 as i32, col);
        }

        for yy in y..=y2 {
            self.put_pixel(x as i32, yy as i32, col);
            self.put_pixel(x2 as i32, yy as i32, col);
        }
    }

    pub fn balk(&self, st: u8) {
        unimplemented!()
    }

    pub fn new_screen(&self, style: u8, color: u8) {
        unimplemented!()
    }

    pub fn alert_box(&self) {
        unimplemented!()
    }

    pub fn write_video(&self) {
        let mut video = self.m.video.borrow_mut();
        let graffa = self.m.graffa.borrow();
        let source = &graffa.as_ref()[0..64000];
        video.copy_from_slice(source);
    }
}
