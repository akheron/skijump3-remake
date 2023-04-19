use crate::graph::GraphModule;
use crate::help::txt;
use crate::lang::LangModule;
use crate::maki::{MakiModule, SIVUJA};
use crate::pcx::PcxModule;
use crate::rs_util::{parse_line, random, read_line};
use crate::sdlport::SDLPortModule;
use chrono::{Datelike, Timelike};
use std::cell::{Cell, RefCell};
use std::ffi::OsString;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::unix::prelude::OsStringExt;
use std::path::Path;
use std::str::from_utf8;

pub const NUM_PL: usize = 75; //{ montako pelaajaa on ylip��t��n }
pub const NUM_TEAMS: usize = 15;
pub const MAX_OWN_PL: i32 = 10; //{ montako omaa pelaajaa voi olla }
pub const NUM_WC_HILLS: i32 = 20; //{ montako m�ke� world cupissa }
pub const MAX_EXTRA_HILLS: i32 = 1000; //{ montako extra m�ke� voi olla. check!!! }
pub const MAX_CUSTOMS: i32 = 200; //{ montako custom hill filea .sjc voi olla }

pub const HEX_CH: [u8; 16] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'A', b'B', b'C', b'D', b'E', b'F',
];

pub const WC_POINTS: [u8; 30] = [
    100, 80, 60, 50, 45, 40, 36, 32, 29, 26, 24, 22, 20, 18, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7,
    6, 5, 4, 3, 2, 1,
];
pub const TEAM_POINTS: [i32; 8] = [400, 350, 300, 250, 200, 150, 100, 50];

#[derive(Clone, Default)]
pub struct Hiscore {
    pub name: Vec<u8>,
    pub pos: u8,
    pub score: i32,
    pub time: Vec<u8>,
}

#[derive(Clone, Copy, Default)]
pub struct Stat {
    pub comp_pos: u8,
    pub wc_pos: u8,
    pub comp_pts: i32,
    pub round1_pos: u8,
    pub round_pts: [i32; 3],
    pub round_len: [i32; 3],
    pub reason: [u8; 3],
}

#[derive(Clone, Default)]
pub struct Hill {
    pub name: Vec<u8>,
    pub author: Vec<u8>,
    pub kr: i32,
    pub fr_index: Vec<u8>,
    pub bk_index: Vec<u8>,
    pub bk_bright: u8,
    pub bk_mirror: u8,
    pub vx_final: u8,
    pub pk: f32,
    pub pl_save: f32,
    pub profile: i32,
    pub hr_name: Vec<u8>,
    pub hr_len: i32,
    pub hr_time: Vec<u8>,
}

#[derive(Clone, Default)]
pub struct Hillinfo {
    pub name: Vec<u8>,
    pub kr: i32,
    pub hrname: Vec<u8>,
    pub hrlen: i32,
    pub hrtime: Vec<u8>,
}

#[derive(Clone, Copy, Default)]
pub struct Time {
    hour: u16,
    minute: u16,
    second: u16,
    sec100: u16,
}

impl Time {
    pub fn now() -> Self {
        let now = chrono::Local::now();
        Time {
            hour: now.hour() as u16,
            minute: now.minute() as u16,
            second: now.second() as u16,
            sec100: (now.nanosecond() / 10_000_000) as u16,
        }
    }
}

pub struct UnitModule<'g, 'l, 'm, 'p, 's, 'si> {
    g: &'g GraphModule<'m, 'p, 's, 'si>,
    l: &'l LangModule,
    m: &'m MakiModule,
    p: &'p PcxModule<'m, 's, 'si>,
    s: &'s SDLPortModule<'si>,

    pub num_hills: Cell<u8>,
    pub vcode: Cell<u8>,
    pub num_extra_hills: Cell<u16>,

    hd: RefCell<Vec<Hillinfo>>,
}

impl<'g, 'h, 'l, 'm, 'p, 's, 'si> UnitModule<'g, 'l, 'm, 'p, 's, 'si> {
    pub fn new(
        g: &'g GraphModule<'m, 'p, 's, 'si>,
        l: &'l LangModule,
        m: &'m MakiModule,
        p: &'p PcxModule<'m, 's, 'si>,
        s: &'s SDLPortModule<'si>,
    ) -> Self {
        UnitModule {
            g,
            l,
            m,
            p,
            s,
            num_hills: Cell::new(0),
            vcode: Cell::new(0),
            num_extra_hills: Cell::new(0),
            hd: RefCell::new(Vec::from_iter(
                (0..NUM_WC_HILLS + MAX_EXTRA_HILLS).map(|_| Hillinfo::default()),
            )),
        }
    }

    fn givech(&self, xx: i32, yy: i32, bkcolor: u8) {
        let mut run: i32 = 0;
        loop {
            run += 1;
            if run > 20 {
                run = 0;
            }
            let col: u8 = if run > 10 { bkcolor } else { 240 };
            self.g.fill_box(
                xx as u16,
                (yy + 6) as u16,
                (xx + 4) as u16,
                (yy + 6) as u16,
                col,
            );
            self.g.draw_screen();
            if self.s.key_pressed() {
                break;
            }
        }

        self.s.wait_for_key_press();

        self.g.fill_box(
            xx as u16,
            (yy + 6) as u16,
            (xx + 4) as u16,
            (yy + 6) as u16,
            bkcolor,
        );
        self.g.write_font(xx, yy, &[self.s.ch2.get()]);
    }

    fn getch(&self, xx: i32, yy: i32, bkcolor: u8) {
        self.g.fill_box(
            (xx - 2) as u16,
            (yy - 2) as u16,
            (xx + 6) as u16,
            (yy + 8) as u16,
            bkcolor,
        );

        self.givech(xx, yy, bkcolor);

        self.g.draw_screen();
    }

    pub fn load_hill(&self, keula_x: &mut i32, nytmaki: i32, act_hill: &Hill) -> u8 {
        let mut res: u8 = 0;
        self.p.lataa_pcx("LOAD.PCX", 320 * 200, 0, 0);
        self.p.siirra_standardi_paletti();
        self.p.aseta_paletti();

        self.g.write_video();

        //{          FillBox(0,0,319,199,242); }

        //{          NewScreen(5,0); }

        /*{          FontColor(240);
        MuutaLogo(2);
        WriteFont(60,90,'LOADING HILL...');
        DrawAnim(148,90,62);
        WriteFont(185,99,'...PLEASE WAIT');
        AsetaPaletti;   }*/

        let mut str1 = txt(act_hill.kr);

        self.g.font_color(246); //{ 205 }
                                //{          Writefont(160-fontlen(str1) div 2,97,str1); }
        self.g.e_write_font(311, 107, &str1);
        self.g.font_color(240);
        self.g.e_write_font(
            311 - self.g.font_len(&str1),
            107,
            &[&act_hill.name as &[u8], b" K"].concat(),
        );

        self.g.draw_screen();

        self.p.lataa_pcx(
            &["FRONT", from_utf8(&act_hill.fr_index).unwrap(), ".PCX"].concat(),
            1024 * 512,
            0,
            0,
        );

        self.p.tallenna_alkuosa(1);

        self.m.laske_linjat(keula_x, act_hill.kr, act_hill.pk);

        let mut l: i32 = 0;

        for temp in 0..=1023 {
            l += self.m.profiili(temp) * (temp % 13 + self.m.profiili(temp) % 11) % 13313;
        }
        l -= 1500000;

        self.p.lataa_pcx(
            &["BACK", from_utf8(&act_hill.bk_index).unwrap(), ".PCX"].concat(),
            1024 * 400,
            SIVUJA,
            act_hill.bk_mirror,
        );

        self.p.takaisin_alkuosa(1);
        if l != act_hill.profile {
            //{ profiilichekkaus }
            /*
               alertbox;

               str1:='RUN THE HILL MAKER AGAIN.';

               if (nytmaki<=NumWCHills) then
                begin
                 res:=1;
                 str1:='EXITING CUP.';
                end;

               writefont(80,90,'THE PROFILE OF HILL #'+acthill.frindex);
               writefont(80,102,'HAS BEEN CHANGED! NOT GOOD.');
               writefont(80,114,str1);

               writefont(80,130,'PRESS A KEY...');
               drawscreen;

               waitforkey;
            */
            panic!(
                "The profile of hill #{} has been changed! Not good.",
                from_utf8(&act_hill.fr_index).unwrap()
            );
        }

        self.p.savyta_paletti(1, act_hill.bk_bright);
        res
    }

    pub fn make_menu(
        &self,
        x: i32,
        y: i32,
        length: i32,
        height: i32,
        items: i32,
        index: i32,
        bgcolor: u8,
        mut phase: u8,
        tab: u8,
    ) -> i32 {
        //{ phase: 0 - DEL ei k�y, 1 - DEL k�y, 2 - 2 columns! DEL k�y }
        //{        3 - kaikki putkeen, 4 - ei fillarea, 5 - old players }
        //{        6 - new profiles, 7 - kaikki putkeen & ei fill }
        let mut out = false;
        let mut del = false; //{ delete n�pp�in ei k�yt�ss� }
        let mut thing = 63;
        if phase == 5 {
            thing = 64;
            phase = 1;
        }

        let del = matches!(phase, 1 | 2 | 6);
        let fill = !matches!(phase, 4 | 7);
        let putkeen = matches!(phase, 3 | 7);

        let mut index = index.abs();
        let mut xx = (x - 6) as u16;
        let mut yy = (y - 3 + ((index - 1) * height)) as u16;

        let boxcol = 240;

        while !out {
            self.g
                .box_(xx, yy, xx + length as u16, yy + height as u16, bgcolor);
            if fill {
                self.g
                    .fill_area(xx, yy, xx + length as u16, yy + height as u16, thing);
            }

            self.s.clearchs();
            let oldindex = index;

            let (ch, ch2) = if self.s.key_pressed() {
                self.s.wait_for_key_press()
            } else {
                (0, 0)
            };

            match ch2 {
                72 | 75 => {
                    if index > 1 {
                        index -= 1;
                    } else {
                        index = items + 2;
                    }
                    if index == items + 1 {
                        index -= 1;
                    } //{ skip v�li }
                }
                77 | 80 => {
                    if index < items + 1 {
                        index += 1;
                    } else {
                        index = 1;
                    }
                    if index == items + 1 {
                        index += 1;
                    } //{ skip v�li }
                }
                59..=68 => {
                    index = (ch2 - 58) as i32;
                    out = true;
                }
                _ => {}
            }

            match ch {
                b'0'..=b'9' => {
                    index = ch as i32 - 48;
                }
                b'A'..=b'F' => {
                    if phase != 6 {
                        index = ch as i32 - 55; /*{ me halutaan ett� E on edit }*/
                    }
                }
                b'a'..=b'f' => {
                    index = ch as i32 - 87;
                }
                _ => {}
            }

            if (index <= 0) || (index > items) {
                index = items + 2;
                if putkeen {
                    index = items + 1;
                }
            }

            if ch2 == 71 {
                index = 1;
            }
            if (ch == 27) || (ch2 == 79) {
                index = items + 2;
                if putkeen {
                    index = items + 1;
                }
            }

            if ch2 == 68 {
                //{ F10 }
                out = true;
                index = items + 2;
                if putkeen {
                    index = items + 1;
                }
                if phase == 6 {
                    index = 254;
                }
            }

            if (ch == 9) && (tab > 0) {
                //{ tab }
                out = true;
                if tab < 254 {
                    index = tab as i32;
                }
                if tab == 255 {
                    index = items + 2;
                }
            }

            if (ch == 13) || (ch == b' ') {
                out = true;
            }

            xx = (x - 6) as u16;
            yy = (y - 3 + ((index - 1) * height)) as u16;

            if (phase == 6) && ((ch == 13) || (ch == b'E' || ch == b'e') || (ch == 9)) {
                index = tab as i32;
                out = true;
            }

            self.g
                .box_(xx, yy, xx + length as u16, yy + height as u16, boxcol);
            self.g.draw_screen();

            if (ch2 == 83) && (del) {
                out = true;
                index = -index;
            } //{ delete! }

            if (phase == 6) && (index != oldindex) {
                out = true;
            }
        }

        self.g
            .box_(xx, yy, xx + length as u16, yy + height as u16, bgcolor);
        if fill {
            self.g
                .fill_area(xx, yy, xx + length as u16, yy + height as u16, thing);
        }

        if phase != 6 {
            self.g.draw_screen();
            if index > items {
                index = 0;
            } //{ exit }
        }

        self.s.clearchs();
        index
    }

    pub fn do_coach_corner(&self, height: i32, kulmalaskuri: i32, grade: u8, ponn: u8, style: u8) {
        let mut cstr = Vec::new();
        let index: i32 = 360 + style as i32 * 40;

        cstr.push(self.l.lstr(
            (index
                + match kulmalaskuri {
                    0..=49 => 2,
                    50..=61 => 3,
                    62..=200 => 4,
                    _ => panic!("kulmalaskuri out of range"),
                }) as u32,
        ));

        cstr.push(self.l.lstr(
            (index
                + if grade < 10 {
                    match grade {
                        1 => 5,
                        2 => 6,
                        3 => 7,
                        _ => panic!("grade out of range"),
                    }
                } else {
                    match grade / 10 {
                        1..=5 => 10,
                        6..=8 => 11,
                        9 => 12,
                        10 => 13,
                        11 => 14,
                        12..=20 => 15,
                        _ => panic!("grade out of range"),
                    }
                }) as u32,
        ));

        cstr.push(self.l.lstr(
            (index
                + match ponn {
                    0..=5 => 18,
                    6..=9 => 19,
                    10..=12 => 20,
                    13..=15 => 21,
                    16 => 22,
                    17..=19 => 23,
                    20..=23 => 24,
                    24..=50 => 25,
                    _ => panic!("ponn out of range"),
                }) as u32,
        ));

        cstr.push(self.l.lstr(
            (index
                + match height {
                    0..=49 => 28,
                    50..=55 => 29,
                    56..=60 => 30,
                    61..=64 => 31,
                    65..=70 => 32,
                    71..=90 => 33,
                    91..=200 => 34,
                    _ => panic!("height out of range"),
                }) as u32,
        ));

        //{ jos kupat n�es }
        if grade < 10 {
            cstr[0] = cstr[1];
        }
        if grade == 1 {
            cstr[3] = self.l.lstr((index + 35) as u32);
        }

        cstr[1] = cstr[random(2) as usize];
        cstr[2] = cstr[random(2) as usize + 2];

        let joined = [cstr[1], b"*", cstr[2]].concat();
        cstr[1] = &joined;

        self.g.font_color(252);

        self.g.draw_anim(3, 150, 65);
        self.g.write_font(12, 150, self.l.lstr(400));

        let mut index = 1;
        let mut x = 12;
        let mut y = 160;

        self.g.write_font(x, y, b"\"");

        let mut count = 30;
        x = 18;
        y = 152;

        let mut wstr = Vec::new();

        for temp in 1..=cstr[1].len() {
            wstr.push(cstr[1][temp - 1]);
            if wstr[index - 1] == b'*' {
                wstr[index - 1] = b' ';
            }
            index += 1;

            if (index > count && cstr[1][temp - 1] == b' ')
                || (cstr[1][temp - 1] == b'*' && index > count / 2)
            {
                if y < 190 {
                    y += 8;
                }
                x = 18;
                if cstr[1][temp - 1] == b'*' {
                    wstr.pop();
                }

                self.g.write_font(x, y, &wstr);
                x += self.g.font_len(&wstr);

                wstr.clear();
                index = 1;
            }
        }

        if index > 1 {
            if wstr.len() < 2 {
                self.g.write_font(x, y, &[&wstr as &[u8], b"\""].concat());
            } else {
                x = 18;
                if y < 192 {
                    y += 8;
                }
                self.g.write_font(x, y, &[&wstr as &[u8], b"\""].concat());
            }
        }
    }

    pub fn hrname(&self, nytmaki: i32) -> Vec<u8> {
        self.hd.borrow()[nytmaki as usize].hrname.to_vec()
    }
    pub fn hrlen(&self, nytmaki: i32) -> i32 {
        self.hd.borrow()[nytmaki as usize].hrlen
    }
    pub fn hrtime(&self, nytmaki: i32) -> Vec<u8> {
        self.hd.borrow()[nytmaki as usize].hrtime.to_vec()
    }
    pub fn set_hrinfo(
        &self,
        nytmaki: i32,
        name: impl Into<Vec<u8>>,
        len: i32,
        time: impl Into<Vec<u8>>,
    ) {
        let mut hd = self.hd.borrow_mut();
        hd[nytmaki as usize].hrname = name.into();
        hd[nytmaki as usize].hrlen = len;
        hd[nytmaki as usize].hrtime = time.into();
    }
    pub fn hillkr(&self, nytmaki: i32) -> i32 {
        if nytmaki > 0 && nytmaki <= NUM_WC_HILLS + self.num_extra_hills.get() as i32 {
            self.hd.borrow()[nytmaki as usize].kr
        } else {
            0
        }
    }
    pub fn hillname(&self, nytmaki: i32) -> Vec<u8> {
        if (nytmaki < 0) || (nytmaki > NUM_WC_HILLS + self.num_extra_hills.get() as i32) {
            b"Unknown".to_vec()
        } else if nytmaki == 0 {
            self.l.lstr(155).to_vec()
        } else {
            self.hd.borrow()[nytmaki as usize].name.clone()
        }
    }

    #[allow(dead_code)]
    pub fn new_unreg_text(&self) {
        self.g.fill_box(130, 120, 310, 120, 9);
        self.g.font_color(240);
        self.g.write_font(150, 130, self.l.lstr(38));
        self.g.write_font(150, 140, self.l.lstr(39));

        let temp = (((rand::random::<f64>() * 3.0) as i32 * 2) + 41) as u16;
        self.g.font_color(246);

        self.g.write_font(150, 155, self.l.lstr(temp as u32));
        self.g.write_font(150, 165, self.l.lstr(temp as u32 + 1));
        self.g.font_color(240);
        self.g.write_font(150, 180, self.l.lstr(40));
        self.g.draw_screen();
    }
    pub fn main_menu_text(&self, phase: u8, version: &[u8]) {
        // var x,y, temp, num : integer;

        let x = 11;
        self.g.font_color(246);

        self.g
            .fill_box(11, 80, 11 + self.g.font_len(self.l.lstr(17)) as u16, 85, 8);
        //{  fillarea(11,80,11+fontlen(lstr(17]),85,63); }

        self.g.write_font(x, 80, self.l.lstr(17 + phase as u32));

        self.g.font_color(240);

        if phase == 1 {
            self.g.fill_box(1, 94, 116, 199, 8);
            //{   FillArea(2,51,149,179,63); }

            for temp in 1..=7 {
                let mut y = 86 + temp * 12;
                let mut num = temp;
                if temp == 7 {
                    y += 12;
                    num = 0;
                }
                // {$IFNDEF REG}
                //      if (temp=4) then fontcolor(241);
                // {$ENDIF}
                if temp == 5 {
                    self.g.font_color(240);
                }
                self.g.write_font(
                    x,
                    y,
                    &[
                        format!("{num} - ").as_bytes(),
                        self.l.lstr(temp as u32 + 26),
                    ]
                    .concat(),
                );
            }
        }

        if phase == 0 {
            for temp in 1..=7 {
                let mut y = 86 + temp * 12;
                let mut num = temp;
                if temp == 7 {
                    y += 12;
                    num = 0;
                }

                self.g.write_font(
                    x,
                    y,
                    &[
                        format!("{num} - ").as_bytes(),
                        self.l.lstr(temp as u32 + 19),
                    ]
                    .concat(),
                );
            }

            //{   fontcolor(241); }

            /*
            {   fillbox(268,6,308,12,104); }
               fillbox(268,6,307,11,164);
            {   fillbox(245,18,308,24,134); }
               fillbox(245,18,307,23,165);
               fillbox(245,30,269,35,166);
            */
        }

        self.g.e_write_font(308, 6, b"SKI JUMP");
        self.g.e_write_font(308, 18, b"INTERNATIONAL");
        self.g.write_font(245, 30, &[b"v", version].concat());
    }

    pub fn new_reg_text(&self, regname: &str, _regnumber: &str) {
        self.g.fill_box(128, 155, 312, 155, 9);
        self.g.font_color(240);
        self.g
            .write_font(132, 163, &[self.l.lstr(35), b" ", self.l.lstr(36)].concat());
        self.g.font_color(246);
        self.g.fill_box(132, 175, 308, 196, 248);
        self.g.write_font(140, 177, regname.as_bytes());
        self.g.draw_screen();
    }

    fn check_file(&self, phase: i32, hill: &mut Hill, str1: &str) {
        let str2: Vec<u8> = if phase == 1 {
            [b"BACK", hill.bk_index.as_slice(), b".PCX"].concat()
        } else {
            [b"FRONT", hill.fr_index.as_slice(), b".PCX"].concat()
        };

        let filename = OsString::from_vec(str2.clone());
        if !Path::new(&filename).exists() {
            println!(
                "Error #345A: File {} does not exist,",
                String::from_utf8_lossy(&str2)
            );
            println!("even though it's mentioned in the {str1} file.");
            println!("Using FRONT1.PCX and BACK1.PCX.");
            println!();
            println!("Press a key...");

            hill.bk_index = b"1".to_vec();
            hill.fr_index = b"1".to_vec();

            self.s.wait_for_key_press();
            /*
               AsetaMoodi($3);
               writeln('Error #345A: File '+filename+' does not exist,');
               writeln('even though it''s mentioned in the ',str1,' file.');
               writeln('Using FRONT1.PCX and BACK1.PCX.');
               writeln;
               writeln('Press a key...');

               hill.bkindex:='1';
               hill.frindex:='1';

               waitforkey;

               AsetaMoodi($13);
              end;
             end;

            */
        }
    }

    fn default_hill(hill: &mut Hill) {
        hill.name = b"Default".to_vec();
        hill.author = b"Unknown".to_vec();
        hill.fr_index = b"1".to_vec(); //{ front index eli etukuvan j�rj n:o }
        hill.bk_index = b"0".to_vec(); //{ back index }
        hill.bk_bright = 100; //{ taustan kirkkaus, 100 = normaali }
        hill.bk_mirror = 0; //{ peilataanko tausta? 0 - ei, 1 - joo }

        hill.kr = 120;
        hill.pk = 1.0;
        hill.pl_save = 0.321;
        hill.vx_final = 140;

        hill.hr_name = b"Default Jumper\xff".to_vec();
        hill.hr_len = 0;
        hill.hr_time = b"Oct 1 2000 0:00".to_vec();
    }

    // 0 - löytyi, 1 - ei lytynyt
    fn findstart(&self, f1: &mut BufReader<File>, nytmaki: i32) -> u8 {
        let mut out = false;
        let mut result = 1;
        let mut str1: Vec<u8> = Vec::new();

        while !out {
            str1.clear();
            let Ok(n) = f1.read_until(0xA, &mut str1) else {
                result = 1;
                break;
            };
            if n == 0 {
                out = true;
            } else if (str1[0] == b'*') && (nytmaki == 0 || str1[1] == (nytmaki as u8 + b'A' - 1)) {
                out = true;
                result = 0;
            }
        }
        result
    }

    pub fn hillfile(&self, _nyt: i32) -> Vec<u8> {
        return b"HILLBASE.SKI".to_vec();
        /*
        function hillfile(nyt:integer):string;
        var str1 : string;
            temp : integer;
            f2 : text;
        begin

         hillfile:='HILLBASE.SKI';

         if (nyt<=NumExtraHills) then
          begin
           assign(f2,'MOREHILL.SKI');
           {$I-}
           reset(f2);
           {$I+}
           FileOK(IOResult,'MOREHILL.SKI');

           str1:='ERROR.SJH';

           readln(f2); { NumExtraHills pois }

           for temp:=1 to nyt do
            readln(f2,str1);

           close(f2);

           hillfile:=str1;

          end;
        end;
         */
    }

    pub fn load_info(&self, nytmaki: i32, hill: &mut Hill) {
        Self::default_hill(hill);

        let str1 = if nytmaki <= NUM_WC_HILLS {
            "HILLBASE.SKI"
        } else {
            //str1:=hillfile(nytmaki-NumWCHills)+'.SJH';
            unimplemented!("extra hills");
        };
        let temp = if nytmaki > NUM_WC_HILLS { 0 } else { nytmaki };

        let mut f1 = BufReader::new(File::open(str1).unwrap());
        if self.findstart(&mut f1, temp) == 0 {
            hill.name = read_line(&mut f1).unwrap();
            hill.kr = parse_line(&mut f1).unwrap();
            hill.fr_index = read_line(&mut f1).unwrap();
            hill.bk_index = read_line(&mut f1).unwrap();
            hill.bk_bright = parse_line(&mut f1).unwrap();
            hill.bk_mirror = parse_line(&mut f1).unwrap();
            hill.vx_final = parse_line(&mut f1).unwrap();
            let b: u8 = parse_line(&mut f1).unwrap();
            hill.pk = (b as f32) / 100.0;
            let a: i32 = parse_line(&mut f1).unwrap();
            hill.pl_save = (a as f32) / 10000.0;
            hill.author = read_line(&mut f1).unwrap();
            let _l2: i32 = parse_line(&mut f1).unwrap(); //{ checksum }
            hill.profile = parse_line(&mut f1).unwrap();

            // TODO: checksum protection
            /*
            inc(l1,valuestr(hill.name,temp+1));
            inc(l1,longint(hill.kr)*77);
            c:=num(hill.frindex); { t�m� yhteensopivuudenkin takia }
             if (c<0) then c:=valuestr(hill.frindex,temp+1); { jos indexiss� kirjaimia (v3.10) }
            inc(l1,longint(c)*272);
            c:=num(hill.bkindex); { t�m� my�s }
             if (c<0) then c:=valuestr(hill.bkindex,temp+1);
            inc(l1,longint(c)*373);
            inc(l1,longint(hill.bkbright)*313);
            inc(l1,longint(hill.bkmirror)*5775);
            inc(l1,longint(hill.vxfinal)*333);
            inc(l1,longint(b) mod 55555);
            inc(l1,longint(a) mod 11111);
            inc(l1,valuestr(hill.author,temp+2));

            l1:=l1 xor 787371;
            */
            if nytmaki > NUM_WC_HILLS {
                //{ voi olla ettei n�it� oo - v3.00}
                let hr_name = read_line(&mut f1);
                let hr_len = parse_line::<i32>(&mut f1);
                let hr_time = read_line(&mut f1);
                let l3 = parse_line::<i32>(&mut f1);
                match (hr_name, hr_len, hr_time, l3) {
                    (Ok(hr_name), Ok(hr_len), Ok(hr_time), Ok(l3)) if l3 != 0 => {
                        hill.hr_name = hr_name;
                        hill.hr_len = hr_len;
                        hill.hr_time = hr_time;
                        /* TODO: checksum protection
                        inc(l4,valuestr(hill.hrname,13));
                        inc(l4,longint(hill.hrlen)*3553);
                        if (l3<>l4) then inc(l1);
                        */
                    }
                    _ => {
                        hill.hr_name = b"Nobody\xff".to_vec();
                        hill.hr_len = 0;
                        hill.hr_time = b"Oct 1 2000 0:00".to_vec();
                    }
                }
            }
        }
        drop(f1);

        self.check_file(0, hill, &str1);
        self.check_file(1, hill, &str1);

        /* TODO: checksum protection
        if (l1<>l2) then { joku ei t�sm��! }
         begin
          if (nytmaki<NumWCHills) then
           begin
            AsetaMoodi($3);
            writeln('Error #324A: Something''s wrong in the ',str1,' file.  ');
            writeln('Maybe it''s been edited or something.  That won''t do.  Exiting.');
            Waitforkey;
            Halt;
           end else
            begin
             sj3help.beep(1);

             AsetaMoodi($3);

             writeln('Warning #56B: Something doesn''t add up in the '+str1+' file. ');
             writeln('Continuing with a default hill.');
             writeln;
             writeln('Press a key...');

             waitforkey;
             defaulthill(hill);

             AsetaMoodi($13);

            end;
         end;
        */
    }

    pub fn check_extra_hills(&self) {
        // TODO
    }

    //{ t�ll� ladataan mnimet ja kri -taulut kuntoon }
    pub fn load_hill_info(&self) {
        // TODO
        /*
        procedure LoadHillInfo;  { t�ll� ladataan mnimet ja kri -taulut kuntoon }
        var f1 : text;
            temp : byte;
            NowHill : hill_type;
            filename : string;

        begin
        */
        {
            let mut f1 = BufReader::new(File::open("MOREHILL.SKI").unwrap());
            self.num_extra_hills.set(parse_line(&mut f1).unwrap());
        }

        let mut hd = self.hd.borrow_mut();
        for temp in 1..=NUM_WC_HILLS + self.num_extra_hills.get() as i32 {
            let mut now_hill = Hill::default();
            self.load_info(temp, &mut now_hill);
            hd[temp as usize].name = now_hill.name.clone();
            hd[temp as usize].kr = now_hill.kr;
        }

        self.num_hills
            .set((NUM_WC_HILLS + self.num_extra_hills.get() as i32) as u8);
    }

    pub fn read_extras(&self) {
        // TODO
    }

    pub fn keyname(&self, chw: u16) -> Vec<u8> {
        let a = (chw >> 8) as u8;
        let b = (chw & 0xff) as u8;
        match a {
            0 => match b {
                59..=67 => [b"F" as &[u8], &txt(b as i32 - 58)].concat(),
                71 => b"HOME".to_vec(),
                72 => self.l.lstr(280).to_vec(),
                73 => b"PAGE UP".to_vec(),
                75 => self.l.lstr(281).to_vec(),
                76 => b"NP 5".to_vec(),
                77 => self.l.lstr(282).to_vec(),
                79 => b"END".to_vec(),
                80 => self.l.lstr(283).to_vec(),
                81 => b"PAGE DOWN".to_vec(),
                82 => b"INSERT".to_vec(),
                83 => b"DELETE".to_vec(),
                _ => b"NULL".to_vec(),
            },
            8 => b"BACKSPACE".to_vec(),
            9 => b"TAB".to_vec(),
            b' ' => b"SPACE".to_vec(),
            b'.' => b".".to_vec(),
            b',' => b",".to_vec(),
            b'-' => b"-".to_vec(),
            b'+' => b"+".to_vec(),
            b'/' => b"/".to_vec(),
            b'*' => b"*".to_vec(),
            48..=57 => txt(a as i32 - 48),
            65..=90 => vec![a],
            97..=122 => vec![a.to_ascii_uppercase()],
            _ => b"NULL".to_vec(),
        }
    }

    //{ 0-main menu, 1-world cup }
    pub fn quitting(&self, phase: u8) -> u8 {
        let str1 = if phase == 1 {
            self.l.lstr(245)
        } else {
            self.l.lrstr(251, 253)
        };
        let str2 = self.l.lrstr(256, 258);

        self.g.alert_box();

        let mut tempb = self.g.font_len(&str2) + 4;

        self.g.font_color(241);
        self.g.write_font(
            70 + tempb,
            110,
            &[
                b"(",
                &self.l.lstr(6)[0..1],
                b"/",
                &self.l.lstr(7)[0..1],
                b")",
            ]
            .concat(),
        );

        tempb += 25;

        self.g.font_color(246);

        self.g.write_font(70, 90, str1);
        self.g.write_font(70, 110, str2);

        self.getch(70 + tempb, 110, 243);

        tempb = 1;
        if self.s.ch.get().to_ascii_uppercase() == self.l.lstr(6)[0] {
            tempb = 0;
        }
        self.s.clearchs();

        tempb as u8
    }

    pub fn wait_for_key3(&self, xx: i32, yy: i32) -> bool {
        while self.s.key_pressed() {
            self.s.wait_for_key_press();
        }

        self.g.font_color(240);

        self.g.e_write_font(xx, yy, &self.l.lstr(15));

        self.getch(xx + 1, yy, 243);

        if self.s.ch.get() == 0 && self.s.ch2.get() == 68 {
            self.s.ch.set(27);
            true
        } else {
            false
        }
    }
}

pub fn uncrypt(mut str0: Vec<u8>, jarj: i32) -> i32 {
    // All the "- 1" are because the original code uses 1-based string indexing

    let mut high = str0[5 - 1];
    str0.remove(5 - 1);
    let chk1 = str0[2 - 1];
    str0.remove(2 - 1);
    let chk2 = str0[2 - 1];
    str0.remove(2 - 1);

    let mut str1: Vec<u8> = Vec::new();

    for index in (1..=5).rev() {
        str1.push(str0[index - 1] - 21);
    }

    let mut luku1: i32 = from_utf8(&str1).unwrap().parse().unwrap();

    if high > 74 {
        //{ ei pelkk� satunnaiskirjain }
        luku1 += 100000 * (high - 75) as i32;
    }

    str1[1 - 1] = 68 + 2 * ((luku1 % 7) ^ 1) as u8;
    if str1[1 - 1] != chk1 {
        luku1 = 0;
    }
    str1[1 - 1] = 65 + ((jarj ^ 33) % 19) as u8;
    if str1[1 - 1] != chk2 {
        luku1 = 0;
    }

    luku1
}

pub fn valuestr(str0: &[u8], arvo: i32) -> u16 {
    let mut word1: u16 = 0;

    for index in 1..=str0.len() {
        word1 = word1.wrapping_add(
            (str0[index - 1] as i32).wrapping_mul(((index as i32) % 5).wrapping_add(41)) as u16,
        );
    }

    word1
        .wrapping_mul((arvo % 7 + 1) as u16)
        .wrapping_add(arvo as u16)
}

pub fn loadgoal(num: i32) -> i32 {
    let mut value: i32 = 0;

    if num > 0 && num <= NUM_WC_HILLS {
        let mut f2 = File::open("GOALS.SKI").unwrap();
        let mut b = BufReader::new(f2).lines();
        for _ in 1..=num {
            let line = b.next().unwrap().unwrap();
            value = line.parse().unwrap();
        }
    }
    value
}

pub fn kword(ch1: u8, ch2: u8) -> u16 {
    ((ch1 as u16) << 8) | ch2 as u16
}

pub fn defaultkeys(k: &mut [u16]) {
    k[1] = 72;
    k[2] = 77;
    k[3] = 75;
    k[4] = b'T' as u16 * 256 + 20;
    k[5] = b'R' as u16 * 256 + 19;
}

pub fn injured() -> u8 {
    match random(5) {
        0 | 1 | 2 => 0,
        3 => 1,
        4 => random(4) as u8 + 1,
        _ => unreachable!(),
    }
}

pub fn dayandtime_now() -> Vec<u8> {
    static DAYS: [&[u8]; 7] = [b"Sun", b"Mon", b"Tue", b"Wed", b"Thu", b"Fri", b"Sat"];
    static MONTHS: [&[u8]; 12] = [
        b"Jan", b"Feb", b"Mar", b"Apr", b"May", b"Jun", b"Jul", b"Aug", b"Sep", b"Oct", b"Nov",
        b"Dec",
    ];

    let now = chrono::Local::now();
    return [
        DAYS[now.weekday().num_days_from_sunday() as usize],
        b" ",
        if now.day() < 10 {
            b" " as &[u8]
        } else {
            b"" as &[u8]
        },
        &txt(now.day() as i32),
        b" ",
        MONTHS[now.month() as usize],
        b" ",
        &txt(now.year()),
        b" ",
        if now.hour() < 10 {
            b" " as &[u8]
        } else {
            b"" as &[u8]
        },
        &txt(now.hour() as i32),
        b":",
        if now.minute() < 10 {
            b"0" as &[u8]
        } else {
            b"" as &[u8]
        },
        &txt(now.minute() as i32),
    ]
    .concat();
}

pub fn makeletter(temp: i32) -> &'static [u8] {
    match temp {
        1 => b"E", // Early Takeoff
        2 => b"L", // Slow Landing
        3 => b"F", // Fall
        4 => b"D", // Did not show
        5 => b"H", // Hillrecord
        6 => b"t", // Telemark-landing
        7 => b"r", // Two Footed landing
        _ => b" ",
    }
}
