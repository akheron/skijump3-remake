use crate::graph::GraphModule;
use crate::help::{beep, cwrite, cwriteln, num, round, txt, txtp};
use crate::lang::LangModule;
use crate::maki::{MakiModule, SIVUJA};
use crate::pcx::PcxModule;
use crate::platform::Platform;
use crate::rs_util::{
    clrscr, dos_file_name, dos_name_bytes, gotoxy, halt, parse_line, random, read_line,
    textbackground, textcolor, val, val_code, window, write, writeln,
};
use crate::trace;
use std::cell::{Cell, RefCell};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::rc::Rc;

pub const NUM_PL: usize = 75; //{ montako pelaajaa on ylip��t��n }
pub const NUM_TEAMS: usize = 15;
pub const MAX_OWN_PL: i32 = 10; //{ montako omaa pelaajaa voi olla }
pub const NUM_WC_HILLS: i32 = 20; //{ montako m�ke� world cupissa }
pub const MAX_EXTRA_HILLS: i32 = 1000; //{ montako extra m�ke� voi olla. check!!! }
pub const MAX_CUSTOMS: i32 = 200; //{ montako custom hill filea .sjc voi olla }

pub const HEX_CH: [&[u8]; 16] = [
    b"0", b"1", b"2", b"3", b"4", b"5", b"6", b"7", b"8", b"9", b"A", b"B", b"C", b"D", b"E", b"F",
];

pub const WC_POINTS: [u8; 31] = [
    0, 100, 80, 60, 50, 45, 40, 36, 32, 29, 26, 24, 22, 20, 18, 16, 15, 14, 13, 12, 11, 10, 9, 8,
    7, 6, 5, 4, 3, 2, 1,
];
pub const TEAM_POINTS: [i32; 9] = [0, 400, 350, 300, 250, 200, 150, 100, 50];

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
pub struct Date {
    pub year: u16,
    pub month: u16,
    pub day: u16,
    pub day_of_week: u16,
}

#[derive(Clone, Copy, Default)]
pub struct Time {
    pub hour: u16,
    pub minute: u16,
    pub second: u16,
    pub sec100: u16,
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
                (0..=NUM_WC_HILLS + MAX_EXTRA_HILLS).map(|_| Hillinfo::default()),
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

        self.g.write_font(xx, yy, &[self.s.get_ch()]);
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

    pub async fn getstr(
        &self,
        xx: i32,
        yy: i32,
        maxlength: i32,
        oldstr: &[u8],
        bkcolor: u8,
    ) -> Vec<u8> {
        let mut out = false;

        let mut newstr = oldstr.to_vec();

        loop {
            let cx = self.g.font_len(&newstr);

            self.g.fill_box(
                (xx - 2) as u16,
                (yy - 2) as u16,
                (xx + maxlength + 2) as u16,
                (yy + 7) as u16,
                bkcolor,
            );
            self.g.write_font(xx, yy, &newstr);

            self.g.draw_screen().await;

            self.givech(xx + cx, yy, bkcolor).await;
            let ch = self.s.get_ch();

            if self.s.get_ch2() == 83 {
                newstr.clear();
            }

            match ch {
                8 => {
                    newstr.pop();
                }
                13 | 27 => out = true,
                32..=254 => {
                    if cx + 7 < maxlength && self.g.font_len(&[ch]) > 0 {
                        newstr.push(ch);
                    }
                }
                _ => {}
            }

            if out {
                break;
            }
        }

        if self.s.get_ch() == 27 {
            newstr = oldstr.to_vec();
        }

        newstr
    }

    pub async fn getstr2(
        &self,
        xx: i32,
        yy: i32,
        maxchars: i32,
        oldstr: &[u8],
        bkcolor: u8,
    ) -> Vec<u8> {
        let mut out = false;

        let mut newstr = oldstr.to_vec();

        loop {
            let cx = self.g.font_len(&newstr);

            self.g.fill_box(
                (xx - 2) as u16,
                (yy - 2) as u16,
                (xx + (maxchars * 7) + 2) as u16,
                (yy + 8) as u16,
                bkcolor,
            );
            self.g.write_font(xx, yy, &newstr);

            self.g.draw_screen().await;

            self.givech(xx + cx, yy, bkcolor).await;
            let ch = self.s.get_ch();

            if self.s.get_ch2() == 83 {
                newstr.clear();
            }

            match ch {
                8 => {
                    newstr.pop();
                }
                13 | 27 => out = true,
                32..=254 => {
                    if (newstr.len() as i32) < maxchars && self.g.font_len(&[ch]) > 0 {
                        newstr.push(ch);
                    }
                }
                _ => {}
            }

            if out {
                break;
            }
        }

        if self.s.get_ch() == 27 {
            newstr = b" ".to_vec();
        }

        newstr
    }

    pub fn get_now(&self) -> Time {
        let (hour, minute, second, sec100) = self.s.get_time();
        Time {
            hour,
            minute,
            second,
            sec100,
        }
    }

    pub fn kothtitle(&self, temp: i32) -> Vec<u8> {
        let mut str1: Vec<u8> = Vec::new();

        match temp {
            1..=6 => str1 = [&txt(temp) as &[u8], b". ", &self.l.lstr(130 + temp as u32)].concat(),
            0 | 7 => str1 = [b"0. " as &[u8], &self.l.lstr(137)].concat(),
            _ => {}
        }

        str1
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

    pub fn kothchallenge(&self, x: i32, mut kothpack: u8) {
        let mut y: i32;

        self.g.font_color(246);
        self.g.write_font(x, 110, &*self.l.lstr(130));

        y = 120;

        if kothpack == 0 {
            kothpack = 7;
        }

        for temp in 1..=7 {
            self.g.font_color(241);
            if temp == kothpack as i32 || kothpack == 255 {
                self.g.font_color(240);
            }

            self.g.write_font(x, y, &self.kothtitle(temp));

            y += 8;
            if temp == 6 {
                y += 8; //{ ylimääräiset }
            }
        }
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
            &dos_file_name(&[b"FRONT", act_hill.fr_index.as_slice(), b".PCX"].concat()),
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
            &dos_file_name(&[b"BACK", act_hill.bk_index.as_slice(), b".PCX"].concat()),
            1024 * 400,
            SIVUJA,
            act_hill.bk_mirror,
        );

        self.p.takaisin_alkuosa(1);

        self.trace_profile(*keula_x, l);

        if l != act_hill.profile {
            //{ profiilichekkaus }
            self.g.alert_box();

            str1 = b"RUN THE HILL MAKER AGAIN.".to_vec();

            if nytmaki <= NUM_WC_HILLS {
                res = 1;
                str1 = b"EXITING CUP.".to_vec();
            }

            self.g.write_font(
                80,
                90,
                &[b"THE PROFILE OF HILL #", &act_hill.fr_index as &[u8]].concat(),
            );
            self.g.write_font(80, 102, b"HAS BEEN CHANGED! NOT GOOD.");
            self.g.write_font(80, 114, &str1);

            self.g.write_font(80, 130, b"PRESS A KEY...");
            self.g.draw_screen().await;

            self.s.wait_for_key().await;
        }

        self.p.savyta_paletti(1, act_hill.bk_bright);
        res
    }

    fn trace_profile(&self, keula_x: i32, l: i32) {
        let profiilit: Vec<u8> = (0..=1300)
            .flat_map(|x| (self.m.profiili(x) as i16).to_le_bytes())
            .collect();
        trace::record(
            "HillProfile",
            &[
                ("keulax", keula_x.into()),
                ("keulay", self.m.profiili(keula_x).into()),
                ("profile", l.into()),
                ("profiilicrc", (trace::crc32(&[&profiilit]) as i32).into()),
            ],
        );
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
        let mut cstr: Vec<Rc<Vec<u8>>> = vec![Rc::new(Vec::new()); 6];
        let index: i32 = 360 + style as i32 * 40;
        let lstr = |offset: i32| self.l.lstr((index + offset) as u32);

        match kulmalaskuri {
            0..=49 => cstr[0] = lstr(2),
            50..=61 => cstr[0] = lstr(3),
            62..=200 => cstr[0] = lstr(4),
            _ => {}
        }

        if grade < 10 {
            match grade {
                1 => cstr[1] = lstr(5),
                2 => cstr[1] = lstr(6),
                3 => cstr[1] = lstr(7),
                _ => {}
            }
        } else {
            match grade / 10 {
                1..=5 => cstr[1] = lstr(10),
                6..=8 => cstr[1] = lstr(11),
                9 => cstr[1] = lstr(12),
                10 => cstr[1] = lstr(13),
                11 => cstr[1] = lstr(14),
                12..=20 => cstr[1] = lstr(15),
                _ => {}
            }
        }

        match ponn {
            0..=5 => cstr[2] = lstr(18),
            6..=9 => cstr[2] = lstr(19),
            10..=12 => cstr[2] = lstr(20),
            13..=15 => cstr[2] = lstr(21),
            16 => cstr[2] = lstr(22),
            17..=19 => cstr[2] = lstr(23),
            20..=23 => cstr[2] = lstr(24),
            24..=50 => cstr[2] = lstr(25),
            _ => {}
        }

        match height {
            0..=49 => cstr[3] = lstr(28),
            50..=55 => cstr[3] = lstr(29),
            56..=60 => cstr[3] = lstr(30),
            61..=64 => cstr[3] = lstr(31),
            65..=70 => cstr[3] = lstr(32),
            71..=90 => cstr[3] = lstr(33),
            91..=200 => cstr[3] = lstr(34),
            _ => {}
        }

        //{ jos kupat n�es }
        if grade < 10 {
            cstr[0] = cstr[1].clone();
        }
        if grade == 1 {
            cstr[3] = lstr(35);
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

    pub async fn hillcode(&self, nytmaki: i32) -> i32 {
        let mut temp: i32 = 0;
        let mut h = Hill::default();

        if nytmaki > 0 && nytmaki <= NUM_WC_HILLS + self.num_extra_hills.get() as i32 {
            self.load_info(nytmaki, &mut h).await;
            temp = h
                .profile
                .wrapping_add(round((h.pl_save as f64 * 311.0) as f32))
                .wrapping_add(h.vx_final as i32 * 131)
                .wrapping_add(round(h.pk) * 17)
                .wrapping_add(h.kr * 11)
                .wrapping_add(svaluestr(&h.name, 1) as i32);
        }

        temp
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

        let b = round(h.pk * 100.0) as u8;
        writeln!(f1, "{}", b).unwrap();

        let mut a = round(h.pl_save * 10000.0);
        writeln!(f1, "{}", a).unwrap();

        f1.write_all(&h.author).unwrap();
        writeln!(f1).unwrap();

        l1 += valuestr(&h.name, index + 1) as i32;
        l1 += h.kr * 77;
        let mut c = num(&h.fr_index) as i16; //{ t�m� yhteensopivuudenkin takia }
        if c < 0 {
            c = valuestr(&h.fr_index, index + 1) as i16; //{ jos indexiss� kirjaimia (v3.10) }
        }
        l1 += c as i32 * 272;
        c = num(&h.bk_index) as i16; //{ t�m� my�s }
        if c < 0 {
            c = valuestr(&h.bk_index, index + 1) as i16;
        }
        l1 += c as i32 * 373;
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
                &dos_file_name(&[b"FRONT", h.fr_index.as_slice(), b".PCX"].concat()),
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

    // 0 - löytyi, 1 - ei lytynyt
    fn findstart(&self, f1: &mut BufReader<P::ReadableFile<'_>>, nytmaki: i32) -> u8 {
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

    async fn check_file(&self, phase: i32, hill: &mut Hill, str1: &[u8]) {
        let str2: Vec<u8> = if phase == 1 {
            [b"BACK", hill.bk_index.as_slice(), b".PCX"].concat()
        } else {
            [b"FRONT", hill.fr_index.as_slice(), b".PCX"].concat()
        };

        if !self.s.file_exists(dos_file_name(&str2)) {
            self.m.aseta_moodi(0x3);
            writeln(&[b"Error #345A: File ", str2.as_slice(), b" does not exist,"].concat());
            writeln(&[b"even though it's mentioned in the ", str1, b" file."].concat());
            writeln(b"Using FRONT1.PCX and BACK1.PCX.");
            writeln(b"");
            writeln(b"Press a key...");

            hill.bk_index = b"1".to_vec();
            hill.fr_index = b"1".to_vec();

            self.s.wait_for_key().await;

            self.m.aseta_moodi(0x13);
        }
    }

    pub async fn load_info(&self, nytmaki: i32, hill: &mut Hill) {
        let mut l1: i32 = 0;
        let mut l2: i32 = 0;
        let mut l4: i32 = 0;

        Self::default_hill(hill);

        let str1 = if nytmaki <= NUM_WC_HILLS {
            b"HILLBASE.SKI".to_vec()
        } else {
            [&self.hillfile(nytmaki - NUM_WC_HILLS) as &[u8], b".SJH"].concat()
        };
        let temp = if nytmaki > NUM_WC_HILLS { 0 } else { nytmaki };

        let mut f1 = BufReader::new(self.s.open_file(dos_file_name(&str1)));
        if self.findstart(&mut f1, temp) == 0 {
            // ReadLn into a string[n] field of Hill_type keeps n characters
            hill.name = read_line(&mut f1).unwrap();
            hill.name.truncate(25);
            hill.kr = parse_line::<i16>(&mut f1).unwrap() as i32;
            hill.fr_index = read_line(&mut f1).unwrap();
            hill.fr_index.truncate(3);
            hill.bk_index = read_line(&mut f1).unwrap();
            hill.bk_index.truncate(3);
            hill.bk_bright = parse_line(&mut f1).unwrap();
            hill.bk_mirror = parse_line(&mut f1).unwrap();
            hill.vx_final = parse_line(&mut f1).unwrap();
            let b: u8 = parse_line(&mut f1).unwrap();
            hill.pk = (b as f64 / 100.0) as f32;
            let a: i16 = parse_line(&mut f1).unwrap();
            hill.pl_save = (a as f64 / 10000.0) as f32;
            hill.author = read_line(&mut f1).unwrap();
            hill.author.truncate(30);
            l2 = parse_line(&mut f1).unwrap(); //{ checksum }
            hill.profile = parse_line(&mut f1).unwrap();

            l1 = l1.wrapping_add(valuestr(&hill.name, temp + 1) as i32);
            l1 = l1.wrapping_add(hill.kr * 77);
            let mut c = num(&hill.fr_index) as i16; //{ tämä yhteensopivuudenkin takia }
            if c < 0 {
                c = valuestr(&hill.fr_index, temp + 1) as i16; //{ jos indexissä kirjaimia (v3.10) }
            }
            l1 = l1.wrapping_add(c as i32 * 272);
            c = num(&hill.bk_index) as i16; //{ tämä myös }
            if c < 0 {
                c = valuestr(&hill.bk_index, temp + 1) as i16;
            }
            l1 = l1.wrapping_add(c as i32 * 373);
            l1 = l1.wrapping_add(hill.bk_bright as i32 * 313);
            l1 = l1.wrapping_add(hill.bk_mirror as i32 * 5775);
            l1 = l1.wrapping_add(hill.vx_final as i32 * 333);
            l1 = l1.wrapping_add(b as i32 % 55555);
            l1 = l1.wrapping_add(a as i32 % 11111);
            l1 = l1.wrapping_add(valuestr(&hill.author, temp + 2) as i32);

            l1 ^= 787371;

            if nytmaki > NUM_WC_HILLS {
                //{ voi olla ettei näitä oo - v3.00}
                let hr_name = read_line(&mut f1);
                let hr_len = parse_line::<i16>(&mut f1);
                let hr_time = read_line(&mut f1);
                let l3 = parse_line::<i32>(&mut f1);
                match (hr_name, hr_len, hr_time, l3) {
                    (Ok(hr_name), Ok(hr_len), Ok(hr_time), Ok(l3)) if l3 != 0 => {
                        //{ kaikki ok }
                        hill.hr_name = hr_name;
                        hill.hr_name.truncate(30);
                        hill.hr_len = hr_len as i32;
                        hill.hr_time = hr_time;
                        hill.hr_time.truncate(22);
                        l4 = l4.wrapping_add(valuestr(&hill.hr_name, 13) as i32);
                        l4 = l4.wrapping_add(hill.hr_len * 3553);
                        if l3 != l4 {
                            l1 = l1.wrapping_add(1);
                        }
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

        self.check_file(0, hill, &str1).await;
        self.check_file(1, hill, &str1).await;

        if l1 != l2 {
            //{ joku ei täsmää! }
            if nytmaki < NUM_WC_HILLS {
                self.m.aseta_moodi(0x3);
                writeln(
                    &[
                        b"Error #324A: Something's wrong in the ",
                        &str1 as &[u8],
                        b" file.  ",
                    ]
                    .concat(),
                );
                writeln(b"Maybe it's been edited or something.  That won't do.  Exiting.");
                self.s.wait_for_key().await;
                halt().await;
            } else {
                beep(self.s, 1);

                self.m.aseta_moodi(0x3);

                writeln(
                    &[
                        b"Warning #56B: Something doesn't add up in the ",
                        &str1 as &[u8],
                        b" file. ",
                    ]
                    .concat(),
                );
                writeln(b"Continuing with a default hill.");
                writeln(b"");
                writeln(b"Press a key...");

                self.s.wait_for_key().await;
                Self::default_hill(hill);

                self.m.aseta_moodi(0x13);
            }
        }

        trace::record(
            "HillLoad",
            &[
                ("nytmaki", nytmaki.into()),
                ("name", (&hill.name as &[u8]).into()),
                ("kr", hill.kr.into()),
                ("pk", hill.pk.into()),
                ("plsave", hill.pl_save.into()),
                ("profile", hill.profile.into()),
                ("frindex", (&hill.fr_index as &[u8]).into()),
                ("bkindex", (&hill.bk_index as &[u8]).into()),
                ("bkbright", hill.bk_bright.into()),
                ("bkmirror", hill.bk_mirror.into()),
                ("vxfinal", hill.vx_final.into()),
                ("checksum", (l1 == l2).into()),
            ],
        );
    }

    fn checkdiff(h1: &Hill, h2: &Hill) -> bool {
        let mut tempb = false;

        if h1.name != h2.name
            || h1.author != h2.author
            || h1.kr != h2.kr
            || h1.fr_index != h2.fr_index
            || h1.bk_index != h2.bk_index
            || h1.bk_bright != h2.bk_bright
            || h1.bk_mirror != h2.bk_mirror
            || h1.vx_final != h2.vx_final
            || h1.pk != h2.pk
            || h1.pl_save != h2.pl_save
            || h1.profile != h2.profile
        {
            tempb = true;
        }

        tempb
    }

    async fn getnum(&self, xx2: i32, yy: i32, thisnum: &mut u8, low: u8, high: u8) {
        let str2 = self.getstr(xx2, yy, 30, &txt(*thisnum as i32), 242).await;
        if let Some(mut temp) = val_code(&str2) {
            if temp < low as i32 {
                temp = low as i32;
            }
            if temp > high as i32 {
                temp = high as i32;
            }

            *thisnum = temp as u8;
        }
    }

    async fn getint(&self, xx2: i32, yy: i32, thisnum: &mut i32, low: i32, high: i32) {
        let str2 = self.getstr(xx2, yy, 30, &txt(*thisnum), 242).await;
        if let Some(mut temp) = val_code(&str2) {
            if temp < low {
                temp = low;
            }
            if temp > high {
                temp = high;
            }

            *thisnum = temp;
        }
    }

    async fn edit_hill(&self, h2: &mut Hill, filestr: &mut Vec<u8>, mut change: bool) {
        let mut leave2 = false;
        let mut index2 = 1;
        let xx = 15;
        let xx2 = 120;
        let mut yy;

        let oldhill = h2.clone();

        loop {
            self.g.new_screen(5, 0).await;

            for temp in 1..=12 {
                let mut str1: Vec<u8> = match temp {
                    1 => b"HILL.NAME".to_vec(),
                    2 => b"K.POINT".to_vec(),
                    3 => b"FRONT INDEX".to_vec(),
                    4 => b"BACK INDEX".to_vec(),
                    5 => b"BACK BRIGHT".to_vec(),
                    6 => b"MIRROR BACK".to_vec(),
                    7 => b"VX.FINAL".to_vec(),
                    8 => b"ADJUST.K".to_vec(),
                    9 => b"AIR.P.PLUS".to_vec(),
                    10 => b"AUTHOR".to_vec(),
                    11 => b"FILENAME".to_vec(),
                    _ => b"EXIT, DON'T SAVE".to_vec(),
                };

                yy = 10 + (temp - 1) * 13;

                self.g.font_color(240);
                if temp < 12 {
                    str1.push(b':');
                }
                self.g
                    .write_font(xx, yy, &[&txt(temp) as &[u8], b". ", &str1].concat());

                let str1: &[u8] = match temp {
                    2 => b"Critical Point (40-300\xab)",
                    3 => b"FRONT*.PCX (000-ZZZ)",
                    4 => b"BACK*.PCX (000-ZZZ)",
                    5 => b"PERCENTS OF ORIGINAL (0-255)",
                    6 => b"0-NO, 1-MIRROR (0-1)",
                    7 => b"TAKE-OFF SPEED (60-145 KM/H)",
                    8 => b"MOVE K-POINT ON HILL (50-150)",
                    9 => b"+ 990 MBAR (0-30)",
                    11 => b"     8 CHARS MAX",
                    _ => b"",
                };

                self.g.font_color(241);
                self.g.write_font(xx2 + 40, yy, str1);

                let str1: Vec<u8> = match temp {
                    1 => h2.name.clone(),
                    2 => txt(h2.kr),
                    3 => h2.fr_index.clone(),
                    4 => h2.bk_index.clone(),
                    5 => txt(h2.bk_bright as i32),
                    6 => txt(h2.bk_mirror as i32),
                    7 => txt(h2.vx_final as i32 - 40),
                    8 => txt(round((h2.pk as f64 * 100.0) as f32)),
                    9 => txt(round((h2.pl_save as f64 * 10000.0) as f32) - 3204),
                    10 => h2.author.clone(),
                    11 => filestr.clone(),
                    _ => Vec::new(),
                };

                self.g.font_color(246);
                self.g.write_font(xx2, yy, &str1);
            }

            let temp = 14;

            yy = 10 + (temp - 1) * 13;

            self.g.write_font(xx, yy, b"0. EXIT and SAVE");

            index2 = self.make_menu(10, 8, 110, 13, 12, index2, 243, 0, 0).await;

            yy = 10 + (index2 - 1) * 13;

            // Name is string[25], Author string[30], and FrIndex and BkIndex
            // string[3] in Hill_type, so an assignment to them truncates.
            match index2 {
                0 => {
                    if !change {
                        change = Self::checkdiff(&oldhill, h2);
                    }
                    if !change {
                        *filestr = b"NULL".to_vec(); //{ ei tarvi tallentaa }
                    }
                    leave2 = true;

                    if filestr != b"NULL" {
                        if self
                            .s
                            .file_exists(dos_file_name(&[filestr as &[u8], b".SJH"].concat()))
                        {
                            self.g.alert_box();

                            self.g.write_font(
                                80,
                                90,
                                &[b"FILE ", filestr as &[u8], b".SJH ALREADY EXISTS."].concat(),
                            );
                            self.g.write_font(80, 110, b"WRITE OVER? (Y/N):");

                            self.getch(190, 110, 243).await;

                            if self.s.get_ch().to_ascii_uppercase() != b'Y' {
                                leave2 = false;
                                index2 = 11;
                            }
                        }
                    }
                }
                1 => {
                    h2.name = self.getstr(xx2, yy, 150, &h2.name, 242).await;
                    h2.name.truncate(25);
                }
                2 => self.getint(xx2, yy, &mut h2.kr, 40, 300).await,
                3 => {
                    let str1 = h2.fr_index.clone();
                    h2.fr_index = self.getstr(xx2, yy, 20, &h2.fr_index, 242).await;
                    h2.fr_index.truncate(3);
                    if !self.s.file_exists(dos_file_name(
                        &[b"FRONT", &h2.fr_index as &[u8], b".PCX"].concat(),
                    )) {
                        self.g.alert_box();
                        self.g.write_font(80, 90, b"INVALID FRONT INDEX VALUE.");
                        self.g.write_font(
                            80,
                            100,
                            &[b"FILE FRONT", &h2.fr_index as &[u8], b".PCX DOESN'T EXIST."]
                                .concat(),
                        );
                        self.wait_for_key3(190, 110).await;
                        h2.fr_index = str1;
                    }
                }
                4 => {
                    let str1 = h2.bk_index.clone();
                    h2.bk_index = self.getstr(xx2, yy, 20, &h2.bk_index, 242).await;
                    h2.bk_index.truncate(3);
                    if !self.s.file_exists(dos_file_name(
                        &[b"BACK", &h2.bk_index as &[u8], b".PCX"].concat(),
                    )) {
                        self.g.alert_box();
                        self.g.write_font(80, 90, b"INVALID BACK INDEX VALUE.");
                        self.g.write_font(
                            80,
                            100,
                            &[b"FILE BACK", &h2.bk_index as &[u8], b".PCX DOESN'T EXIST."].concat(),
                        );
                        self.wait_for_key3(190, 110).await;
                        h2.bk_index = str1;
                    }
                }
                5 => self.getnum(xx2, yy, &mut h2.bk_bright, 0, 255).await,
                6 => self.getnum(xx2, yy, &mut h2.bk_mirror, 0, 1).await,
                7 => {
                    let mut tempb = h2.vx_final.wrapping_sub(40);
                    self.getnum(xx2, yy, &mut tempb, 60, 145).await;
                    h2.vx_final = tempb.wrapping_add(40);
                }
                8 => {
                    let mut temp = round((h2.pk as f64 * 100.0) as f32);
                    self.getint(xx2, yy, &mut temp, 50, 150).await;
                    h2.pk = (temp as f64 / 100.0) as f32;
                }
                9 => {
                    let mut temp = round((h2.pl_save as f64 * 10000.0) as f32) - 3204;
                    self.getint(xx2, yy, &mut temp, 0, 30).await;
                    h2.pl_save = ((temp + 3204) as f64 / 10000.0) as f32;
                }
                10 => {
                    h2.author = self.getstr(xx2, yy, 150, &h2.author, 242).await;
                    h2.author.truncate(30);
                }
                11 => {
                    let str1 = self.getstr2(xx2, yy, 8, filestr, 242).await;
                    if str1 != b" " {
                        *filestr = str1;
                    }

                    //{        if (length(filestr)>8) then filestr:=copy(filestr,1,8); }
                }
                12 => {
                    leave2 = true;
                    *h2 = oldhill.clone();
                    *filestr = b"NULL".to_vec();
                }
                _ => {}
            }

            if leave2 {
                break;
            }
        }
    }

    pub async fn replayinfo(
        &self,
        filename: &mut Vec<u8>,
        author: &mut Vec<u8>,
        name: &mut Vec<u8>,
        nytmaki: i32,
        hp: i32,
    ) -> u8 {
        let mut result = 0;
        let mut index2 = 1;
        let mut leave2 = false;
        let xx = 18;
        let mut yy;

        *name = [b"Huge Jump in " as &[u8], &self.hillname(nytmaki)].concat();
        *filename = b"TEMP".to_vec();

        loop {
            self.g.new_screen(1, 0).await;

            self.g.font_color(240);

            self.g.write_font(
                30,
                6,
                &[
                    &self.l.lstr(25) as &[u8],
                    b": ",
                    &txtp(hp),
                    b"\xab at ",
                    &self.hillname(nytmaki),
                    b" K",
                    &txt(self.hillkr(nytmaki)),
                ]
                .concat(),
            );

            for temp in 1..=5 {
                yy = (temp * 16) + 26;

                if temp < 5 {
                    self.g.font_color(240);
                } else {
                    yy += 16;
                }

                let str1: Vec<u8> = match temp {
                    1..=3 => self.l.lstr(290 + temp as u32).to_vec(),
                    4 | 5 => self.l.lstr(291 + temp as u32).to_vec(),
                    _ => Vec::new(),
                };

                self.g
                    .write_font(xx, yy, &[&txt(temp) as &[u8], b". ", &str1].concat());
                self.g.font_color(246);

                let str1: Vec<u8> = match temp {
                    1 => author.clone(),
                    2 => name.clone(),
                    3 => filename.clone(),
                    _ => Vec::new(),
                };

                self.g.write_font(xx + 130, yy, &str1);
            }

            index2 = self.make_menu(15, 39, 135, 16, 4, index2, 243, 0, 0).await;

            yy = (index2 * 16) + 26;

            match index2 {
                1 => *author = self.getstr(xx + 130, yy, 130, author, 242).await,
                2 => *name = self.getstr(xx + 130, yy, 130, name, 242).await,
                3 => {
                    let str1 = self.getstr2(xx + 130, yy, 8, filename, 242).await;
                    if str1 != b" " {
                        *filename = str1;
                    }
                    //{       if (length(filename)>8) then filename:=copy(filename,1,8); }
                }
                4 => {
                    leave2 = true;
                    result = 1;
                }
                0 => {
                    leave2 = true;

                    if self
                        .s
                        .file_exists(dos_file_name(&[filename as &[u8], b".SJR"].concat()))
                    {
                        self.g.alert_box();

                        self.g.write_font(
                            80,
                            90,
                            &[filename as &[u8], b".SJR ", &self.l.lstr(345)].concat(),
                        );
                        self.g.write_font(
                            80,
                            110,
                            &[
                                &self.l.lstr(346) as &[u8],
                                b" (",
                                &[self.l.lch(6, 1)],
                                b"/",
                                &[self.l.lch(7, 1)],
                                b"):",
                            ]
                            .concat(),
                        );

                        self.getch(190, 110, 243).await;

                        if self.s.get_ch().to_ascii_uppercase() != self.l.lch(6, 1) {
                            leave2 = false;
                            index2 = 3;
                        }
                    }
                }
                _ => {}
            }

            if leave2 {
                break;
            }
        }

        result
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
                        a = self.s.get_ch().to_ascii_uppercase();
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

    async fn write_extra_hill_hr(&self, index: i32) {
        let mut h = Hill::default();
        self.load_info(index, &mut h).await;

        let hd = self.hd.borrow();
        h.hr_name = hd[index as usize].hrname.clone();
        h.hr_len = hd[index as usize].hrlen;
        h.hr_time = hd[index as usize].hrtime.clone();

        let temp = index - NUM_WC_HILLS;

        let mut f1 = self.s.create_file(dos_file_name(
            &[&self.hillfile(temp) as &[u8], b".SJH"].concat(),
        ));
        self.write_hill(&mut f1, h, 0, 1); //{ ainoa mesta jossa phase=1 }
        drop(f1);
    }

    pub fn check_extra_hills(&self) {
        self.num_extra_hills.set(0);

        let mut filenames: Vec<Vec<u8>> = vec![Vec::new(); MAX_EXTRA_HILLS as usize + 1];

        for name in self.s.find_files("*.SJH") {
            if (self.num_extra_hills.get() as i32) < MAX_EXTRA_HILLS {
                self.num_extra_hills.set(self.num_extra_hills.get() + 1);
                let filename = &mut filenames[self.num_extra_hills.get() as usize];

                for c in dos_name_bytes(&name).into_iter().take_while(|&c| c != b'.') {
                    // string[12]
                    if filename.len() < 12 {
                        filename.push(c);
                    }
                }
            }
        }

        let mut f1 = self.s.create_file("MOREHILL.SKI");

        writeln!(f1, "{}", self.num_extra_hills.get()).unwrap();

        for temp in 1..=self.num_extra_hills.get() as usize {
            f1.write_all(&filenames[temp]).unwrap();
            writeln!(f1).unwrap();
        }
    }

    //{ t�ll� ladataan mnimet ja kri -taulut kuntoon }
    pub async fn load_hill_info(&self) {
        {
            let mut f1 = BufReader::new(self.s.open_file("MOREHILL.SKI"));
            self.num_extra_hills.set(parse_line(&mut f1).unwrap());
        }

        for temp in 1..=NUM_WC_HILLS + self.num_extra_hills.get() as i32 {
            let mut now_hill = Hill::default();
            self.load_info(temp, &mut now_hill).await;
            let mut hd = self.hd.borrow_mut();
            hd[temp as usize].name = now_hill.name.clone();
            hd[temp as usize].kr = now_hill.kr;
        }

        self.num_hills
            .set((NUM_WC_HILLS + self.num_extra_hills.get() as i32) as u8);
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

    pub async fn write_extras(&self) {
        for temp in 1..=self.num_extra_hills.get() {
            self.write_extra_hill_hr(temp as i32 + NUM_WC_HILLS).await;
        }
    }

    pub async fn read_extras(&self) {
        for temp in 1..=self.num_extra_hills.get() as i32 {
            let index = temp + NUM_WC_HILLS;

            let mut h = Hill::default();
            self.load_info(index, &mut h).await;

            let mut hd = self.hd.borrow_mut();
            hd[index as usize].hrname = h.hr_name;
            hd[index as usize].hrlen = h.hr_len;
            hd[index as usize].hrtime = h.hr_time;
        }
    }

    pub async fn might_delete(&self, filestr: &[u8]) -> u8 {
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
            self.s.remove_file(dos_file_name(filestr));
            tempb = 0; //{ yep, deleted }
        }

        self.s.clearchs();
        tempb
    }

    pub async fn hill_maker(&self, phase: u8) {
        let show = 18;

        let mut index;
        let mut start = 0; //{ sivulis�ys... }

        let cols = [/* unused */ 0, 100, 160];

        let mut h = Hill::default();
        let mut filestr = Vec::<u8>::new();

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
                    self.load_hill_info().await;
                }
            }

            if index > 0 {
                let mut edit = false;

                if index == next {
                    start += show;
                }
                if index == prev {
                    start -= show;
                }

                if index == create {
                    filestr = [b"NEW", &txt(index + start) as &[u8]].concat();
                    Self::default_hill(&mut h);
                    edit = true;
                    newfile = true;
                }

                if index <= temp2 {
                    //{ uusi }
                    self.load_info(start + index + NUM_WC_HILLS, &mut h).await;
                    filestr = self.hillfile(start + index);
                    edit = true;
                }

                if edit {
                    self.edit_hill(&mut h, &mut filestr, newfile).await;

                    if filestr != b"NULL" {
                        //{ sitten kirjoitetaan p��lle }
                        {
                            let mut f1 = self
                                .s
                                .create_file(dos_file_name(&[&filestr as &[u8], b".SJH"].concat()));
                            self.write_hill(&mut f1, h.clone(), 0, 0); //{ file aukesi ok }
                        }

                        self.g.new_screen(5, 0).await;
                        // The test of IOResult after rewrite clears it, so it is 0 here.
                        self.result_box(3, 0).await;
                    }

                    self.check_extra_hills();
                    self.load_hill_info().await;
                }
            }

            if index == 0 {
                break;
            }
        }
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

    pub async fn teamwarning(&self) {
        // { alertbox; }

        self.g.fill_box(59, 59, 261, 141, 242);
        self.g.fill_box(60, 60, 260, 140, 244);

        self.g.fill_area(60, 60, 260, 140, 63);

        self.g.font_color(246);

        self.g.write_font(80, 72, self.l.lstr(261));
        self.g.write_font(80, 82, self.l.lstr(262));
        self.g.write_font(80, 92, self.l.lstr(263));
        self.g.write_font(80, 112, self.l.lstr(264));
        self.g.write_font(80, 122, self.l.lstr(265));

        self.g.draw_screen().await;

        self.s.wait_for_key().await;
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

    #[allow(dead_code)]
    pub async fn new_unreg_text(&self) {
        self.g.fill_box(130, 120, 310, 120, 9);
        self.g.font_color(240);
        self.g.write_font(150, 130, self.l.lstr(38));
        self.g.write_font(150, 140, self.l.lstr(39));

        let temp = ((random(3) as i32 * 2) + 41) as u16;
        self.g.font_color(246);

        self.g.write_font(150, 155, self.l.lstr(temp as u32));
        self.g.write_font(150, 165, self.l.lstr(temp as u32 + 1));
        self.g.font_color(240);
        self.g.write_font(150, 180, self.l.lstr(40));
        self.g.draw_screen().await;
    }

    pub fn regendtext(&self, _regname: &[u8], _regnumber: &[u8]) {
        gotoxy(2, 10);
        cwrite(8, b"- YOU RULE!!!");
        writeln(b"  This version is registered to Everyone. ");
        textcolor(15);
        cwriteln(8, b"   Thanks for all the support!!!");
    }

    pub fn lopputext(&self, version: &[u8], a1: Time, a2: Time) {
        textcolor(0);
        gotoxy(69, 2);
        write(b" \xdb\xdf \xdb");
        gotoxy(69, 3);
        write(b" \xdb  \xdb");
        gotoxy(69, 4);
        write(b"\xdc\xdb \xdc\xdb");

        textcolor(4);
        gotoxy(75, 2);
        write(b"\xdf\xdb");
        gotoxy(75, 3);
        write(b"\xfe\xdb");
        gotoxy(75, 4);
        write(b"\xdc\xdb");

        let temp1 = a1.second as i32 + a1.minute as i32 * 60 + a1.hour as i32 * 3600;
        let mut temp2 = a2.second as i32 + a2.minute as i32 * 60 + a2.hour as i32 * 3600;

        if temp2 < temp1 {
            temp2 += 86400;
        }

        let diff = temp2 - temp1;

        let hour = diff / 3600;
        let minute = (diff / 60) - (hour * 60);
        let second = diff - (diff / 60) * 60;

        gotoxy(2, 2);
        cwrite(15, &[b"- You played SJ3 v", version, b" for"].concat());

        if hour > 0 {
            write(format!(" {hour} hour").as_bytes());
            if hour > 1 {
                write(b"s");
            }

            if minute > 0 && second > 0 {
                write(b",");
            } else if minute > 0 || second > 0 {
                write(b" and");
            }
        }

        if minute > 0 {
            write(format!(" {minute} minute").as_bytes());
            if minute > 1 {
                write(b"s");
            }
            if second > 0 {
                write(b" and");
            }
        }

        if second > 0 {
            write(format!(" {second} second").as_bytes());
            if second > 1 {
                write(b"s");
            }
        }

        if hour == 0 && minute == 0 && second == 0 {
            write(b" no time at all");
        }
        writeln(b". ");

        gotoxy(2, 4);
        cwriteln(8, b"- Play original DOS version if you aim for the");
        cwrite(8, b"   Universal Hall of Fame.");

        gotoxy(2, 7);
        cwriteln(
            15,
            b"- Be sure to check out the Official Ski Jump International",
        );
        write(b"   webpage located at  ");
        cwrite(8, b"http://www.nomasi.com/sj3/");

        window(1, 16, 80, 19);
        textbackground(1);
        clrscr();
        writeln(b"");
        cwriteln(9, b"   Quote of the moment: ");
        cwrite(15, b"   ");

        match random(40) {
            0 => writeln(b"Drama, excitement, speed, life, Ski Jump International."),
            1 => writeln(b"\"I traded K.I.T.T for Ski Jump International\" - M. Knight"),
            2 => writeln(b"\"Ski Jump International is out there\" - F. Mulder"),
            3 => writeln(b"NSA, CIA, FBI, INTERPOL, SJ3 = ass-kicking action all the way"),
            4 => writeln(b"Ski Jump International - for the love of flying since 1994"),
            5 => writeln(b"\"Only nuclear testing is more fun than SJ3\" - J. Chirac"),
            6 => writeln(
                b"\"Our new phone had SJ3 in it and nobody used it for talking\" - J. Ollila",
            ),
            7 => writeln(b"\"Only exploiting Nigeria is more addictive than SJ3\" - Shell Inc."),
            8 => writeln(b"\"Only exploiting the Amazonas is more fun than SJ3\" - Texaco"),
            9 => writeln(b"\"There shall be no tobacco advertising in SJ3\" - European Union"),
            10 => writeln(b"S.J.3. - four letters that deliver pure enjoyment"),
            11 => writeln(b"Genesis, Exodus, Leviticus, Numeri, Deuterononium, SJ3"),
            12 => writeln(b"\"SJ3 was not the reason for our comeback\" - Spice Girls"),
            13 => writeln(b"\"SJ3 can be explained with modern science\" - D. Scully"),
            14 => writeln(b"\"There is no cleaner sport than SJ3\" - J. Samaranch"),
            15 => writeln(
                b"\"We have warned SJ3 for it's refusal to allow weapons inspectors\" - USA",
            ),
            16 => writeln(
                b"\"SJ3 is a neo-communist plot and it must be stopped at any cost\" - J. Helms",
            ),
            17 => {
                writeln(b"\"Half of USA's 396 Billion Dollar defense budget is spent on SJ3 regs\"")
            }
            18 => writeln(b"\"I made SJ3 out of fertilizer and window cleaner\" - MacGyver"),
            19 => {
                writeln(b"\"Salt Lake City offered $700 000 in bribes to get into SJ3\" - D. Pound")
            }
            20 => {
                writeln(b"\"SJ3 played a key part in our struggle for independence\" - X. Gusmao")
            }
            21 => {
                writeln(b"\"We'll bomb and invade any nation that uses pirate SJ3's!\" - G.W.Bush")
            }
            22 => writeln(b"Engineered with love and tradition from 1994 - SJ3"),
            23 => {
                writeln(b"VX ANTHRAX AK-47 NWO FNORD G-7 NSA SJ3 PGP PRISM BLANKET IRA PLUTONIUM")
            }
            24 => {
                writeln(b"\"SJ3 is the only reason why I still use a Microsoft OS\" - L. Torvalds")
            }
            25 => writeln(b"\"SJ3 is banned in China for it's pro-Tibetian views\" - J. Zemin"),
            26 => writeln(b"\"SJ3 is the last beacon of free press in Russia\" - V. Guzinsky"),
            27 => writeln(b"SJ3 - no sex, no drugs, no violence - just the love of flying"),
            28 => writeln(b"\"I am still the greatest, but SJ3 is getting damn close\" - M. Ali"),
            29 => writeln(b"\"I pity the fool who doesn't play SJ3\" - Mr.T"),
            30 => writeln(b"S is for Serenity, J is for Joy and 3 is for ... a Free Palestine!"),
            31 => writeln(
                b"\"We didn't join the Euro as we can buy SJ3 with any currency\" - DK, SE & UK",
            ),
            32 => writeln(b"H2O, CO2, O2, SJ3 - stuff I can't live without"),
            33 => writeln(b"Sex, drugs & SJ3"),
            34 => writeln(
                b"\"The downloads of SJ3 are monitored because of its suspicious content\" - NSA",
            ), //{ max length }
            35 => writeln(
                b"SJ3 - almost as addictive as Coca-Cola and almost as fattening as McDonald's",
            ),
            36 => writeln(b"\"SJ3 is more magic than me\" - D. Blaine"),
            37 => writeln(
                b"\"New Europe doesn't need democratic decisions, it needs SJ3.\" - D. Rumsfeld",
            ),
            38 => {
                writeln(b"\"As soon as I own SJ3, it will start to love me too.\" - S. Berlusconi")
            }
            39 => writeln(
                b"\"I have never overstated anyones weapons or games.  SJ3 is god.\" - T. Blair",
            ),
            40 => {
                writeln(b"\"SJ3 inspired me to admit that we are illegal occupiers.\" - A. Sharon")
            }
            _ => {}
        }

        textcolor(7);

        window(1, 1, 80, 25);

        gotoxy(1, 22);
    }
}

pub fn loadgoal<P: Platform>(s: &P, num: i32) -> i32 {
    let mut value: i32 = 0;

    if s.file_exists("GOALS.SKI") && num > 0 && num <= NUM_WC_HILLS {
        let mut f2 = BufReader::new(s.open_file("GOALS.SKI"));
        for _ in 1..=num {
            value = read_goal(&mut f2);
        }
    }
    value
}

fn load_goals<P: Platform>(s: &P, goals: &mut [i32; NUM_WC_HILLS as usize + 1]) {
    if s.file_exists("GOALS.SKI") {
        let mut f2 = BufReader::new(s.open_file("GOALS.SKI"));
        for temp2 in 1..=NUM_WC_HILLS as usize {
            goals[temp2] = read_goal(&mut f2);
        }
    }
}

/// `readln(f2, value)`: skips blanks and empty lines, reads the number up to
/// the first character that is not a digit, and gives 0 at the end of the file
fn read_goal(f2: &mut BufReader<impl Read>) -> i32 {
    loop {
        let mut line = Vec::new();
        if f2.read_until(b'\n', &mut line).unwrap_or(0) == 0 {
            return 0;
        }
        let text = line.trim_ascii_start();
        if !text.is_empty() {
            let sign = usize::from(matches!(text[0], b'+' | b'-'));
            let digits = text[sign..]
                .iter()
                .take_while(|c| c.is_ascii_digit())
                .count();
            return val(&text[..sign + digits]);
        }
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

pub fn dayandtime(d1: Date, t1: Time) -> Vec<u8> {
    static DAYS: [&[u8]; 7] = [b"Sun", b"Mon", b"Tue", b"Wed", b"Thu", b"Fri", b"Sat"];
    static MONTHS: [&[u8]; 12] = [
        b"Jan", b"Feb", b"Mar", b"Apr", b"May", b"Jun", b"Jul", b"Aug", b"Sep", b"Oct", b"Nov",
        b"Dec",
    ];

    return [
        DAYS[d1.day_of_week as usize],
        b" ",
        if d1.day < 10 {
            b" " as &[u8]
        } else {
            b"" as &[u8]
        },
        &txt(d1.day as i32),
        b" ",
        MONTHS[(d1.month - 1) as usize],
        b" ",
        &txt(d1.year as i32),
        b" ",
        if t1.hour < 10 {
            b" " as &[u8]
        } else {
            b"" as &[u8]
        },
        &txt(t1.hour as i32),
        b":",
        if t1.minute < 10 {
            b"0" as &[u8]
        } else {
            b"" as &[u8]
        },
        &txt(t1.minute as i32),
    ]
    .concat();
}

pub fn crypt(arvo: i32, jarj: i32) -> Vec<u8> {
    let mut str1 = txt(arvo);

    if str1.len() < 5 {
        for _ in 1..=5 - str1.len() {
            str1.insert(0, b'0');
        }
    }

    let mut str2: Vec<u8> = Vec::new();

    for index in (1..=5).rev() {
        str2.push(str1[index - 1] + 21);
    }

    // Pascal assigns one character to the short string str1 below, so
    // str1[6] still has the sixth digit of arvo.
    let digits = str1;

    let mut str1 = vec![(68 + 2 * ((arvo % 7) ^ 1)) as u8]; //{ tarkistusluku arvosta }
    insert(&str1, &mut str2, 2);
    str1 = vec![(65 + ((jarj ^ 33) % 19)) as u8]; //{ tarkistusluku j�rjestyksest� }
    insert(&str1, &mut str2, 3);
    str1 = vec![(68 + random(6)) as u8]; //{ ihan vaan satunnainen kirjain }
    if txt(arvo).len() > 5 {
        str1 = vec![digits[6 - 1] + 27];
    }
    insert(&str1, &mut str2, 5);
    // {  insert('*',str2,5); }

    str2
}

pub fn uncrypt(mut str0: Vec<u8>, jarj: i32) -> i32 {
    // All the "- 1" are because the original code uses 1-based string indexing

    let high = char_at(&str0, 5 - 1); //{ satunnainen kirjain, tai yli 100 000 }
    delete(&mut str0, 5 - 1); //{ pois }
    let chk1 = char_at(&str0, 2 - 1);
    delete(&mut str0, 2 - 1);
    let chk2 = char_at(&str0, 2 - 1);
    delete(&mut str0, 2 - 1);

    let mut str1: Vec<u8> = Vec::new();

    for index in (1..=5).rev() {
        str1.push(char_at(&str0, index - 1).wrapping_sub(21));
    }

    let mut luku1: i32 = val(&str1);

    if high > 74 {
        //{ ei pelkkä satunnaiskirjain }
        luku1 += 100000 * (high - 75) as i32;
    }

    str1[1 - 1] = (68 + 2 * ((luku1 % 7) ^ 1)) as u8;
    if str1[1 - 1] != chk1 {
        luku1 = 0;
    }
    str1[1 - 1] = (65 + ((jarj ^ 33) % 19)) as u8;
    if str1[1 - 1] != chk2 {
        luku1 = 0;
    }

    luku1
}

/// A character past the end of a Pascal string is undefined. This gives 0.
fn char_at(str0: &[u8], index: usize) -> u8 {
    str0.get(index).copied().unwrap_or(0)
}

/// `Delete(str0, index + 1, 1)` of Pascal, which does nothing past the end
fn delete(str0: &mut Vec<u8>, index: usize) {
    if index < str0.len() {
        str0.remove(index);
    }
}

/// `Insert(source, s, index)` of Pascal, which appends past the end
pub fn insert(source: &[u8], s: &mut Vec<u8>, index: usize) {
    let at = index.clamp(1, s.len() + 1) - 1;
    s.splice(at..at, source.iter().copied());
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

pub fn svaluestr(str0: &[u8], arvo: i32) -> u16 {
    let mut word1: u16 = 0;

    for index in 1..=str0.len() {
        match str0[index - 1] {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' => {
                word1 = word1.wrapping_add(
                    (str0[index - 1] as i32).wrapping_mul(((index as i32) % 5).wrapping_add(41))
                        as u16,
                ); //{ v3.10 }
            }
            _ => word1 = word1.wrapping_add(3),
        }
    }

    word1
        .wrapping_mul((arvo % 7 + 1) as u16)
        .wrapping_add(arvo as u16)
}

#[cfg(test)]
mod tests {
    use super::{crypt, read_goal, uncrypt};
    use std::io::BufReader;

    // The expected strings come from the Pascal crypt and uncrypt, built with
    // FPC -Mtp, with random(6) replaced by 0. The fifth character is random
    // when arvo has at most 5 digits.
    fn assert_crypt(arvo: i32, jarj: i32, expected: &[u8], uncrypted: i32) {
        let mut str1 = crypt(arvo, jarj);
        assert_eq!(str1.len(), 8);
        if arvo < 100000 {
            assert!((b'D'..=b'I').contains(&str1[4]));
            str1[4] = b'D';
        }
        assert_eq!(str1, expected);
        assert_eq!(uncrypt(str1, jarj), uncrypted);
    }

    #[test]
    fn crypt_round_trip() {
        assert_crypt(0, 1, b"EFNEDEEE", 0);
        assert_crypt(7, 5, b"LFREDEEE", 7);
        assert_crypt(1515, 1, b"JHNFDJFE", 1515);
        assert_crypt(42, 21, b"GFOIDEEE", 42);
        assert_crypt(12345, 3, b"JNPIDHGF", 12345);
        assert_crypt(99999, 41, b"NNINDNNN", 99999);
    }

    #[test]
    fn crypt_more_than_5_digits() {
        // crypt keeps the first 6 digits, and uncrypt reads the sixth digit as
        // the hundred thousands. The check then fails and gives 0.
        assert_crypt(123456, 7, b"JNAIQHGF", 0);
        assert_crypt(100000, 2, b"ELQEKEEF", 0);
        assert_crypt(1234567, 9, b"JLCIQHGF", 0);
    }

    #[test]
    fn read_goal_like_readln() {
        let mut f2 = BufReader::new(&b"12 \r\n\r\n  -7x\r\n"[..]);
        assert_eq!(read_goal(&mut f2), 12);
        assert_eq!(read_goal(&mut f2), -7);
        assert_eq!(read_goal(&mut f2), 0);
    }
}
