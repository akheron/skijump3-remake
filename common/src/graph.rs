use crate::maki::MakiModule;
use crate::pcx::PcxModule;
use crate::platform::Platform;
use std::cell::{Cell, RefCell};
use std::io::Read;
use std::rc::Rc;

pub struct GraphModule<'m, 'p, 's, P: Platform> {
    m: &'m MakiModule,
    p: &'p PcxModule<'m, 's, P>,
    s: &'s P,

    anim: RefCell<Vec<Vec<u8>>>,
    anim_p: RefCell<[[u8; 2]; 200]>,
    num_anim: Cell<u8>,
}

fn read_byte<R: Read>(mut file: R) -> u8 {
    let mut buf = [0; 1];
    file.read_exact(&mut buf).unwrap();
    buf[0]
}

impl<'m, 'p, 's, 'si, P: Platform> GraphModule<'m, 'p, 's, P> {
    pub fn new(m: &'m MakiModule, p: &'p PcxModule<'m, 's, P>, s: &'s P) -> Self {
        GraphModule {
            m,
            p,
            s,
            anim: RefCell::new(vec![Vec::new(); 200]),
            anim_p: RefCell::new([[0; 2]; 200]),
            num_anim: Cell::new(0),
        }
    }

    pub async fn draw_screen(&self) {
        self.s.render_phase1(self.m.video.borrow().as_ref());
        self.s.render_phase2().await;
    }
    pub async fn draw_hill_screen(&self) {
        self.m.tulosta();
        self.draw_screen().await;
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
        let anim_p = self.anim_p.borrow();
        let num_anim = self.num_anim.get();

        let x = x - anim_p[num as usize][0] as i32;
        let y = y - anim_p[num as usize][1] as i32;
        let mut ysize: i32 = 0;

        if (num > 0) && (num <= num_anim) {
            ysize = anim[num as usize][2] as i32 + ((anim[num as usize][3] as i32) << 8);

            if (x >= 0) && (y >= 0) && (x < 320) && (y < 200 - ysize) {
                self.sprite(&anim[num as usize], x as u16, y as u16);
            }
        }
    }

    pub fn load_anim(&self, filename: &str) {
        let mut anim = self.anim.borrow_mut();
        let mut anim_p = self.anim_p.borrow_mut();
        let mut f1 = self.s.open_file(filename);

        let mut num_anim = 0;

        let mut x = read_byte(&mut f1);
        let mut y = read_byte(&mut f1);

        loop {
            // TODO: it seems that anim[0] is unused
            num_anim += 1;

            anim[num_anim as usize] = vec![0; x as usize * y as usize + 4];

            anim[num_anim as usize][0] = x;
            anim[num_anim as usize][1] = 0; //x >> 8;
            anim[num_anim as usize][2] = y;
            anim[num_anim as usize][3] = 0; //y >> 8;

            for yy in 0..=(y - 1) {
                for xx in 0..=(x - 1) {
                    let mut tempb = read_byte(&mut f1);
                    if tempb == 9 {
                        tempb = 15; // suksen keskityspiste (kai?)
                    }
                    anim[num_anim as usize][yy as usize * x as usize + xx as usize + 4] = tempb;
                }
            }

            anim_p[num_anim as usize][0] = read_byte(&mut f1); // keskittämisplussa x
            anim_p[num_anim as usize][1] = read_byte(&mut f1); // -"- y

            x = read_byte(&mut f1);
            y = read_byte(&mut f1);

            if num_anim == 83 {
                // invert skis!
                for temp in 72..=83 {
                    num_anim += 1;
                    let x2: u16 = (anim[temp][0] as u16) + ((anim[temp][1] as u16) << 8);
                    let y2: u16 = (anim[temp][2] as u16) + ((anim[temp][3] as u16) << 8);

                    anim[num_anim as usize] = vec![0; (x2 * y2 + 4) as usize];

                    anim[num_anim as usize][0] = (x2 & 0xff) as u8;
                    anim[num_anim as usize][1] = ((x2 >> 8) & 0xff) as u8;
                    anim[num_anim as usize][2] = (y2 & 0xff) as u8;
                    anim[num_anim as usize][3] = ((y2 >> 8) & 0xff) as u8;

                    for yy in 0..=(y2 - 1) {
                        for xx in 0..=(x2 - 1) {
                            anim[num_anim as usize][(yy * x2 + xx + 4) as usize] =
                                anim[temp][((y2 - yy - 1) * x2 + xx + 4) as usize];
                            anim_p[num_anim as usize][0] = anim_p[temp][0];
                            anim_p[num_anim as usize][1] =
                                (y2 - 1 - (anim_p[temp][1] as u16)) as u8;
                        }
                    }
                }
            }

            if (x == 255) && (y == 255) {
                break;
            }
        }

        self.num_anim.set(num_anim);
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

    pub fn e_write_font(&self, xx: i32, yy: i32, s: impl AsBytes) {
        let s = s.as_bytes();
        self.write_font(xx - self.font_len(s), yy, s);
    }

    pub fn write_font(&self, xx: i32, yy: i32, s: impl AsBytes) {
        if (xx > 0) && (yy > 0) {
            self.do_font(xx, yy, s.as_bytes(), true);
        }
    }

    fn do_font(&self, xx: i32, yy: i32, s: &[u8], draw: bool) -> i32 {
        let mut p = 0; //{ siirtym� }

        for chh in s {
            let mut t = match chh.to_ascii_uppercase() {
                b' ' => {
                    p += 4; //{ vaan space! }
                    100
                }
                b'$' => {
                    p += 5; //{ % numeron pituinen space }
                    100
                }
                b'0' => 29,
                c @ b'1'..=b'9' => c - 19,
                c @ b'A'..=b'Z' => c - 65,
                0x86 | 0x8f => 26,
                0x84 | 0x8e => 27,
                0x94 | 0x99 => 28,
                b':' => 39,
                b'.' => 40,
                b'?' => 41,
                b'!' => 42,
                b'*' => 43,
                b'-' => 44,
                b'+' => 45,
                b',' => 46,
                b'(' => 47,
                b')' => 48,
                0xab => 49,  //{ pieni m }
                b'"' => 50,  //{ tuplaheittomerkki }
                b'\'' => 51, //{ yksi heittomerkki }
                b'#' => 52,
                0x9b | 0x9d => 53, //{ norja � eli o ja viiva halki }
                0x81 | 0x9a => 54, //{ �ber y }
                0xe1 => 55,        //{ stuit staffel }
                b'/' => 56,
                0x92 | 0x91 => 57, //{ AE:t }
                b'%' => 58,
                _ => 100,
            };

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

    pub fn font_len(&self, s: impl AsBytes) -> i32 {
        self.do_font(0, 0, s.as_bytes(), false)
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

    pub async fn new_screen(&self, style: u8, color: u8) {
        self.fill_box(0, 0, 319, 199, 0);

        match style {
            1 => {
                //{ highscores ym. }
                self.fill_box(0, 0, 319, 18, 245);
                self.fill_box(0, 20, 319, 199, 243);
            }
            2 => {
                //{ valitsem�ki }
                self.fill_box(0, 0, 10, 199, 245);
                self.fill_box(12, 0, 307, 199, 243);
                self.fill_box(309, 0, 319, 199, 245);
            }
            3 => {
                //{ kingofthehill }
                self.fill_box(0, 0, 168, 98, 245);
                self.fill_box(0, 100, 168, 199, 244);
                self.fill_box(170, 0, 319, 199, 243);
            }
            4 => {
                //{ hiscore2 }
                self.fill_box(0, 0, 319, 18, 245);
                self.fill_box(0, 20, 319, 118, 243);
                self.fill_box(0, 120, 319, 138, 245);
                self.fill_box(0, 140, 319, 199, 243);
            }
            5 => {
                self.fill_box(0, 0, 319, 199, 243);
            }
            6 => {
                //{ welcomescreen }
                self.fill_box(0, 0, 50, 199, 245);
                self.fill_box(52, 0, 267, 199, 243);
                self.fill_box(269, 0, 319, 199, 245);
            }
            7 => {
                self.fill_box(0, 0, 158, 199, 243);
                self.fill_box(160, 0, 319, 199, 244);
            }
            _ => {}
        }

        self.fill_area(0, 0, 319, 199, 63);

        //{ LOGO }
        match style {
            0 => {
                self.draw_anim(20, 14, 61);
            }
            1 => {
                self.draw_anim(5, 2, 62);
            }
            2 => {
                self.draw_anim(30, 8, 62);
            }
            4 => {
                self.draw_anim(5, 2, 62);
                self.draw_anim(5, 122, 62);
            }
            6 => {
                self.draw_anim(80, 6, 61);
            }
            _ => {}
        }

        self.p.siirra_standardi_paletti();

        match style {
            0 => {
                self.p.muuta_logo(0); //{ logo sinisen s�vyiksi }
            }
            1 | 2 | 4 => {
                self.p.muuta_menu(3, 0); //{ menu3 harmaaksi }
            }
            6 => {
                self.p.muuta_logo(0);
            }
            _ => {}
        }

        match color {
            1 => {
                self.p.muuta_menu(1, 2); //{ menu1 violetiksi }
            }
            2 => {
                self.p.muuta_menu(1, 4); //{ menu1 ruskeaksi KOTH }
            }
            3 => {
                self.p.muuta_menu(1, 5); //{ menu1 punaiseksi WCRES }
            }
            4 => {
                self.p.muuta_menu(1, 1); //{ menu1 mustaksi 4HILLS }
            }
            5 => {
                self.p.muuta_menu(1, 3); //{ menu1 turkoosiksi? STATS }
            }
            _ => {}
        }

        self.p.aseta_paletti();
        self.draw_screen().await;
    }

    pub fn alert_box(&self) {
        self.fill_box(59, 79, 261, 131, 242);
        self.fill_box(60, 80, 260, 130, 244);

        self.fill_area(60, 80, 260, 130, 63);
    }

    pub fn write_video(&self) {
        let mut video = self.m.video.borrow_mut();
        let graffa = self.m.graffa.borrow();
        let source = &graffa[0..64000];
        video.copy_from_slice(source);
    }

    // Originally in SJ3HELP.PAS
    pub fn nsh(&self, str1: &[u8], maxpituus: i32) -> Vec<u8> {
        let mut temp1: i32;
        let mut temp2 = self.font_len(str1);
        let mut temp3 = str1.len();

        if temp2 > maxpituus {
            let mut str2: Vec<u8> = Vec::new();
            for temp1 in 1..temp3 - 1 {
                if str1[temp1] == b' ' && str1[temp1 + 1] != b' ' {
                    str2 = [&str1[0..1], b".", &str1[temp1..]].concat();
                    if self.font_len(&str2) <= maxpituus {
                        return str2;
                    }
                }
            }

            if str2.is_empty() {
                str2 = str1.to_vec();
            }

            temp1 = self.font_len(&str2);
            temp3 = str2.len();

            if temp1 > maxpituus {
                while self.font_len(&str2) > maxpituus {
                    temp3 -= 1;
                    str2 = [&str2[0..temp3], b"."].concat();
                }
            }
            str2
        } else {
            str1.to_vec()
        }
    }
}

pub trait AsBytes {
    fn as_bytes(&self) -> &[u8];
}

impl AsBytes for &[u8] {
    fn as_bytes(&self) -> &[u8] {
        self
    }
}

impl AsBytes for Rc<Vec<u8>> {
    fn as_bytes(&self) -> &[u8] {
        self
    }
}

impl AsBytes for &Vec<u8> {
    fn as_bytes(&self) -> &[u8] {
        self
    }
}

impl<const N: usize> AsBytes for &[u8; N] {
    fn as_bytes(&self) -> &[u8] {
        *self
    }
}
