use crate::graph::GraphModule;
use crate::help::{beep, nmstr, round, toansi, txt, txtp, upstr};
use crate::lang::LangModule;
use crate::pcx::PcxModule;
use crate::platform::Platform;
use crate::rs_util::{dos_file_name, dos_name_bytes, pad_left, parse_line, random, read_line};
use crate::unit::{
    crypt, dayandtime, insert, svaluestr, valuestr, Date, Hiscore, Time, UnitModule, MAX_CUSTOMS,
    NUM_PL, NUM_TEAMS, NUM_WC_HILLS,
};
use std::cell::{Cell, RefCell};
use std::io::{BufReader, Write};

pub struct Profile {
    name: Vec<u8>,
    realname: Vec<u8>,
    pub suitcolor: u8,
    pub skicolor: u8,
    pub kothlevel: u8,
    pub replace: u8,
    pub bestwchill: u8,
    pub besthill: u8,
    pub cstyle: u8, // coach style?
    pub skipquali: u8,
    pub bestwcjump: u16,
    pub bestpoints: u16,
    pub best4points: u16,
    pub bestjump: u16,
    pub besthillfile: Vec<u8>,
    pub bestresult: Vec<u8>,
    pub best4result: Vec<u8>,
    pub wcs: i32,
    pub legswon: i32,
    pub wcswon: i32,
    pub totaljumps: i32,
}

impl Profile {
    fn new() -> Self {
        Self {
            name: b"SKI JUMPER".to_vec(),
            realname: b"".to_vec(),
            suitcolor: 0,
            skicolor: 0,
            kothlevel: 0,
            replace: 0,
            cstyle: 1,
            wcs: 0,
            legswon: 0,
            wcswon: 0,
            bestwcjump: 0,
            bestwchill: 1,
            bestjump: 0,
            besthill: 1,
            besthillfile: b"HILLBASE".to_vec(),
            bestresult: b"-".to_vec(),
            best4result: b"-".to_vec(),
            totaljumps: 0,

            skipquali: 0,
            bestpoints: 0,
            best4points: 0,
        }
    }
}

const NUM_HISCORES: u32 = 61; //{ 1-20=WC + 21-30=TC + 31-35=4h + 36-41=KOTH + 42-61=CC }
                              //{$IFDEF REG}
const MAX_CUSTOM_HILLS: i32 = 40;
// {$ELSE}
// const MAX_CUSTOM_HILLS = 5;
// {$ENDIF}

//{$IFDEF REG}
const MAX_PMAARA: u8 = 10;
const MAX_PROFILES: u8 = 20;
//{$ELSE}
//const MAX_PMAARA: u8 = 2;
//const MAX_PROFILES: u8 = 4;
//{$ENDIF}

pub struct InfoModule<'g, 'l, 'm, 'p, 's, 'u, P: Platform> {
    g: &'g GraphModule<'m, 'p, 's, P>,
    l: &'l LangModule<'s, P>,
    p: &'p PcxModule<'m, 's, P>,
    s: &'s P,
    u: &'u UnitModule<'g, 'l, 'm, 'p, 's, P>,
    pub top: RefCell<[Hiscore; NUM_HISCORES as usize + 1]>,
    pub nimet: RefCell<Vec<Vec<u8>>>,
    pub jnimet: RefCell<Vec<Vec<u8>>>,
    pub pmaara: Cell<u8>,
    num_profiles: Cell<u8>, //{ piilota alas }
    pub profileorder: RefCell<[u8; 21]>,
    pub profile: RefCell<Vec<Profile>>, //{ ehk� my�s }

    maxprofiles: i32,
    maxpmaara: i32,
}

impl<'g, 'l, 'm, 'p, 's, 'u, P: Platform> InfoModule<'g, 'l, 'm, 'p, 's, 'u, P> {
    pub fn new(
        g: &'g GraphModule<'m, 'p, 's, P>,
        l: &'l LangModule<'s, P>,
        p: &'p PcxModule<'m, 's, P>,
        s: &'s P,
        u: &'u UnitModule<'g, 'l, 'm, 'p, 's, P>,
    ) -> Self {
        let profiles = (0..=20).map(|_| Profile::new()).collect::<Vec<_>>();
        InfoModule {
            g,
            l,
            p,
            s,
            u,
            top: RefCell::new(std::array::from_fn(|_| Hiscore::default())),
            nimet: RefCell::new((0..=NUM_PL + 1).map(|_| b"".to_vec()).collect()),
            jnimet: RefCell::new((0..=NUM_TEAMS).map(|_| b"".to_vec()).collect()),
            pmaara: Cell::new(1),
            num_profiles: Cell::new(1),
            profileorder: RefCell::new([0; 21]),
            profile: RefCell::new(profiles),
            maxprofiles: 0,
            maxpmaara: 0,
        }
    }

    pub fn load_custom(
        &self,
        setfile: &[u8],
        sortby: &mut u8,
        hillorder: &mut [i32],
        cuphills: &mut i32,
    ) -> i32 {
        // TODO
        todo!();
    }

    pub fn write_custom(
        &self,
        setfile: &[u8],
        sortby: u8,
        hillorder: &[i32],
        cuphills: i32,
    ) -> i32 {
        // TODO
        0
    }

    pub async fn newcrecordscreen(
        &self,
        setfile: &[u8],
        newhi: &Hiscore,
        oldhi: &Hiscore,
        sortby: u8,
    ) {
        let mut tempb = oldhi.score > 0;
        let mut str1: Vec<u8>;

        self.g.fill_box(54, 59, 276, 135, 248);
        self.g.fill_box(55, 60, 275, 134, 243);

        self.g.font_color(246);
        self.g.write_font(75, 69, self.l.lstr(128));
        str1 = if sortby == 1 {
            txtp(newhi.score)
        } else {
            txt(newhi.score)
        };

        self.g.e_write_font(242, 80, &str1);

        self.g.font_color(240);
        self.g.write_font(85, 80, &self.g.nsh(&newhi.name, 90));
        self.g
            .e_write_font(200, 80, &[&txt(newhi.pos as i32) as &[u8], b"."].concat());

        self.g.font_color(241);
        self.g.write_font(95, 87, &newhi.time);
        self.g.write_font(75, 100, self.l.lstr(129));

        if tempb {
            str1 = if sortby == 1 {
                txtp(oldhi.score)
            } else {
                txt(oldhi.score)
            };
            self.g.write_font(85, 110, &self.g.nsh(&oldhi.name, 90));
            self.g
                .e_write_font(200, 110, &[&txt(oldhi.pos as i32) as &[u8], b"."].concat());
            self.g.e_write_font(242, 110, &str1);
            self.g.write_font(95, 117, &oldhi.time);
        } else {
            self.g.write_font(85, 110, b"-");
        }
        self.g.draw_screen().await;
        self.s.wait_for_key().await;
    }

    fn find_customs(&self, num_customs: &mut i32, filenames: &mut [Vec<u8>]) {
        *num_customs = 0;
        for temp in 0..=MAX_CUSTOMS as usize {
            filenames[temp] = b"".to_vec();
        }

        for name in self.s.find_files("*.SJC") {
            if *num_customs < MAX_CUSTOMS {
                *num_customs += 1;

                let name = dos_name_bytes(&name);
                let dot = name.iter().position(|&c| c == b'.').unwrap_or(name.len());
                // FileStr is string[8]
                filenames[*num_customs as usize] = name[..dot.min(8)].to_vec();

                if filenames[*num_customs as usize] == b"TEMP" {
                    filenames[*num_customs as usize] = b"".to_vec();
                    *num_customs -= 1;
                }
            }
        }
    }

    pub fn load_names(&self, whatset: u8, teamdiscount: u8, teamlineup: &mut [u8], replaces: bool) {
        let mut l1: i32 = 0;
        let mut s = b"".to_vec();

        let profile = self.profile.borrow();
        let mut nimet = self.nimet.borrow_mut();

        let mut f4 = BufReader::new(self.s.open_file(dos_file_name(
            &[b"NAMES" as &[u8], &[whatset], b".SKI"].concat(),
        )));

        read_line(&mut f4).unwrap(); //{ titlerivi pois }

        let mut temp2: i32 = 0; //{ toteutuneet nimet }
        for temp in 1..=NUM_PL as u8 + 1 {
            s = read_line(&mut f4).unwrap();
            let mut ok = true; //{ k�yk� nimi eli ei kai ole replace-listoilla }
            for temp3 in 1..=self.pmaara.get() {
                if profile[self.profileorder.borrow()[temp3 as usize] as usize].replace == temp
                    && replaces
                {
                    ok = false;
                }
            }
            l1 += valuestr(&s, (temp % 43) as i32) as i32;
            if ok {
                temp2 += 1;
                nimet[temp2 as usize] = s;
            }
        }

        if nimet[76].is_empty() {
            nimet[76] = b"Trainee".to_vec();
        }

        for temp in 1..=self.pmaara.get() {
            //{ siirret��n profilesta nimet }
            nimet[NUM_PL + 1 - temp as usize] = profile
                [self.profileorder.borrow()[temp as usize] as usize]
                .name
                .clone();
        }

        read_line(&mut f4).unwrap(); //{ tyhj� rivi, ehk� koodi }

        for temp in 1..=NUM_TEAMS - 1 {
            s = read_line(&mut f4).unwrap();
            l1 += valuestr(&s, (temp % 31) as i32) as i32;

            for temp2 in 1..=4 {
                let mut tempb: u8 = parse_line(&mut f4).unwrap();
                if tempb > 67 {
                    tempb = 1;
                }
                l1 += (tempb % temp2) as i32;

                if temp < NUM_TEAMS + 1 - teamdiscount as usize {
                    self.jnimet.borrow_mut()[temp] = s.clone();
                    teamlineup[(temp - 1) * 4 + temp2 as usize] = tempb;
                }
            }
        }

        if whatset == b'0' {
            let l2: i32 = parse_line(&mut f4).unwrap(); //{ haetaan se names:n checksum }
            l1 += 5661;
            if l1 != l2 {
                panic!("Error #92: Something wrong with the namefile. Shit.");
            }
        }
    }

    pub fn reset_tops(&self, kerroin: u8, today: Date, now: Time) {
        let mut top = self.top.borrow_mut();
        let nimet = self.nimet.borrow();
        let jnimet = self.jnimet.borrow();

        top[1].score = 1515;
        top[1].pos = 1;
        top[2].score = 1343;
        top[2].pos = 1;
        top[3].score = 1212;
        top[3].pos = 2;
        top[4].score = 1101;
        top[4].pos = 3;
        top[5].score = 999;
        top[5].pos = 4;
        top[6].score = 888;
        top[6].pos = 5;
        top[7].score = 777;
        top[7].pos = 6;
        top[8].score = 665;
        top[8].pos = 7;
        top[9].score = 555;
        top[9].pos = 8;
        top[10].score = 494;
        top[10].pos = 10;
        top[11].score = 444;
        top[11].pos = 12;
        top[12].score = 383;
        top[12].pos = 14;
        top[13].score = 333;
        top[13].pos = 16;
        top[14].score = 272;
        top[14].pos = 18;
        top[15].score = 222;
        top[15].pos = 20;
        top[16].score = 161;
        top[16].pos = 22;
        top[17].score = 111;
        top[17].pos = 25;
        top[18].score = 77;
        top[18].pos = 30;
        top[19].score = 33;
        top[19].pos = 35;
        top[20].score = 11;
        top[20].pos = 40;

        for temp in 1..=20 {
            top[temp].name = [&nimet[top[temp].pos as usize] as &[u8], b"\xff"].concat();
            top[temp].time = dayandtime(today, now);
        }

        for temp in 1..=NUM_WC_HILLS {
            let name = [
                &nimet[1 + random(6 + random(10)) as usize] as &[u8],
                b"\xff",
            ]
            .concat();
            let len = round((self.u.hillkr(temp) as f64 / 0.94) as f32) * 10 * kerroin as i32;
            let time = dayandtime(today, now);
            self.u.set_hrinfo(temp, name, len, time);
        }

        if self.u.num_extra_hills.get() > 0 {
            for temp in 1..=self.u.num_extra_hills.get() as i32 {
                let name = [
                    &nimet[1 + random(6 + random(10)) as usize] as &[u8],
                    b"\xff",
                ]
                .concat();
                let len = round((self.u.hillkr(temp + NUM_WC_HILLS) as f64 / 0.94) as f32)
                    * 10
                    * kerroin as i32;
                let time = dayandtime(today, now);
                self.u.set_hrinfo(temp + NUM_WC_HILLS, name, len, time);
            }
        }

        for temp in 21..=30 {
            top[temp].pos = round(((temp - 20) * 1) as f32) as u8;
            top[temp].name = [&jnimet[top[temp].pos as usize] as &[u8], b"\xff"].concat();
            top[temp].time = dayandtime(today, now);
        }

        top[21].score = 42;
        top[22].score = 35;
        top[23].score = 28;
        top[24].score = 21;
        top[25].score = 17;
        top[26].score = 11;
        top[27].score = 8;
        top[28].score = 5;
        top[29].score = 3;
        top[30].score = 1;

        top[31].score = 10111;
        top[31].pos = 1;
        top[32].score = 9876;
        top[32].pos = 3;
        top[33].score = 9449;
        top[33].pos = 6;
        top[34].score = 9010;
        top[34].pos = 10;
        top[35].score = 8321;
        top[35].pos = 15;

        for temp in 31..=35 {
            top[temp].name = [&nimet[top[temp].pos as usize] as &[u8], b"\xff"].concat();
            top[temp].time = dayandtime(today, now);
        }

        for temp in 1..=35 {
            //{ katsos jos nollaa ne scoret! }
            top[temp].pos = top[temp].pos.wrapping_mul(kerroin);
            top[temp].score = top[temp].score * kerroin as i32;
        }

        for temp in 36..=41 {
            top[temp].name = b"Not Completed".to_vec();
            top[temp].pos = 0;
            top[temp].score = 0;
            top[temp].time = dayandtime(today, now);
        }
    }

    fn default_profile(&self, index: i32, phase: u8) {
        let mut profiles = self.profile.borrow_mut();
        profiles[index as usize] = Profile::new();

        let mut count: u8;
        let mut count2 = 2u8;
        if phase == 1 {
            loop {
                let mut ok = true;
                count = 0;
                for temp in 1..=self.num_profiles.get() as i32 {
                    if profiles[temp as usize].name == profiles[index as usize].name
                        && temp != index
                    {
                        count = count.wrapping_add(1);
                    }
                }
                if count > 0 {
                    profiles[index as usize].name = format!("SKI JUMPER {}", count2).into_bytes();
                    ok = false;
                }
                count2 += 1;
                if count2 > 200 {
                    ok = true; //{ jos se dorka j�� rundiin }
                }
                if ok {
                    break;
                }
            }
        }
    }

    pub fn load_profiles(&self) {
        let mut f1 = BufReader::new(self.s.open_file("PLAYERS.SKI"));

        for temp in 0..=20 {
            self.default_profile(temp, 0);
        }

        self.num_profiles.set(parse_line(&mut f1).unwrap());
        if self.num_profiles.get() == 0 || self.num_profiles.get() > MAX_PROFILES {
            self.num_profiles.set(1);
        }

        for temp in 1..=self.num_profiles.get() {
            let l1: i32;
            let code: i32;
            {
                let mut profiles = self.profile.borrow_mut();
                let mut str1 = vec![1];
                while str1.is_empty() || str1[0] != b'*' {
                    str1 = read_line(&mut f1).unwrap();
                }

                let profile = &mut profiles[temp as usize];
                profile.name = read_line(&mut f1).unwrap();
                profile.suitcolor = parse_line(&mut f1).unwrap();
                profile.skicolor = parse_line(&mut f1).unwrap();
                profile.cstyle = parse_line(&mut f1).unwrap();
                profile.kothlevel = parse_line(&mut f1).unwrap();
                profile.replace = parse_line(&mut f1).unwrap();
                profile.wcs = parse_line(&mut f1).unwrap();
                profile.legswon = parse_line(&mut f1).unwrap();
                profile.wcswon = parse_line(&mut f1).unwrap();
                profile.bestwcjump = parse_line(&mut f1).unwrap();
                profile.bestwchill = parse_line(&mut f1).unwrap();
                profile.bestjump = parse_line(&mut f1).unwrap();
                profile.besthill = parse_line(&mut f1).unwrap();
                profile.besthillfile = read_line(&mut f1).unwrap();
                profile.bestresult = read_line(&mut f1).unwrap();
                profile.bestpoints = parse_line(&mut f1).unwrap();
                profile.best4result = read_line(&mut f1).unwrap();
                profile.best4points = parse_line(&mut f1).unwrap();
                profile.totaljumps = parse_line(&mut f1).unwrap();
                profile.skipquali = parse_line(&mut f1).unwrap();
                profile.realname = read_line(&mut f1).unwrap();
                if profile.realname == b"0" {
                    profile.realname = profile.name.clone();
                }
                read_line(&mut f1).unwrap();
                read_line(&mut f1).unwrap();
                l1 = parse_line(&mut f1).unwrap(); //{ checksum }
                code = profile_code(profile);
            }

            if l1 != code {
                beep(self.s, 1);
                self.default_profile(temp as i32, 0);
            }
            /*
            {$IFNDEF REG}
                 with Profile[temp] do
                  begin
                   suitcolor:=0;
                   skicolor:=0;
                   replace:=0;
                  end;
            {$ENDIF}
            */
        }
    }

    pub fn write_profiles(&self) {
        let mut f1 = self.s.create_file("PLAYERS.SKI");

        writeln!(f1, "{}", self.num_profiles.get()).unwrap();
        if self.num_profiles.get() == 0 || self.num_profiles.get() > 20 {
            self.num_profiles.set(1);
        }

        let profiles = self.profile.borrow();
        for temp in 1..=self.num_profiles.get() {
            f1.write_all(&[b"*" as &[u8], &txt(temp as i32)].concat())
                .unwrap();
            writeln!(f1).unwrap();

            let profile = &profiles[temp as usize];
            f1.write_all(&profile.name).unwrap();
            writeln!(f1).unwrap();
            writeln!(f1, "{}", profile.suitcolor).unwrap();
            writeln!(f1, "{}", profile.skicolor).unwrap();
            writeln!(f1, "{}", profile.cstyle).unwrap();
            writeln!(f1, "{}", profile.kothlevel).unwrap();
            writeln!(f1, "{}", profile.replace).unwrap();
            writeln!(f1, "{}", profile.wcs).unwrap();
            writeln!(f1, "{}", profile.legswon).unwrap();
            writeln!(f1, "{}", profile.wcswon).unwrap();
            writeln!(f1, "{}", profile.bestwcjump).unwrap();
            writeln!(f1, "{}", profile.bestwchill).unwrap();
            writeln!(f1, "{}", profile.bestjump).unwrap();
            writeln!(f1, "{}", profile.besthill).unwrap();
            f1.write_all(&profile.besthillfile).unwrap();
            writeln!(f1).unwrap();
            f1.write_all(&profile.bestresult).unwrap();
            writeln!(f1).unwrap();
            writeln!(f1, "{}", profile.bestpoints).unwrap();
            f1.write_all(&profile.best4result).unwrap();
            writeln!(f1).unwrap();
            writeln!(f1, "{}", profile.best4points).unwrap();
            writeln!(f1, "{}", profile.totaljumps).unwrap();
            writeln!(f1, "{}", profile.skipquali).unwrap();
            f1.write_all(&profile.realname).unwrap();
            writeln!(f1).unwrap();
            writeln!(f1, "0").unwrap();
            writeln!(f1, "0").unwrap();

            writeln!(f1, "{}", profile_code(profile)).unwrap();
            writeln!(f1).unwrap();
            writeln!(f1).unwrap();
        }
    }

    pub async fn welcome_screen(&self, languagenumber: &mut u8) {
        let full = *languagenumber == 255;

        if *languagenumber == 255 {
            *languagenumber = 1;
        }

        {
            let mut profile = self.profile.borrow_mut();
            for temp in 1..=5 {
                profile[temp as usize].cstyle = 1;
            }
        }

        if full {
            let x = 240;

            self.g.new_screen(6, 0).await;

            self.g.font_color(240);
            self.g.e_write_font(x, 6, b"WELCOME!");
            self.g.font_color(246);
            self.g.e_write_font(x, 16, b"TERVETULOA!");
            self.g.font_color(247);
            self.g.e_write_font(x, 26, b"WILLKOMMEN!");
            self.g.font_color(240);
            self.g.e_write_font(x, 36, b"V\x8eLKOMMEN!");
        } else {
            self.g.fill_box(74, 41, 246, 186, 248);
            self.g.fill_box(75, 42, 245, 185, 243);
        }

        let x = 100;

        self.g.font_color(240);
        self.g.write_font(x, 50, b"PLEASE CHOOSE A LANGUAGE:");

        let x = 155;

        self.g.font_color(246);

        for temp in 1..=self.l.num_languages() {
            self.g.write_font(
                x - (self.g.font_len(&self.l.lnames[temp - 1]) / 2),
                temp as i32 * 8 + 55,
                &self.l.lnames[temp - 1],
            );
        }

        let temp = if full { 3 } else { 7 };
        let index = self
            .u
            .make_menu(
                112,
                64,
                100,
                8,
                (self.l.num_languages() - 1) as i32,
                1,
                243,
                temp,
                0,
            )
            .await;

        let index = if index == 0 {
            self.l.num_languages() as i32
        } else {
            index
        };

        *languagenumber = index as u8;

        self.l.load_language(*languagenumber);
    }

    pub async fn choose_seecomps(&self, seecomps: &mut u8) {
        self.g.font_color(240);

        self.g.fill_box(74, 79, 246, 133, 248);
        self.g.fill_box(75, 80, 245, 132, 243);

        self.g.write_font(85, 85, self.l.lstr(219));
        self.g.font_color(241);
        self.g.write_font(85, 95, self.l.lstr(150));

        let mut index = *seecomps;
        let mut sch1: u8 = 0;
        let mut sch2: u8 = 0;

        loop {
            let (str1, color) = if index as usize > NUM_PL {
                (self.l.lstr(index as u32).to_vec(), 240)
            } else {
                (
                    [
                        b"#" as &[u8],
                        &txt(index as i32),
                        b" ",
                        &self.g.nsh(&self.nimet.borrow()[index as usize], 115),
                    ]
                    .concat(),
                    246,
                )
            };

            self.g.font_color(color);
            self.g.fill_box(85, 105, 235, 125, 245);
            self.g.write_font(95, 112, &str1);

            self.g.draw_screen().await;
            (sch1, sch2) = self.s.wait_for_key_press().await;

            if sch1 == b'+' || sch2 == 77 || sch2 == 80 {
                index += 1;
                if index as usize > NUM_PL - 11 && index < 235 {
                    index = 235;
                }
                if index > 240 {
                    index = 1;
                }
            }

            if sch1 == b'-' || sch2 == 75 || sch2 == 72 {
                index -= 1;
                if index < 235 && index as usize > NUM_PL {
                    index = (NUM_PL - 11) as u8;
                }
                if index < 1 {
                    index = 240;
                }
            }

            if sch2 == 71 {
                index = 1;
            }

            if sch2 == 79 {
                index = 240;
            }

            if sch1 == 27 || sch2 == 68 || sch1 == 13 {
                break;
            }
        }

        if sch1 == 13 {
            *seecomps = index;
        }
    }

    pub fn call_sendme(
        &self,
        version: &[u8],
        regname: &[u8],
        regnumber: &[u8],
        today: Date,
        now: Time,
        vcode: u8,
    ) {
        let mut str1: Vec<u8> = Vec::new();
        let mut sortby: u8 = 0;
        let mut anywritten: bool;
        let mut apu2: i32;
        let mut laskuri: i32;
        let mut ahi: i32;
        let mut code: i32;
        let mut num_customs: i32 = 0;
        let mut files: Vec<Vec<u8>> = vec![Vec::new(); MAX_CUSTOMS as usize + 1];
        let mut temphillorder = [0i32; 42];
        let mut checkstrs: [Vec<u8>; 22] = Default::default();
        let mut tempcuphills: i32 = 0;

        let dotop = |fname: &str| {
            let mut f2 = self.s.create_file(fname);

            writeln_bytes(
                &mut f2,
                &[
                    b"SJ3 v",
                    version,
                    b" - ",
                    fname.as_bytes(),
                    b" - created at ",
                    &dayandtime(today, now),
                ],
            );
            writeln!(f2).unwrap();
            writeln!(
                f2,
                "            (please fill in the real full names below!)"
            )
            .unwrap();
            writeln_bytes(
                &mut f2,
                &[&toansi(
                    b"            (example: \"LARS76\" = Ville K\x94n\x94nen )",
                )],
            );
            writeln_bytes(
                &mut f2,
                &[&toansi(
                    b"            (         \"VILLE KONONEN\" = Ville K\x94n\x94nen )",
                )],
            );
            writeln!(f2, ".:Jumpers:.").unwrap();
            writeln!(f2).unwrap();
            f2
        };

        //{ 1-sendme, 2-extras, 3-customs }
        let findnames = |f2: &mut dyn Write,
                         phase: u8,
                         str1: &mut Vec<u8>,
                         sortby: &mut u8,
                         temphillorder: &mut [i32; 42],
                         tempcuphills: &mut i32,
                         num_customs: i32,
                         files: &[Vec<u8>]| {
            let mut writtenstr: Vec<Vec<u8>> = vec![Vec::new(); 201];
            let mut numwritten: usize = 0;
            let stop: i32;
            let mut stop2: i32 = 1;

            match phase {
                1 => stop = 40 + self.num_profiles.get() as i32,
                2 => stop = self.u.num_extra_hills.get() as i32,
                _ => {
                    stop = num_customs;
                    stop2 = 20;
                }
            }

            for temp1 in 1..=stop {
                if phase == 3 {
                    self.load_custom(&files[temp1 as usize], sortby, temphillorder, tempcuphills);
                }

                for temp2 in 1..=stop2 {
                    let mut ok = true;
                    match phase {
                        1 => match temp1 {
                            1..=20 => *str1 = self.top.borrow()[temp1 as usize].name.clone(),
                            21..=40 => *str1 = self.u.hrname(temp1 - 20),
                            41..=99 => {
                                *str1 = self.profile.borrow()[(temp1 - 40) as usize].name.clone()
                            }
                            _ => {}
                        },
                        2 => *str1 = self.u.hrname(temp1 + 20),
                        _ => *str1 = self.top.borrow()[(temp2 + 41) as usize].name.clone(),
                    }

                    for temp3 in 1..=numwritten {
                        if *str1 == writtenstr[temp3] {
                            ok = false; //{ on jo }
                        }
                    }

                    if str1.last() == Some(&b'\xff') {
                        ok = false; //{ computer jumper }
                    }
                    if *str1 == b"Nobody" && phase == 3 {
                        ok = false; //{ customs default }
                    }

                    let mut realname: Vec<u8> = Vec::new();
                    {
                        let profile = self.profile.borrow();
                        for temp3 in 1..=self.num_profiles.get() as usize {
                            if *str1 == profile[temp3].name {
                                realname = nmstr(&profile[temp3].realname);
                            }
                        }
                    }

                    if ok {
                        writeln_bytes(f2, &[b"\"", &toansi(str1), b"\" = ", &toansi(&realname)]);
                        if numwritten < 200 {
                            numwritten += 1;
                        }
                        // writtenstr is a string[31]
                        writtenstr[numwritten] = str1[..str1.len().min(31)].to_vec();
                    }
                }
            }

            writeln!(f2).unwrap();
            writeln!(f2, "- - - - - - - - - - - - - - - - - - - - - - - - - ").unwrap();
            *str1 = match phase {
                1 => b"YWF".to_vec(),
                2 => b"UWF".to_vec(),
                _ => b"IWF".to_vec(),
            };
            *str1 = [str1 as &[u8], &txt(vcode as i32), b"ANYT"].concat();
            writeln_bytes(f2, &[b"* DO NOT EDIT THIS FILE FROM HERE *   C: ", str1]);
            writeln!(f2).unwrap();
            write!(f2, "From: ").unwrap();

            //{$IFDEF REG}
            *str1 = [regname, b" #", regnumber].concat();
            //{$ELSE}
            //str1:='Some nice gentleman who hasn''t yet registered SJ3.';
            //{$ENDIF}

            writeln_bytes(f2, &[&toansi(str1)]);
        };

        let writehead = |f2: &mut dyn Write, phase: u8| {
            let str2 = match phase {
                1 => "World_Cup",
                2 => "Hillrecords",
                3 => "All_Hill_Index",
                4 => "Profile_info",
                5 => "Extra_hills",
                _ => "Custom_Cups",
            };

            writeln!(f2).unwrap();
            writeln!(f2, ".:{str2}:.").unwrap();
            writeln!(f2).unwrap();
        };

        let mut f2 = dotop("SENDME.TXT");
        findnames(
            &mut f2,
            1,
            &mut str1,
            &mut sortby,
            &mut temphillorder,
            &mut tempcuphills,
            num_customs,
            &files,
        );

        writehead(&mut f2, 1);
        anywritten = false;

        laskuri = svaluestr(&str1, 11) as i32;

        for apu1 in 1..=20 {
            let top = self.top.borrow();
            if top[apu1].name.last() != Some(&b'\xff') {
                anywritten = true;
                str1 = crypt(top[apu1].score, apu1 as i32);
                writeln_bytes(
                    &mut f2,
                    &[
                        &pad_left(&txt(apu1 as i32), 2),
                        b".",
                        &pad_left(&toansi(&top[apu1].name), 25),
                        &pad_left(&txt(top[apu1].pos as i32), 3),
                        b".",
                        &pad_left(&txt(top[apu1].score), 5),
                        b" C: ",
                        &str1,
                    ],
                );
                laskuri += svaluestr(&str1, apu1 as i32) as i32;
                laskuri += svaluestr(&top[apu1].name, top[apu1].pos as i32) as i32;
            }
        }

        writeln!(f2).unwrap();
        write!(f2, "Checksum: ").unwrap();

        if anywritten {
            writeln!(f2, "{}", ((laskuri / 2) + 20424) ^ 545033).unwrap();
        } else {
            writeln!(f2, "None.").unwrap();
        }

        laskuri = 0;
        ahi = 0; //{ all hill index - prosentti }

        writehead(&mut f2, 2);
        anywritten = false;

        for apu1 in 1..=NUM_WC_HILLS {
            str1 = self.u.hrname(apu1);
            apu2 = self.u.hillkr(apu1) * 10;

            if str1.last() != Some(&b'\xff') {
                anywritten = true;
                str1 = crypt(self.u.hrlen(apu1), apu1);

                let hillname = self.u.hillname(apu1);
                writeln_bytes(
                    &mut f2,
                    &[
                        &upstr(&hillname[..hillname.len().min(3)]),
                        &pad_left(&toansi(&self.u.hrname(apu1)), 25),
                        b" ",
                        &pad_left(&txtp(self.u.hrlen(apu1)), 8),
                        b" C: ",
                        &str1,
                    ],
                );

                laskuri += svaluestr(&str1, apu1) as i32;
                laskuri += svaluestr(&self.u.hrname(apu1), apu1) as i32;

                apu2 = self.u.hrlen(apu1);
            }

            ahi += apu2; //{ eli lisätään joko omalla enkalla tai kriittisellä }
        }

        writeln!(f2).unwrap();
        write!(f2, "Checksum: ").unwrap();

        if anywritten {
            writeln!(f2, "{}", ((laskuri / 3) + 69721) ^ 408675).unwrap();
        } else {
            writeln!(f2, "None.").unwrap();
        }

        laskuri = 0;

        for apu1 in 1..=NUM_WC_HILLS {
            laskuri += self.u.hillkr(apu1) * 10;
        }

        writehead(&mut f2, 3);

        ahi = round(((ahi as f64 / laskuri as f64) * 10000.0) as f32);
        str1 = txt(ahi);
        let index = str1.len() - 1;
        insert(b".", &mut str1, index);
        str1.extend_from_slice(b" %");

        writeln_bytes(&mut f2, &[&pad_left(&str1, 10)]);

        writeln!(f2).unwrap();
        write!(f2, "Checksum: ").unwrap();

        if ahi > 10000 {
            writeln!(f2, "{}", ((ahi << 3) + 69721) ^ 408675).unwrap();
        } else {
            writeln!(f2, "None.").unwrap();
        }

        writehead(&mut f2, 4);

        for apu1 in 1..=self.num_profiles.get() as usize {
            let profile = &self.profile.borrow()[apu1];
            laskuri = 0;

            str1 = profile.name.clone();
            writeln_bytes(&mut f2, &[&toansi(&str1)]);
            laskuri = laskuri.wrapping_add(svaluestr(&str1, 3) as i32);

            ahi = profile.totaljumps; //{ ahi temp-käytössä, älä pelästy }
            writeln!(f2, " Total_jumps: {ahi}").unwrap();
            laskuri = laskuri.wrapping_add(ahi.wrapping_add(3613));

            ahi = profile.wcswon;
            writeln!(f2, " WC_won: {ahi}").unwrap();
            laskuri = laskuri.wrapping_add(ahi.wrapping_mul(311));

            ahi = profile.legswon;
            writeln!(f2, " Legs_won: {ahi}").unwrap();
            laskuri = laskuri.wrapping_add(ahi.wrapping_mul(133));

            str1 = profile.best4result.clone();
            writeln_bytes(&mut f2, &[b" Best_4H: ", &str1]);
            // str1[0] is the length of the Pascal string
            let mut apu2 = 0;
            while (if apu2 == 0 {
                str1.len() as u8
            } else {
                str1[apu2 - 1]
            }) != b' '
                && apu2 < str1.len()
            {
                apu2 += 1;
            }
            laskuri = laskuri.wrapping_add(svaluestr(&str1[..apu2.saturating_sub(1)], 7) as i32);

            let apu2 = profile.kothlevel as i32;
            writeln!(f2, " KOTH_level: {apu2}").unwrap();
            laskuri = laskuri.wrapping_add(apu2 * 1313);

            write!(f2, " Checksum: ").unwrap();
            if laskuri > 0 {
                writeln!(f2, "{}", laskuri.wrapping_add(11278) ^ 591219).unwrap();
            } else {
                writeln!(f2, "None.").unwrap();
            }
        }

        writeln!(f2, "End.").unwrap();

        writeln!(f2).unwrap();
        writeln!(f2, ".END_OF_SENDME.TXT").unwrap();
        writeln!(f2).unwrap();
        writeln!(
            f2,
            "NOTE: This file is generated for legacy reasons. You need to play"
        )
        .unwrap();
        writeln!(
            f2,
            "      original DOS version in order to participate in Universal"
        )
        .unwrap();
        writeln!(f2, "      Hiscores of SJ3! Please do not send this file.").unwrap();
        writeln!(f2).unwrap();
        writeln!(f2, "The Universal Hiscore List @ http://nomasi.com/sj3").unwrap();

        drop(f2);

        if self.u.num_extra_hills.get() > 0 {
            //{      --- EXTRAS! ---      }
            let mut f2 = dotop("EXTRAS.TXT");
            findnames(
                &mut f2,
                2,
                &mut str1,
                &mut sortby,
                &mut temphillorder,
                &mut tempcuphills,
                num_customs,
                &files,
            );

            laskuri = 0;

            writehead(&mut f2, 5);

            for apu1 in 21..=self.u.num_extra_hills.get() as i32 + 20 {
                str1 = self.u.hrname(apu1);
                if str1.last() != Some(&b'\xff') {
                    str1 = crypt(self.u.hrlen(apu1), apu1);

                    writeln_bytes(
                        &mut f2,
                        &[
                            &upstr(&self.u.hillname(apu1)),
                            b" K",
                            &txt(self.u.hillkr(apu1)),
                        ],
                    );
                    writeln!(f2, "C: {}", self.u.hillcode(apu1)).unwrap();

                    writeln_bytes(
                        &mut f2,
                        &[
                            b" ",
                            &pad_left(&toansi(&self.u.hrname(apu1)), 25),
                            b" ",
                            &pad_left(&txtp(self.u.hrlen(apu1)), 8),
                            b" C: ",
                            &str1,
                        ],
                    );

                    laskuri += svaluestr(&str1, apu1) as i32;
                    laskuri += svaluestr(&self.u.hrname(apu1), apu1) as i32;
                }
            }

            if laskuri > 0 {
                writeln!(f2).unwrap();
                writeln!(f2, "Checksum: {}", ((laskuri / 3) + 69721) ^ 408675).unwrap();
            } else {
                writeln!(f2, "None.").unwrap();
            }

            writeln!(f2).unwrap();
            writeln!(f2, ".END_OF_EXTRAS.TXT").unwrap();
            writeln!(f2).unwrap();

            drop(f2);
        }

        self.find_customs(&mut num_customs, &mut files);

        if num_customs > 0 {
            //{      --- CUSTOMS! ---      }
            let mut f2 = dotop("CUSTOMS.TXT");
            findnames(
                &mut f2,
                3,
                &mut str1,
                &mut sortby,
                &mut temphillorder,
                &mut tempcuphills,
                num_customs,
                &files,
            );

            writehead(&mut f2, 6);

            for apu1 in 1..=num_customs as usize {
                laskuri = 0;
                code = self.load_custom(
                    &files[apu1],
                    &mut sortby,
                    &mut temphillorder,
                    &mut tempcuphills,
                );

                writeln_bytes(&mut f2, &[b"File: ", &upstr(&files[apu1])]);
                writeln!(f2, "Sort: {sortby}").unwrap();
                writeln!(f2, "Code: {code}").unwrap();

                let top = self.top.borrow();
                for apu2 in 1..=20 {
                    if top[apu2 + 41].score > 0 {
                        str1 = crypt(top[apu2 + 41].score, apu2 as i32);
                        checkstrs[apu2] = str1.clone();
                        laskuri += svaluestr(&str1, apu2 as i32) as i32;
                        laskuri +=
                            svaluestr(&toansi(&top[apu2 + 41].name), top[apu2 + 41].pos as i32)
                                as i32;
                    }
                }

                write!(f2, "Chsm: ").unwrap();
                if laskuri > 0 {
                    writeln!(
                        f2,
                        "{}",
                        (laskuri + (code / 2) + (sortby as i32 * 4) + 14034) ^ 502667
                    )
                    .unwrap();
                } else {
                    writeln!(f2, "None.").unwrap();
                }

                for apu2 in 1..=20 {
                    if top[apu2 + 41].score > 0 {
                        str1 = checkstrs[apu2].clone();
                        f2.write_all(
                            &[
                                &pad_left(&txt(apu2 as i32), 2) as &[u8],
                                b".",
                                &pad_left(&toansi(&top[apu2 + 41].name), 25),
                                &pad_left(&txt(top[apu2 + 41].pos as i32), 3),
                                b".",
                                &pad_left(&txt(top[apu2 + 41].score), 7),
                                b" C: ",
                                &str1,
                            ]
                            .concat(),
                        )
                        .unwrap();
                        if top[apu2 + 41].time.first() == Some(&b'(') {
                            writeln_bytes(&mut f2, &[b" ", &top[apu2 + 41].time]);
                        } else {
                            writeln!(f2).unwrap();
                        }
                    }
                }

                writeln!(f2, "End.").unwrap();
                writeln!(f2).unwrap();
            }

            writeln!(f2, ".END_OF_CUSTOMS.TXT").unwrap();
            writeln!(f2).unwrap();

            drop(f2);
        }
    }

    pub async fn draw_main_menu(&self) {
        self.g.fill_box(0, 0, 319, 199, 0);
        self.g.draw_screen().await;
        self.p.lataa_pcx("MAIN.PCX", 320 * 200, 0, 0);

        self.p.siirra_standardi_paletti();
        self.p.special_main_paletti();
        self.p.aseta_paletti();
        self.g.write_video();

        /*
            StartSuit; { lataa ne hyppypukujen v�rit 190-200... }
        */
        self.g.font_color(240);
        self.g.write_font(170, 51, self.l.lstr(34));
        self.g.font_color(241);

        let x = 150;

        for temp in 1..=self.pmaara.get() {
            let y = (temp - 1) * 9 + 64;
            self.g
                .e_write_font(x + 12, y as i32, (temp.to_string() + ".").as_bytes());
            self.g.write_font(
                x + 20,
                y as i32,
                &self.nimet.borrow()[NUM_PL + 1 - temp as usize],
            );

            // (*
            //    if (reg) then
            //     begin
            //      fillbox(302,y,312,y+6,209+haalarit[temp]);
            //      fillbox(303,y+1,311,y+5,202+haalarit[temp]);
            //     end;
            // *)
        }

        // { Fillbox(160,120,319,199,245); }
    }
}

fn profile_code(p: &Profile) -> i32 {
    let mut temp = 0i32;

    temp += valuestr(
        &[&p.name as &[u8], &p.bestresult, &p.best4result].concat(),
        11,
    ) as i32;
    temp += (p.suitcolor ^ 31) as i32;
    temp += (p.skicolor ^ 53) as i32;
    temp += (p.kothlevel & 44) as i32;
    temp += (p.replace | 91) as i32;
    temp += (p.bestwchill ^ 157) as i32;
    temp += (p.bestwcjump & 311) as i32;
    temp += (p.besthill ^ 113) as i32;
    temp += (p.bestjump & 277) as i32;
    temp += (p.bestpoints | 133) as i32;
    temp += (p.best4points & 31) as i32;

    temp += (p.cstyle ^ 37) as i32;
    temp += (p.wcs & 741) as i32;
    temp += (p.legswon | 453) as i32;
    temp += (p.wcswon ^ 857) as i32;
    temp += (p.totaljumps + 5) as i32;

    temp
}

fn writeln_bytes(f: &mut (impl Write + ?Sized), parts: &[&[u8]]) {
    f.write_all(&parts.concat()).unwrap();
    writeln!(f).unwrap();
}
