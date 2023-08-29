use crate::graph::GraphModule;
use crate::help::{num, txt, txtp};
use crate::lang::LangModule;
use crate::maki::{MakiModule, SIVUJA};
use crate::pcx::PcxModule;
use crate::platform::Platform;
use crate::rs_util::{parse_line, random, read_line};
use chrono::{Datelike, Timelike};
use std::cell::{Cell, RefCell};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::rc::Rc;
use std::str::from_utf8;

pub const NUM_PL: usize = 75; //{ montako pelaajaa on ylip��t��n }
pub const NUM_TEAMS: usize = 15;
pub const MAX_OWN_PL: i32 = 10; //{ montako omaa pelaajaa voi olla }
pub const NUM_WC_HILLS: i32 = 20; //{ montako m�ke� world cupissa }
pub const MAX_EXTRA_HILLS: i32 = 1000; //{ montako extra m�ke� voi olla. check!!! }
pub const MAX_CUSTOMS: i32 = 200; //{ montako custom hill filea .sjc voi olla }

pub const HEX_CH: [&[u8]; 16] = [
    b"0", b"1", b"2", b"3", b"4", b"5", b"6", b"7", b"8", b"9", b"A", b"B", b"C", b"D", b"E", b"F",
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
    pub pk: f64,
    pub pl_save: f64,
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

pub struct UnitModule<'g, 'l, 'm, 'p, 's, P: Platform> {
    g: &'g GraphModule<'m, 'p, 's, P>,
    l: &'l LangModule<'s, P>,
    m: &'m MakiModule,
    p: &'p PcxModule<'m, 's, P>,
    s: &'s P,

    pub num_hills: Cell<u8>,
    pub vcode: Cell<u8>,
    pub num_extra_hills: Cell<u16>,

    hd: RefCell<Vec<Hillinfo>>,
}

impl<'g, 'h, 'l, 'm, 'p, 's, 'si, P: Platform> UnitModule<'g, 'l, 'm, 'p, 's, P> {
    pub fn new(
        g: &'g GraphModule<'m, 'p, 's, P>,
        l: &'l LangModule<'s, P>,
        m: &'m MakiModule,
        p: &'p PcxModule<'m, 's, P>,
        s: &'s P,
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

    pub fn makevcode(&self, version: &[u8], reg: bool) {
        let mut pos: u8 = 2; //{ v3.11 !!! }
        if reg {
            pos = 4;
        }

        if self.vcode.get() & pos == 0 {
            self.vcode.set(self.vcode.get() | pos);
        }
    }

    async fn givech(&self, xx: i32, yy: i32, bkcolor: u8) {
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
            self.g.draw_screen().await;
            if self.s.key_pressed().await {
                break;
            }
        }

        self.s.wait_for_key_press().await;

        self.g.fill_box(
            xx as u16,
            (yy + 6) as u16,
            (xx + 4) as u16,
            (yy + 6) as u16,
            bkcolor,
        );

        if let Some(chr) = char::from_u32(self.s.get_ch() as u32) {
            self.g.write_font(xx, yy, format!("{}", chr).as_bytes());
        }
    }

    pub async fn getch(&self, xx: i32, yy: i32, bkcolor: u8) {
        self.g.fill_box(
            (xx - 2) as u16,
            (yy - 2) as u16,
            (xx + 6) as u16,
            (yy + 8) as u16,
            bkcolor,
        );

        self.givech(xx, yy, bkcolor).await;

        self.g.draw_screen().await;
    }

    pub async fn load_hill(&self, keula_x: &mut i32, nytmaki: i32, act_hill: &Hill) -> u8 {
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

        self.g.draw_screen().await;

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

    pub async fn make_menu(
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

            let (ch, ch2) = if self.s.key_pressed().await {
                self.s.wait_for_key_press().await
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
            self.g.draw_screen().await;

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
            self.g.draw_screen().await;
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
            cstr[0] = cstr[1].clone();
        }
        if grade == 1 {
            cstr[3] = self.l.lstr((index + 35) as u32);
        }

        cstr[1] = cstr[random(2) as usize].clone();
        cstr[2] = cstr[random(2) as usize + 2].clone();

        let joined = [&cstr[1] as &[u8], b"*", &cstr[2]].concat();
        cstr[1] = Rc::new(joined);

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

    pub fn setup_item(&self, index: u8, screen: u8, entries: u8, str1: &[u8]) {
        let mut xx = 25;
        let mut yy = (index as i32 * 10) + 30;

        // {$IFNDEF REG}
        //   case screen of
        //    0 : if (index>4) then good:=false;
        //    1 : if (index=4) then good:=false;
        //    2 : case index of
        //        1,2,6,8,10 : good:=false;
        //        end;
        //   end;
        //
        //  if (not good) then fontcolor(241); { harmaaksi, jos ei ole reg option }
        //
        //  if (good) then fontcolor(246);
        // {$ELSE}
        self.g.font_color(246);
        // {$ENDIF}

        if index == 0 {
            yy = (entries as i32 * 10) + 50;
        }

        let istr = [HEX_CH[index as usize], b"."].concat();
        self.g.e_write_font(xx, yy, &istr);

        self.g.font_color(240);

        xx = 35;

        self.g.write_font(
            xx,
            yy,
            match screen {
                0 => self.l.lstr(195 + index as u32),
                1 => self.l.lstr(203 + index as u32),
                2 => self.l.lstr(211 + index as u32),
                3 => self.l.lstr(225 + index as u32),
                _ => unreachable!(),
            },
        );

        self.g.font_color(246);

        xx = 255;
        self.g.write_font(xx, yy, str1);
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
    pub async fn new_unreg_text(&self) {
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
        self.g.draw_screen().await;
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
                        &self.l.lstr(temp as u32 + 26),
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
                        &self.l.lstr(temp as u32 + 19),
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

    pub async fn new_reg_text(&self, regname: &str, _regnumber: &str) {
        self.g.fill_box(128, 155, 312, 155, 9);
        self.g.font_color(240);
        self.g.write_font(
            132,
            163,
            &[&self.l.lstr(35) as &[u8], b" ", &self.l.lstr(36)].concat(),
        );
        self.g.font_color(246);
        self.g.fill_box(132, 175, 308, 196, 248);
        self.g.write_font(140, 177, regname.as_bytes());
        self.g.draw_screen().await;
    }

    fn check_file(&self, phase: i32, hill: &mut Hill, str1: &[u8]) {
        let str2: Vec<u8> = if phase == 1 {
            [b"BACK", hill.bk_index.as_slice(), b".PCX"].concat()
        } else {
            [b"FRONT", hill.fr_index.as_slice(), b".PCX"].concat()
        };

        if !self.s.file_exists(&from_utf8(&str2).unwrap()) {
            println!(
                "Error #345A: File {} does not exist,",
                String::from_utf8_lossy(&str2)
            );
            println!(
                "even though it's mentioned in the {} file.",
                String::from_utf8_lossy(str1)
            );
            println!("Using FRONT1.PCX and BACK1.PCX.");
            println!();
            println!("Press a key...");

            hill.bk_index = b"1".to_vec();
            hill.fr_index = b"1".to_vec();
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
    fn findstart(&self, f1: &mut BufReader<P::ReadableFile>, nytmaki: i32) -> u8 {
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

    pub fn hillfile(&self, nyt: i32) -> Vec<u8> {
        if nyt <= self.num_extra_hills.get() as i32 {
            let mut f2 = BufReader::new(self.s.open_file("MOREHILL.SKI"));

            let mut str1 = b"ERROR.SJH".to_vec();
            read_line(&mut f2).unwrap(); //{ NumExtraHills pois }
            for _ in 1..=nyt {
                str1 = read_line(&mut f2).unwrap();
            }
            str1
        } else {
            b"HILLBASE.SKI".to_vec()
        }
    }

    pub fn load_info(&self, nytmaki: i32, hill: &mut Hill) {
        Self::default_hill(hill);

        let str1 = if nytmaki <= NUM_WC_HILLS {
            b"HILLBASE.SKI".to_vec()
        } else {
            [&self.hillfile(nytmaki - NUM_WC_HILLS) as &[u8], b".SJH"].concat()
        };
        let temp = if nytmaki > NUM_WC_HILLS { 0 } else { nytmaki };

        let mut f1 = BufReader::new(self.s.open_file(String::from_utf8(str1.clone()).unwrap()));
        if self.findstart(&mut f1, temp) == 0 {
            hill.name = read_line(&mut f1).unwrap();
            hill.kr = parse_line(&mut f1).unwrap();
            hill.fr_index = read_line(&mut f1).unwrap();
            hill.bk_index = read_line(&mut f1).unwrap();
            hill.bk_bright = parse_line(&mut f1).unwrap();
            hill.bk_mirror = parse_line(&mut f1).unwrap();
            hill.vx_final = parse_line(&mut f1).unwrap();
            let b: u8 = parse_line(&mut f1).unwrap();
            hill.pk = (b as f64) / 100.0;
            let a: i32 = parse_line(&mut f1).unwrap();
            hill.pl_save = (a as f64) / 10000.0;
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

    fn edit_hill(&self, h2: &mut Hill, filestr: &mut Vec<u8>, change: bool) {
        unimplemented!();
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
            let mut f1 = BufReader::new(self.s.open_file("MOREHILL.SKI"));
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

    pub async fn configure_keys(&self, k: &mut [u16]) {
        let mut leave2 = false;
        let mut index2 = 1;
        let mut x = 25;

        loop {
            self.g.new_screen(1, 0).await;

            self.g.font_color(240);
            self.g.write_font(30, 6, self.l.lstr(199));

            let mut y = 0;
            for temp in 1..=6 {
                y = temp * 10 + 30;
                self.g.font_color(240);
                self.g.write_font(x + 10, y, self.l.lstr(temp as u32 + 330));

                let str2 = if temp < 6 {
                    self.keyname(k[temp as usize])
                } else {
                    b"".to_vec()
                };

                self.g.font_color(246);
                self.g.write_font(x + 160, y, &str2);
                self.g
                    .e_write_font(x, y, &[&txt(temp) as &[u8], b"."].concat());
            }

            y += 20;
            self.g.e_write_font(x, y, b"0.");
            self.g.font_color(240);
            self.g.write_font(x + 10, y, self.l.lstr(337));

            index2 = self.make_menu(35, 40, 150, 10, 6, index2, 243, 0, 0).await;

            let mut a: u8 = 0;
            let mut b: u8 = 0;

            match index2 {
                0 => leave2 = true,
                1..=5 => {
                    loop {
                        y = index2 * 10 + 30;

                        self.g
                            .fill_box(180, (y - 2) as u16, 319, (y + 7) as u16, 243);
                        self.g.fill_area(180, y as u16 - 2, 319, (y + 7) as u16, 63);

                        self.getch(185, y, 245).await;
                        a = self.s.get_ch();
                        b = self.s.get_ch2();

                        let mut str2 = self.keyname(kword(a, b));

                        let mut temp2 = 0;

                        for temp in 1..=5 {
                            if kword(a, b) == k[temp as usize] {
                                temp2 = temp;
                            }
                        }

                        if temp2 > 0 && temp2 != index2 {
                            str2 = b"NULL".to_vec();
                        }

                        if str2 != b"NULL" {
                            break;
                        }
                    }
                    k[index2 as usize] = kword(a, b);
                }
                6 => defaultkeys(k),
                _ => unreachable!(),
            }

            if leave2 {
                break;
            }
        }
    }

    pub async fn set_goals(&self) {
        let mut goals = [0; NUM_WC_HILLS as usize + 1];

        load_goals(self.s, &mut goals);

        self.g.new_screen(1, 0).await;

        self.g.font_color(240);
        self.g.write_font(30, 6, self.l.lstr(200));
        self.g.font_color(241);
        self.g.write_font(40, 13, self.l.lstr(243));

        self.g.font_color(246);

        self.g.write_font(24, 23, self.l.lstr(106));
        self.g.e_write_font(200, 23, self.l.lstr(242));
        self.g.e_write_font(250, 23, b"K");
        self.g.e_write_font(300, 23, b"HR");

        let mut index = 1;
        let mut leave = false;

        for temp in 1..=NUM_WC_HILLS {
            let y = temp * 8 + 24;
            self.g.font_color(246);
            self.g
                .e_write_font(18, y, &[&txt(temp) as &[u8], b"."].concat());
            self.g.font_color(240);
            self.g
                .write_font(24, y, &self.g.nsh(&self.hillname(temp), 140));

            self.draw_goal(&goals, temp, 0);
            self.g.font_color(247);
            self.g.e_write_font(250, y, &txtp(self.hillkr(temp) * 10));
            self.g.e_write_font(300, y, &txtp(self.hrlen(temp)));
        }

        self.draw_goal(&goals, NUM_WC_HILLS + 1, 0); //{ exit }

        loop {
            let y = index * 8 + 24;
            self.draw_goal(&goals, index, 1);
            self.g.box_(168, (y - 2) as u16, 202, (y + 6) as u16, 240);

            self.g.draw_screen().await;

            self.s.clearchs();
            let (ch, ch2) = self.s.wait_for_key_press().await;

            let oldindex = index;

            if (ch2 == 75 || ch == b'-') && goals[index as usize] > 4 {
                goals[index as usize] -= 5;
            }
            if (ch2 == 77 || ch == b'+') && goals[index as usize] < 2500 {
                goals[index as usize] += 5;
            }
            if ch2 == 72 && index > 1 {
                index -= 1;
            }
            if ch2 == 80 && index < 21 {
                index += 1;
            }
            if ch == 13 && index == 21 || ch2 == 68 {
                leave = true;
            }
            if ch == 27 || ch2 == 79 {
                index = 21;
            }
            if ch2 == 71 {
                index = 1;
            }

            self.draw_goal(&goals, oldindex, 0);

            if leave {
                break;
            }
        }

        write_goals(self.s, &goals);
    }

    fn draw_goal(&self, goals: &[i32; NUM_WC_HILLS as usize + 1], temp2: i32, phase: i32) {
        if phase == 0 {
            self.g.font_color(240);
        } else {
            self.g.font_color(246);
        }
        let y = temp2 * 8 + 24;
        self.g
            .fill_box(168, (y - 2) as u16, 202, (y + 6) as u16, 243);
        self.g
            .fill_area(168, (y - 2) as u16, 202, (y + 6) as u16, 63);
        if temp2 > 20 {
            self.g.e_write_font(200, y, self.l.lstr(154));
        } else {
            self.g.e_write_font(200, y, &txtp(goals[temp2 as usize]));
        }
    }

    pub fn write_extras(&self) {
        for temp in 1..=self.num_extra_hills.get() {
            self.write_extra_hill_hr(temp as i32 + NUM_WC_HILLS);
        }
    }

    fn write_extra_hill_hr(&self, index: i32) {
        let mut h = Hill::default();
        self.load_info(index, &mut h);

        let hd = self.hd.borrow();
        h.hr_name = hd[index as usize].hrname.clone();
        h.hr_len = hd[index as usize].hrlen;
        h.hr_time = hd[index as usize].hrtime.clone();

        let temp = index - NUM_WC_HILLS;

        let mut f1: Vec<u8> = vec![];
        //     String::from_utf8([&self.hillfile(temp) as &[u8], b".SJH"].concat()).unwrap(),
        // );
        // if let Ok(mut f1) = f1 {
        self.write_hill(&mut f1, h, 0, 1); //{ ainoa mesta jossa phase=1 }
                                           // } else {
                                           //     println!("Error #29: Couldn''t write file {}.SJH", temp);
                                           //     println!("Maybe the disk is full or something.");
                                           // }
    }

    //{ phase: 0 - tavallinen, 1 - ei profiilin checkkausta (HR:t) }
    fn write_hill(&self, mut f: impl Write, mut h: Hill, index: i32, phase: i32) {
        let mut f1 = BufWriter::new(f);

        let mut l1: i32 = 0;

        writeln!(f1, "*{}", (index + 64) as u8 as char).unwrap();
        f1.write_all(&h.name).unwrap();
        writeln!(f1).unwrap();
        writeln!(f1, "{}", h.kr).unwrap();
        f1.write_all(&h.fr_index).unwrap();
        writeln!(f1).unwrap();
        f1.write_all(&h.bk_index).unwrap();
        writeln!(f1).unwrap();
        writeln!(f1, "{}", h.bk_bright).unwrap();
        writeln!(f1, "{}", h.bk_mirror).unwrap();
        writeln!(f1, "{}", h.vx_final).unwrap();

        let b = (h.pk * 100.0).round() as u8;
        writeln!(f1, "{}", b).unwrap();

        let mut a = (h.pl_save * 10000.0).round() as i32;
        writeln!(f1, "{}", a).unwrap();

        f1.write_all(&h.author).unwrap();
        writeln!(f1).unwrap();

        l1 += valuestr(&h.name, index + 1) as i32;
        l1 += h.kr * 77;
        let mut c = num(&h.fr_index); //{ t�m� yhteensopivuudenkin takia }
        if c < 0 {
            c = valuestr(&h.fr_index, index + 1) as i32; //{ jos indexiss� kirjaimia (v3.10) }
        }
        l1 += c * 272;
        c = num(&h.bk_index); //{ t�m� my�s }
        if c < 0 {
            c = valuestr(&h.bk_index, index + 1) as i32;
        }
        l1 += c * 373;
        l1 += h.bk_bright as i32 * 313;
        l1 += h.bk_mirror as i32 * 5775;
        l1 += h.vx_final as i32 * 333;

        l1 += b as i32 % 55555;
        l1 += a % 11111;
        l1 += valuestr(&h.author, index + 2) as i32;

        l1 ^= 787371;

        writeln!(f1, "{}", l1).unwrap();

        let mut l2: i32 = 0;
        if phase == 1 {
            writeln!(f1, "{}", h.profile).unwrap();
        } else {
            h.hr_name = b"Default\xff".to_vec();
            h.hr_len = 0;
            h.hr_time = b"Jan 1 2001 1:00".to_vec(); //{ nollataan hillrec }

            self.p.lataa_pcx(
                &format!("FRONT{}.PCX", from_utf8(&h.fr_index).unwrap()),
                1024 * 512,
                0,
                0,
            );
            self.m.laske_linjat(&mut a, h.kr, h.pk);

            for a in 0..=1023 {
                l2 += (self.m.profiili(a) * (a % 13 + self.m.profiili(a) % 11)) % 13313;
            }

            l2 -= 1500000;

            writeln!(f1, "{}", l2).unwrap();
        }

        if index == 0 {
            //{ extra hill }
            l1 = 0;

            f1.write_all(&h.hr_name).unwrap(); //{ m�kienkkatiedot }
            writeln!(f1).unwrap();
            writeln!(f1, "{}", h.hr_len).unwrap();
            f1.write_all(&h.hr_time).unwrap();
            writeln!(f1).unwrap();

            l1 += valuestr(&h.hr_name, 13) as i32; //{ checksum }
            l1 += h.hr_len as i32 * 3553;
            writeln!(f1, "{}", l1).unwrap();
        }
    }

    pub async fn result_box(&self, phase: u8, result: i32) {
        let mut temp = 348 + (phase as i32 * 2);

        if result != 0 {
            temp += 1;
        }

        //{ 1 - repl, 2 - custo, 3 - extrh }

        self.g.fill_box(59, 79, 261, 121, 250);
        self.g.fill_box(60, 80, 260, 120, 245);

        self.g.font_color(246);

        self.g.write_font(75, 90, self.l.lstr(temp as u32));
        self.wait_for_key3(240, 105).await;
    }

    pub async fn hill_maker(&self, phase: u8) {
        let show = 18;

        let mut index;
        let mut start = 0; //{ sivulis�ys... }

        let cols = [/* unused */ 0, 100, 160];

        let mut newfile = false;

        loop {
            self.g.new_screen(5, 0).await;

            //{ koodeja... }
            let mut create = 0;
            let mut next = 0;
            let mut prev = 0;

            self.g.font_color(246);
            self.g.write_font(5, 5, self.l.lstr(270));

            self.g.font_color(241);
            self.g.write_font(5, 21, self.l.lstr(271));
            self.g.write_font(5, 29, self.l.lstr(272));

            self.g.font_color(247);

            let str1 = [
                &txt(1 + (start / show)) as &[u8],
                b" ",
                &self.l.lstr(8),
                b" ",
                &txt((self.num_extra_hills.get() as i32 - 1) / show + 1),
            ]
            .concat();

            self.g
                .write_font(5, 45, &[&self.l.lstr(157) as &[u8], b" ", &str1].concat());
            self.g.write_font(cols[1], 5, self.l.lstr(273));
            self.g.write_font(cols[2], 5, self.l.lstr(274));

            self.g.font_color(240);

            let mut temp2 = self.num_extra_hills.get() as i32 - start;
            if temp2 > show {
                temp2 = show;
            }

            let mut items = temp2;

            if self.num_extra_hills.get() > 0 {
                for temp in 1..=temp2 {
                    let str1 = self.g.nsh(
                        &[
                            &self.hillname(start + temp + NUM_WC_HILLS) as &[u8],
                            b" K",
                            &txt(self.hillkr(start + temp + NUM_WC_HILLS)),
                        ]
                        .concat(),
                        150,
                    );
                    self.g
                        .write_font(cols[1], 5 + temp * 8, &self.hillfile(start + temp));
                    self.g.write_font(cols[2], 5 + temp * 8, &str1);
                }
            }

            let mut temp = temp2;
            if (self.num_extra_hills.get() as i32) < MAX_EXTRA_HILLS {
                self.g.font_color(246);
                temp += 1;
                items += 1;
                create = items;
                self.g.write_font(cols[1], 5 + temp * 8, self.l.lstr(275)); //{ * add new * }
            }

            self.g.font_color(247);

            if start + temp2 < self.num_extra_hills.get() as i32 {
                temp += 1;
                items += 1;
                next = items;
                self.g.write_font(cols[1], 5 + temp * 8, self.l.lstr(158)); //{ * next page * }
            }

            if start > 0 {
                temp += 1;
                items += 1;
                prev = items;
                self.g.write_font(cols[1], 5 + temp * 8, self.l.lstr(159)); //{ * previous page * }
            }

            self.g.font_color(240);

            temp += 2;
            self.g.write_font(cols[1], 5 + temp * 8, self.l.lstr(276));

            self.g.draw_screen().await;

            index = self.make_menu(99, 14, 221, 8, items, 1, 243, 1, 0).await;

            if index < 0 && index.abs() + start <= self.num_extra_hills.get() as i32 {
                //{ delete? }
                let filestr = [&self.hillfile(index.abs() + start) as &[u8], b".SJH"].concat();

                if self.might_delete(&filestr).await == 0 {
                    self.check_extra_hills();
                    self.load_hill_info();
                }
            }

            if index > 0 {
                let mut h = Hill::default();
                let mut filestr = Vec::<u8>::new();
                let mut edit = false;

                if index == next {
                    start += show;
                }
                if index == prev {
                    start -= show;
                }

                if index == create {
                    filestr = [b"NEW", &txt(index + start) as &[u8]].concat();
                    h = Hill::default();
                    edit = true;
                    newfile = true;
                }

                if index <= temp2 {
                    //{ uusi }
                    self.load_info(start + index + NUM_WC_HILLS, &mut h);
                    filestr = self.hillfile(start + index);
                    edit = true;
                }

                if edit {
                    self.edit_hill(&mut h, &mut filestr, newfile);

                    if filestr != b"NULL" {
                        //{ sitten kirjoitetaan p��lle }
                        {
                            let mut f1 = self
                                .s
                                .create_file(String::from_utf8(filestr).unwrap() + ".SJH");
                            self.write_hill(&mut f1, h.clone(), 0, 0);
                        }

                        self.g.new_screen(5, 0).await;
                        self.result_box(3, 0).await;
                    }

                    self.check_extra_hills();
                    self.load_hill_info();
                }
            }

            if index == 0 {
                break;
            }
        }
    }

    pub fn read_extras(&self) {
        let mut hd = self.hd.borrow_mut();
        for temp in 1..=self.num_extra_hills.get() as i32 {
            let index = temp + NUM_WC_HILLS;

            let mut h = Hill::default();
            self.load_info(index, &mut h);

            hd[index as usize].hrname = h.hr_name;
            hd[index as usize].hrlen = h.hr_len;
            hd[index as usize].hrtime = h.hr_time;
        }
    }

    async fn might_delete(&self, filestr: &[u8]) -> u8 {
        let mut tempb = 1;

        self.g.alert_box();

        self.g.write_font(
            75,
            90,
            &[&self.l.lstr(194) as &[u8], b" ", &filestr].concat(),
        );
        self.g.write_font(
            75,
            110,
            &[
                &self.l.lstr(193) as &[u8],
                b" (",
                &[self.l.lch(6, 1)],
                b"/",
                &[self.l.lch(7, 1)],
                b")",
            ]
            .concat(),
        );

        self.getch(220, 110, 243).await;

        if self.s.get_ch().to_ascii_uppercase() == self.l.lch(6, 1).to_ascii_uppercase() {
            self.s.remove_file(from_utf8(filestr).unwrap());
            tempb = 0; //{ yep, deleted }
        }

        self.s.clearchs();
        tempb
    }

    pub async fn choose_wind_place(&self, place: &mut u8) {
        let winds = 11;

        self.g.fill_box(54, 19, 276, 181, 248);
        self.g.fill_box(55, 20, 275, 180, 243);

        self.g.font_color(246);
        self.g.write_font(75, 30, self.l.lstr(221));

        let mut yy = 0;
        for apu1 in 1..=winds {
            yy = apu1 * 10 + 34;
            //{ 390-top, 391-middle, 392-bottom, 393-left, 394-center, 395-right, 396-jpr }
            let str1 = match apu1 {
                1 => [&self.l.lstr(392) as &[u8], b"-", &self.l.lstr(393)].concat(),
                2 => [&self.l.lstr(391) as &[u8], b"-", &self.l.lstr(393)].concat(),
                3 => [&self.l.lstr(392) as &[u8], b"-", &self.l.lstr(395)].concat(),
                4 => [&self.l.lstr(392) as &[u8], b"-", &self.l.lstr(394)].concat(),
                5 => [&self.l.lstr(391) as &[u8], b"-", &self.l.lstr(395)].concat(),
                6 => [&self.l.lstr(390) as &[u8], b"-", &self.l.lstr(395)].concat(),
                7 => [&self.l.lstr(390) as &[u8], b"-", &self.l.lstr(394)].concat(),
                8 => [&self.l.lstr(390) as &[u8], b"-", &self.l.lstr(393)].concat(),
                9 => [&self.l.lstr(396) as &[u8], b": ", &self.l.lstr(390)].concat(), //{ oikeasti 11 }
                10 => [&self.l.lstr(396) as &[u8], b": ", &self.l.lstr(391)].concat(), //{ 12 }
                11 => [&self.l.lstr(396) as &[u8], b": ", &self.l.lstr(392)].concat(), //{ 13 }
                _ => unreachable!(),
            };
            self.g.font_color(246);
            self.g
                .e_write_font(85, yy, &[&txt(apu1) as &[u8], b"."].concat());
            self.g.font_color(240);
            self.g.write_font(90, yy, &str1);
        }

        yy += 20;
        self.g.font_color(246);
        self.g.e_write_font(85, yy, b"0.");
        self.g.font_color(240);
        self.g.write_font(90, yy, self.l.lstr(154));

        let mut apu1 = *place;
        if apu1 > 10 {
            apu1 -= 2;
        }

        let mut index = self
            .make_menu(70, 44, 140, 10, winds, apu1 as i32, 243, 4, 0)
            .await;
        if index > 8 {
            index += 2;
        }
        if index > 0 {
            *place = index as u8;
        }
    }

    //{ 0-main menu, 1-world cup }
    pub async fn quitting(&self, phase: u8) -> u8 {
        let mut str1 = self.l.lrstr(251, 253);
        if phase == 1 {
            str1 = self.l.lstr(245);
        }
        let str2 = self.l.lrstr(256, 258);

        self.g.alert_box();

        let mut tempb = self.g.font_len(str2.clone()) + 4;

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

        self.getch(70 + tempb, 110, 243).await;

        tempb = 1;
        if self.s.get_ch().to_ascii_uppercase() == self.l.lstr(6)[0] {
            tempb = 0;
        }
        self.s.clearchs();

        tempb as u8
    }

    pub async fn wait_for_key3(&self, xx: i32, yy: i32) -> bool {
        while self.s.key_pressed().await {
            self.s.wait_for_key_press().await;
        }

        self.g.font_color(240);

        self.g.e_write_font(xx, yy, self.l.lstr(15));

        self.getch(xx + 1, yy, 243).await;

        if self.s.get_ch() == 0 && self.s.get_ch2() == 68 {
            self.s.set_ch(27);
            true
        } else {
            false
        }
    }
}

pub fn crypt(arvo: i32, jarj: i32) -> Vec<u8> {
    let str1 = format!("{:05}", arvo);
    let mut str2: Vec<u8> = Vec::new();

    for c in str1.chars().rev() {
        str2.push((c as u8) + 21);
    }

    let ch1 = (68 + 2 * ((arvo % 7) ^ 1)) as u8; //{ tarkistusluku arvosta }
    if txt(arvo).len() > 5 {
        str2.insert(0, ch1);
    } else {
        str2.insert(1, ch1);
    }
    str2.insert(1, ch1);

    let ch1 = (65 + ((jarj ^ 33) % 19)) as u8; //{ tarkistusluku j�rjestyksest� }
    str2.insert(2, ch1);

    let mut ch1 = (68 + random(6)) as u8; //{ ihan vaan satunnainen kirjain }
    if txt(arvo).len() > 5 {
        ch1 = str1.as_bytes()[5] + 27;
    }
    str2.insert(4, ch1);

    str2
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

pub fn loadgoal<P: Platform>(s: &P, num: i32) -> i32 {
    let mut value: i32 = 0;

    if num > 0 && num <= NUM_WC_HILLS {
        let mut f2 = s.open_file("GOALS.SKI");
        let mut b = BufReader::new(f2).lines();
        for _ in 1..=num {
            let line = b.next().unwrap().unwrap();
            value = line.parse().unwrap();
        }
    }
    value
}

fn load_goals<P: Platform>(s: &P, goals: &mut [i32; NUM_WC_HILLS as usize + 1]) {
    let mut f2 = BufReader::new(s.open_file("GOALS.SKI"));
    for temp2 in 1..=NUM_WC_HILLS as usize {
        goals[temp2] = parse_line(&mut f2).unwrap();
    }
}

fn write_goals(s: &impl Platform, goals: &[i32; NUM_WC_HILLS as usize + 1]) {
    let mut f2 = BufWriter::new(s.create_file("GOALS.SKI"));
    for temp2 in 1..=NUM_WC_HILLS as usize {
        writeln!(f2, "{}", goals[temp2]).unwrap();
    }
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
