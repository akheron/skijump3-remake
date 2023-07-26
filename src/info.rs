use crate::graph::GraphModule;
use crate::help::{txt, txtp};
use crate::lang::LangModule;
use crate::pcx::PcxModule;
use crate::platform::Platform;
use crate::rs_util::{parse_line, random, read_line};
use crate::unit::{dayandtime_now, valuestr, Hiscore, UnitModule, NUM_PL, NUM_TEAMS, NUM_WC_HILLS};
use std::cell::{Cell, RefCell};
use std::fs::File;
use std::io::BufReader;

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
    l: &'l LangModule,
    p: &'p PcxModule<'m, 's, P>,
    s: &'s P,
    u: &'u UnitModule<'g, 'l, 'm, 'p, 's, P>,
    pub top: RefCell<Vec<Hiscore>>,
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
        l: &'l LangModule,
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
            top: RefCell::new(vec![]),
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

    pub fn reset_tops(&self, kerroin: u8) {
        let scores1 = [
            [1515, 1],
            [1343, 1],
            [1212, 2],
            [1101, 3],
            [999, 4],
            [888, 5],
            [777, 6],
            [665, 7],
            [555, 8],
            [494, 10],
            [444, 12],
            [383, 14],
            [333, 16],
            [272, 18],
            [222, 20],
            [161, 22],
            [111, 25],
            [77, 30],
            [33, 35],
            [11, 40],
        ];

        let mut top = self.top.borrow_mut();
        let nimet = self.nimet.borrow();
        let jnimet = self.jnimet.borrow();

        for (i, t) in scores1.iter().enumerate() {
            top[i].score = t[0];
            top[i].pos = t[1] as u8;
        }

        for temp in 1..=20 {
            top[temp].name = [&nimet[top[temp].pos as usize] as &[u8], b"\xff"].concat();
            top[temp].time = dayandtime_now();
        }

        for temp in 1..=NUM_WC_HILLS {
            let name = [
                &nimet[1 + random(6 + random(10)) as usize] as &[u8],
                b"\xff",
            ]
            .concat();
            let len = (self.u.hillkr(temp) as f64 / 0.94) as i32 * 10 * kerroin as i32;
            let time = dayandtime_now();
            self.u.set_hrinfo(temp, name, len, time);
        }

        if self.u.num_extra_hills.get() > 0 {
            for temp in 1..=self.u.num_extra_hills.get() as i32 {
                let name = [
                    &nimet[1 + random(6 + random(10)) as usize] as &[u8],
                    b"\xff",
                ]
                .concat();
                let len =
                    (self.u.hillkr(temp + NUM_WC_HILLS) as f64 / 0.94) as i32 * 10 * kerroin as i32;
                let time = dayandtime_now();
                self.u.set_hrinfo(temp + NUM_WC_HILLS, name, len, time);
            }
        }

        let scores2 = [42, 35, 28, 21, 17, 11, 8, 5, 3, 1];
        for temp in 21..=30 {
            top[temp].pos = (temp - 20) as u8;
            top[temp].name = [&jnimet[top[temp].pos as usize] as &[u8], b"\xff"].concat();
            top[temp].time = dayandtime_now();
            top[temp].score = scores2[temp - 21];
        }

        let scores3 = [[10111, 1], [9876, 3], [9449, 6], [9010, 10], [8321, 15]];
        for temp in 31..=35 {
            top[temp].score = scores3[temp - 31][0];
            top[temp].pos = scores3[temp - 31][1] as u8;
            top[temp].name = [&nimet[top[temp].pos as usize] as &[u8], b"\xff"].concat();
            top[temp].time = dayandtime_now();
        }

        for temp in 1..=35 {
            //{ katsos jos nollaa ne scoret! }
            top[temp].pos = top[temp].pos * kerroin;
            top[temp].score = top[temp].score * kerroin as i32;
        }

        for temp in 36..=41 {
            top[temp].name = b"Not Completed".to_vec();
            top[temp].pos = 0;
            top[temp].score = 0;
            top[temp].time = dayandtime_now();
        }
    }

    fn default_profile(&self, index: i32, phase: u8) {
        let mut profiles = self.profile.borrow_mut();
        profiles[index as usize] = Profile::new();

        let mut count = 0u8;
        let mut count2 = 2u8;
        if phase == 1 {
            loop {
                let mut ok = true;
                for temp in 1..=self.num_profiles.get() as i32 {
                    if profiles[temp as usize].name == profiles[index as usize].name
                        && temp != index
                    {
                        count += 1;
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
        let mut f1 = BufReader::new(File::open("PLAYERS.SKI").unwrap());

        for temp in 0..=20 {
            self.default_profile(temp, 0);
        }

        self.num_profiles.set(parse_line(&mut f1).unwrap());
        if self.num_profiles.get() == 0 || self.num_profiles.get() > MAX_PROFILES {
            self.num_profiles.set(1);
        }

        let mut profiles = self.profile.borrow_mut();
        for temp in 1..=self.num_profiles.get() {
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
            let l1: i32 = parse_line(&mut f1).unwrap();

            if l1 != profile_code(profile) {
                println!("Profile {temp} checksum mismatch, resetting");
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

    pub fn load_names(
        &self,
        whatset: char,
        teamdiscount: u8,
        teamlineup: &mut [u8],
        replaces: bool,
    ) {
        let mut l1: i32 = 0;
        let mut s = b"".to_vec();

        let profile = self.profile.borrow();
        let mut nimet = self.nimet.borrow_mut();

        let mut f4 = BufReader::new(File::open(format!("NAMES{}.SKI", whatset)).unwrap());

        read_line(&mut f4).unwrap(); //{ titlerivi pois }

        let mut temp2: i32 = 0; //{ toteutuneet nimet }
        for temp in 1..=NUM_PL as u8 + 1 {
            s = read_line(&mut f4).unwrap();
            let mut ok = true; //{ k�yk� nimi eli ei kai ole replace-listoilla }
            for temp3 in 1..=self.pmaara.get() {
                if profile[temp3 as usize].replace == temp && replaces {
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
                    teamlineup[(temp - 1) * 4 + temp2 as usize] = tempb;
                }
            }
        }

        if whatset == '0' {
            let l2: i32 = parse_line(&mut f4).unwrap(); //{ haetaan se names:n checksum }
            l1 += 5661;
            if l1 != l2 {
                panic!("Error #92: Something wrong with the namefile. Shit.");
            }
        }
    }

    pub fn load_custom(
        &self,
        setfile: &[u8],
        sortby: &mut u8,
        hillorder: &mut [i32],
        cuphills: &mut i32,
    ) {
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
            self.l.num_languages() as i32 - 1
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
