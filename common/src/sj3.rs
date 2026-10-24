use crate::graph::GraphModule;
use crate::help::{beep, dowindow, nsqrt, pcomp, round, txt, txtp, upstr};
use crate::info::InfoModule;
use crate::lang::LangModule;
use crate::list::ListModule;
use crate::lumi::LumiModule;
use crate::maki::MakiModule;
use crate::pcx::{PcxModule, NUM_SKIS, NUM_SUITS};
use crate::platform::Platform;
use crate::regfree::{REGNAME, REGNUMBER};
use crate::rs_util::{
    clrscr, dos_file_name, halt, parse_line, random, read_line, start_end_text, textbackground,
    write, writeln, writeln_latin1,
};
use crate::table::{
    find_landing, jump_risk, lasku_anim, lasku_asento, lento_anim, parru_anim, ponn_anim,
    suksi_laskussa, suksi_lennossa,
};
use crate::trace;
use crate::tuuli::TuuliModule;
use crate::unit::{
    crypt, dayandtime, defaultkeys, injured, insert, kword, loadgoal, makeletter, uncrypt,
    valuestr, Date, Hill, Hiscore, Stat, Time, UnitModule, MAX_CUSTOMS, NUM_PL, NUM_TEAMS,
    NUM_WC_HILLS, TEAM_POINTS, WC_POINTS,
};
use std::io::{BufReader, Write};
use std::rc::Rc;
use std::str::from_utf8;

#[cfg(not(feature = "trace"))]
const VERSION: &[u8] = b"3.13-remake0";
#[cfg(not(feature = "trace"))]
const VERSION_FULL: &[u8] = b"3.13-remake0";
// The versions of the Pascal SDL port, so that its traces can be replayed
#[cfg(feature = "trace")]
const VERSION: &[u8] = b"3.13";
#[cfg(feature = "trace")]
const VERSION_FULL: &[u8] = b"3.13-SP4";

pub struct SJ3Module<'g, 'i, 'l, 'm, 'p, 's, 't, 'u, P: Platform> {
    g: &'g GraphModule<'m, 'p, 's, P>,
    i: &'i InfoModule<'g, 'l, 'm, 'p, 's, 'u, P>,
    l: &'l LangModule<'s, P>,
    ls: ListModule<'g, 'l, 'm, 'p, 's, 'u, P>,
    lumi: LumiModule,
    m: &'m MakiModule,
    p: &'p PcxModule<'m, 's, P>,
    s: &'s P,
    tuuli: &'t TuuliModule<'g, 'm, 'p, 's, P>,
    u: &'u UnitModule<'g, 'l, 'm, 'p, 's, P>,

    act_hill: Hill,
    nosamename: bool,
    automatichrr: bool,
    jmaara: u8,
    nytmaki: i32,
    wcup: bool,
    jcup: bool,
    cupslut: bool,
    pub today: Date,
    now: Time,
    pub start: Time,
    stats: [[Stat; NUM_WC_HILLS as usize + 1]; 16],
    cstats: [[i32; NUM_PL + 2]; 3],
    eka: bool,
    treeni: bool,
    compactlist: bool,
    comphrs: bool,
    diff: bool,
    diffwc: bool,
    lct: bool,
    inv_back: bool,
    beeppi: bool,
    goals: bool,
    kosystem: bool,
    trainrounds: u8,
    windplace: u8,
    sija: [u8; NUM_PL + 2],
    qual: [u8; NUM_PL + 2],
    inj: [u8; NUM_PL + 2],
    pisteet: [i32; NUM_PL + 2],
    mcpisteet: [i32; NUM_PL + 2],
    fourpts: [i32; NUM_PL + 2],
    cup_style: u8,
    cup_hills: i32,
    mcluett: [u8; NUM_PL + 2],
    luett: [u8; NUM_PL + 2],
    k: [u16; 6],
    osakilpailu: i32,
    kierros: i32,
    startgate: i32,
    hill_order: [i32; 42],
    setfile: Vec<u8>, //{ pakko olla global, muuten se unohtuu }
    koth: bool,
    kothwind: bool,
    kothmaara: u8,
    kothpack: u8,
    kothrounds: u8,
    kothmaki: i32,
    kothpel: [u8; 21],
    namenumber: u8,
    languagenumber: u8,
    gdetail: u8,
    seecomps: u8,
    teamlineup: [u8; 61],
    keula_x: i32, /*{ oltava global?
                  juu, koska se lukee sen vain m�en latauksessa,
                  ja muuten se unohtuu... }*/
    this_is_a_hill_record: i32,

    lmaara: u16,
}

impl<'g, 'h, 'i, 'l, 'm, 'p, 's, 't, 'u, P: Platform> SJ3Module<'g, 'i, 'l, 'm, 'p, 's, 't, 'u, P> {
    pub fn new(
        g: &'g GraphModule<'m, 'p, 's, P>,
        i: &'i InfoModule<'g, 'l, 'm, 'p, 's, 'u, P>,
        l: &'l LangModule<'s, P>,
        lumi: LumiModule,
        ls: ListModule<'g, 'l, 'm, 'p, 's, 'u, P>,
        m: &'m MakiModule,
        p: &'p PcxModule<'m, 's, P>,
        s: &'s P,
        tuuli: &'t TuuliModule<'g, 'm, 'p, 's, P>,
        u: &'u UnitModule<'g, 'l, 'm, 'p, 's, P>,
    ) -> Self {
        SJ3Module {
            g,
            i,
            l,
            ls,
            lumi,
            m,
            p,
            s,
            tuuli,
            u,
            act_hill: Hill::default(),
            nosamename: false,
            automatichrr: false,
            jmaara: 0,
            nytmaki: 0,
            wcup: false,
            jcup: false,
            cupslut: false,
            today: Date::default(),
            now: Time::default(),
            start: Time::default(),
            stats: ([[Stat::default(); NUM_WC_HILLS as usize + 1]; 16]),
            cstats: ([[0; NUM_PL + 2]; 3]),
            eka: false,
            treeni: false,
            compactlist: false,
            comphrs: false,
            diff: false,
            diffwc: false,
            lct: false,
            inv_back: false,
            beeppi: false,
            goals: false,
            kosystem: false,
            trainrounds: 0,
            windplace: 1,
            sija: [0; NUM_PL + 2],
            qual: [0; NUM_PL + 2],
            inj: [0; NUM_PL + 2],
            pisteet: [0; NUM_PL + 2],
            mcpisteet: [0; NUM_PL + 2],
            fourpts: [0; NUM_PL + 2],
            cup_style: 0,
            cup_hills: 0,
            mcluett: [0; NUM_PL + 2],
            luett: [0; NUM_PL + 2],
            k: [0; 6],
            osakilpailu: 0,
            kierros: 0,
            startgate: 0,
            hill_order: [0; 42],
            setfile: Vec::new(),
            koth: false,
            kothwind: false,
            kothmaara: 0,
            kothpack: 0,
            kothrounds: 0,
            kothmaki: 0,
            kothpel: [0; 21],
            namenumber: b'0',
            languagenumber: 0,
            gdetail: 0,
            seecomps: 0,
            teamlineup: [0; 61],
            keula_x: 0,
            this_is_a_hill_record: 0,
            lmaara: 0,
        }
    }

    async fn make_send_me(&self) {
        self.i
            .call_sendme(
                VERSION,
                REGNAME.as_bytes(),
                REGNUMBER.as_bytes(),
                self.today,
                self.now,
                self.u.vcode.get(),
            )
            .await;
    }

    fn write_records(&self) {
        let mut l1: i32 = 0;
        let mut l2: i32 = 0;

        let mut f1 = self.s.create_file("HISCORE.SKI");
        writeln!(f1, "HISCORE.SKI - !!! DO NOT ATTEMPT TO EDIT THIS FILE !!!").unwrap();

        //{ 20 world cupia, 10 teamia, 5 4hillsia ja 6 kothia }
        {
            let top = self.i.top.borrow();
            for temp in 1..=41i32 {
                f1.write_all(&top[temp as usize].name).unwrap();
                writeln!(f1).unwrap();
                l1 += valuestr(&top[temp as usize].name, temp) as i32;
                let str1 = crypt(top[temp as usize].pos as i32, temp);
                f1.write_all(&str1).unwrap();
                writeln!(f1).unwrap();
                l2 += valuestr(&str1, temp) as i32;
                let str1 = crypt(top[temp as usize].score, temp);
                f1.write_all(&str1).unwrap();
                writeln!(f1).unwrap();
                l2 += valuestr(&str1, temp) as i32;
            }
        }

        for temp in 1..=NUM_WC_HILLS {
            let hrname = self.u.hrname(temp);
            f1.write_all(&hrname).unwrap();
            writeln!(f1).unwrap();
            l1 += valuestr(&hrname, temp) as i32;
            let str1 = crypt(self.u.hrlen(temp), temp);
            f1.write_all(&str1).unwrap();
            writeln!(f1).unwrap();
            l2 += valuestr(&str1, temp) as i32;
        }

        l1 = l1 ^ 734697;

        writeln!(f1, "{}", l1).unwrap();
        writeln!(f1, "{}", l2).unwrap();

        let str1 = format!("{}", l1 + l2 + 53).into_bytes();
        writeln!(f1, "{}", valuestr(&str1, 22)).unwrap();

        {
            let top = self.i.top.borrow();
            for temp in 1..=41 {
                f1.write_all(&top[temp].time).unwrap();
                writeln!(f1).unwrap();
            }
        }

        for temp in 1..=NUM_WC_HILLS {
            f1.write_all(&self.u.hrtime(temp)).unwrap();
            writeln!(f1).unwrap();
        }

        self.u.makevcode(VERSION, true);
        writeln!(f1, "{}", self.u.vcode.get()).unwrap();
    }

    fn write_config(&mut self) {
        let reg = true; // { registered wai no }

        let mut f2 = self.s.create_file("CONFIG.SKI"); //{ CONFIG }

        writeln!(f2, "{}", reg as i32).unwrap();
        writeln!(f2, "{}", self.comphrs as i32).unwrap();
        writeln!(f2, "{}", self.lct as i32).unwrap();
        writeln!(f2, "{}", self.diff as i32).unwrap();
        writeln!(f2, "{}", self.compactlist as i32).unwrap();
        writeln!(f2, "{}", self.inv_back as i32).unwrap();
        writeln!(f2, "{}", self.automatichrr as i32).unwrap();
        writeln!(f2, "{}", self.beeppi as i32).unwrap();
        writeln!(f2, "{}", self.nosamename as i32).unwrap();
        writeln!(f2, "{}", self.goals as i32).unwrap();
        writeln!(f2, "{}", self.diffwc as i32).unwrap();
        writeln!(f2, "{}", self.kosystem as i32).unwrap();

        writeln!(f2, "{}", self.languagenumber).unwrap();
        writeln!(f2, "{}", self.trainrounds).unwrap();
        f2.write_all(&[self.namenumber, b'\n']).unwrap();
        f2.write_all(&self.setfile).unwrap();
        writeln!(f2).unwrap();

        writeln!(f2, "{}", self.gdetail).unwrap();
        writeln!(f2, "{}", self.seecomps).unwrap();
        writeln!(f2, "0").unwrap();
        writeln!(f2, "0").unwrap();
        writeln!(f2, "0").unwrap();

        writeln!(f2, "{}", self.jmaara).unwrap();
        let jnimet = self.i.jnimet.borrow();
        for temp in 1..=self.jmaara {
            f2.write_all(&jnimet[16 - temp as usize]).unwrap();
            writeln!(f2).unwrap();
        }

        let pmaara = self.i.pmaara.get();
        writeln!(f2, "{}", pmaara).unwrap();
        let profileorder = self.i.profileorder.borrow();
        for temp in 1..=pmaara {
            writeln!(f2, "{}", profileorder[temp as usize]).unwrap();
        }

        writeln!(f2, "{}", self.kothwind as i32).unwrap();
        writeln!(f2, "{}", self.kothrounds).unwrap();

        writeln!(f2, "{}", self.kothpack).unwrap();
        writeln!(f2, "{}", self.kothmaki).unwrap();

        writeln!(f2, "{}", self.kothmaara).unwrap();
        for temp in 1..=self.kothmaara {
            writeln!(f2, "{}", self.kothpel[temp as usize]).unwrap();
        }

        //{ n�ppiskomennot }
        for temp in 1..=5 {
            writeln!(f2, "{}", self.k[temp as usize]).unwrap();
        }

        writeln!(f2, "{}", self.windplace).unwrap();

        writeln!(f2, "0").unwrap();
        writeln!(f2, "0").unwrap();
    }

    fn reset_hiscore(&mut self, kerroin: u8) {
        self.now = self.u.get_now();

        self.i.reset_tops(kerroin, self.today, self.now);
        self.write_records();
    }

    fn reset_config(&mut self) {
        self.kothmaara = 1;
        for temp in 1..=20 {
            self.kothpel[temp as usize] = temp;
        }

        self.i.pmaara.set(1);
        {
            let mut profileorder = self.i.profileorder.borrow_mut();
            for temp in 1..=20 {
                profileorder[temp as usize] = 0;
            }
            profileorder[1] = 1;
        }

        self.jmaara = 1;

        //{ configien nollaus ocks� }
        self.languagenumber = 255; //{ se kysyy sitten alussa }

        self.beeppi = false;
        self.gdetail = 0;

        self.namenumber = b'0';

        self.trainrounds = 0;

        //{$IFDEF REG}
        self.lct = true;
        //{$ELSE}
        //self.lct = false;
        //{$ENDIF}

        self.diff = false;
        self.diffwc = false;
        self.compactlist = false;
        self.inv_back = false;
        self.automatichrr = true;

        self.goals = true;
        self.kosystem = true;
        self.seecomps = 1;

        self.comphrs = true;
        self.nosamename = false;

        defaultkeys(&mut self.k);
        self.windplace = 1;

        self.setfile = b"TEMP".to_vec();

        self.write_config();
    }

    async fn read_records(&mut self) {
        let mut l1: i32 = 0;
        let mut l2: i32 = 0;

        let mut f1 = BufReader::new(self.s.open_file("HISCORE.SKI"));

        read_line(&mut f1).unwrap(); //{ se varoitusrivi pois sielt� }

        {
            let mut top = self.i.top.borrow_mut();
            for temp in 1..=41 {
                //{ 20 World Cup, 10 Team Cupia, 5 4hillsia ja 6 Kothia }
                top[temp as usize].name = read_line(&mut f1).unwrap();
                l1 += valuestr(&top[temp as usize].name, temp) as i32;
                let str1 = read_line(&mut f1).unwrap();
                l2 += valuestr(&str1, temp) as i32;
                top[temp as usize].pos = uncrypt(str1, temp) as u8;
                let str1 = read_line(&mut f1).unwrap();
                l2 += valuestr(&str1, temp) as i32;
                top[temp as usize].score = uncrypt(str1, temp);
            }
        }

        for temp in 1..=NUM_WC_HILLS {
            //{ katso mit� tehd��n kun tehd��n lis�� m�ki� }
            let name = read_line(&mut f1).unwrap();
            l1 += valuestr(&name, temp) as i32;

            let str1 = read_line(&mut f1).unwrap();
            l2 += valuestr(&str1, temp) as i32;
            let len = uncrypt(str1.clone(), temp);

            self.u.set_hrinfo(temp, &name as &[u8], len, b"" as &[u8]);
        }

        l1 ^= 734697;

        let l3 = parse_line(&mut f1).unwrap();
        let l4 = parse_line(&mut f1).unwrap();
        let l5 = parse_line(&mut f1).unwrap();

        let str1 = (l1 + l2 + 53).to_string().into_bytes();

        {
            let mut top = self.i.top.borrow_mut();
            for temp in 1..=41 {
                top[temp].time = read_line(&mut f1).unwrap();
            }
        }

        for temp in 1..=NUM_WC_HILLS {
            let name = self.u.hrname(temp);
            let len = self.u.hrlen(temp);
            let time = read_line(&mut f1).unwrap();
            self.u.set_hrinfo(temp, name, len, time);
        }

        self.u.vcode.set(parse_line(&mut f1).unwrap_or(0));
        if self.u.vcode.get() == 0 {
            self.u.vcode.set(1); //{ pre-v3.11 }
        }

        drop(f1);

        if l1 != l3 || l2 != l4 || valuestr(&str1, 22) as i32 != l5 {
            self.m.aseta_moodi(0x3);
            writeln(b"Error #21A: Something doesn't add up in the HISCORE.SKI file.");
            writeln(b"Maybe you tried to edit it or something.  Please don't do it again.");
            self.reset_hiscore(1);
            self.reset_config();
            self.s.wait_for_key().await;
            halt().await;
        }
    }

    fn read_config(&mut self) {
        let mut temp: i32 = 0;

        let mut f2 = BufReader::new(self.s.open_file("CONFIG.SKI"));

        parse_line::<i32>(&mut f2).unwrap();
        temp = parse_line(&mut f2).unwrap();
        self.comphrs = temp != 0;
        temp = parse_line(&mut f2).unwrap();
        //{$IFDEF REG}
        self.lct = temp != 0;
        //{$ELSE}
        //self.lct=false;
        //{$ENDIF}
        temp = parse_line(&mut f2).unwrap();
        self.diff = temp != 0;
        temp = parse_line(&mut f2).unwrap();
        self.compactlist = temp != 0;
        temp = parse_line(&mut f2).unwrap();
        //{$IFDEF REG}
        self.inv_back = temp != 0;
        //{$ELSE}
        //self.inv_back=false;
        //{$ENDIF}
        temp = parse_line(&mut f2).unwrap();
        //{$IFDEF REG}
        self.automatichrr = temp != 0;
        //{$ELSE}
        //self.automatichrr=false;
        //{$ENDIF}
        temp = parse_line(&mut f2).unwrap();
        self.beeppi = temp != 0;
        temp = parse_line(&mut f2).unwrap();
        self.nosamename = temp != 0;
        temp = parse_line(&mut f2).unwrap();
        self.goals = temp != 0;
        temp = parse_line(&mut f2).unwrap();
        self.diffwc = temp != 0;
        temp = parse_line(&mut f2).unwrap();
        self.kosystem = temp != 0;

        self.languagenumber = parse_line(&mut f2).unwrap();
        self.trainrounds = parse_line(&mut f2).unwrap();
        //{$IFNDEF REG}
        //self.trainrounds=0;
        //{$ENDIF}

        // readln of a char at the end of a DOS line gives #13
        self.namenumber = read_line(&mut f2)
            .unwrap()
            .first()
            .copied()
            .unwrap_or(b'\r');
        //{$IFNDEF REG}
        //self.namenumber=0;
        //{$ENDIF}

        self.setfile = b"TEMP".to_vec();
        let str1 = read_line(&mut f2).unwrap();
        let filename = [&str1 as &[u8], b".SJC"].concat();
        if self.s.file_exists(from_utf8(&filename).unwrap()) {
            self.setfile = upstr(&str1);
        }

        self.gdetail = parse_line(&mut f2).unwrap();
        self.seecomps = parse_line(&mut f2).unwrap();

        parse_line::<i32>(&mut f2).unwrap();
        parse_line::<i32>(&mut f2).unwrap();
        parse_line::<i32>(&mut f2).unwrap();

        self.jmaara = parse_line(&mut f2).unwrap();

        {
            let mut jnimet = self.i.jnimet.borrow_mut();
            for temp in 1..=self.jmaara {
                jnimet[NUM_TEAMS + 1 - temp as usize] = read_line(&mut f2).unwrap();
            }
            if jnimet[NUM_TEAMS].is_empty() {
                jnimet[NUM_TEAMS] = b"Team Finlando".to_vec();
            }
        }

        {
            let mut pmaara = parse_line(&mut f2).unwrap();
            let mut profileorder = self.i.profileorder.borrow_mut();
            if pmaara > 0 && pmaara < 11 {
                for temp in 1..=pmaara {
                    profileorder[temp as usize] = parse_line(&mut f2).unwrap();
                }
            } else {
                pmaara = 1;
            }
            //{$IFNDEF REG}
            //if pmaara > 2 { pmaara = 2; }
            //{$ENDIF}
            self.i.pmaara.set(pmaara);
        }

        temp = parse_line(&mut f2).unwrap();
        self.kothwind = temp != 0;

        self.kothrounds = parse_line(&mut f2).unwrap();
        self.kothpack = parse_line(&mut f2).unwrap();
        //{$IFNDEF REG}
        //if (self.kothpack < 1) || (self.kothpack > 6) { self.kothpack = 1; }
        //{$ENDIF}
        self.kothmaki = parse_line(&mut f2).unwrap();
        self.kothmaara = parse_line(&mut f2).unwrap();

        if self.kothmaara > 20 {
            self.kothmaara = 1;
        }

        for temp in 1u8..=20u8 {
            self.kothpel[temp as usize] = temp; //{ ettei siell� ole nollia }
        }

        for temp in 1..=self.kothmaara {
            self.kothpel[temp as usize] = parse_line(&mut f2).unwrap();
        }

        defaultkeys(&mut self.k);

        for temp in 1..=5 {
            self.k[temp as usize] = parse_line(&mut f2).unwrap();
        }

        self.windplace = parse_line(&mut f2).unwrap();
    }

    async fn setup_menu(&mut self) {
        let mut index = [1, 1, 1, 1, 1];
        let mut screen: u8 = 0;
        let mut out = false;

        loop {
            self.g.new_screen(1, 0).await;

            self.g.font_color(240);

            let (str1, entries, length) = match screen {
                0 => (self.l.lstr(175), 6, 221),
                1 => (self.l.lstr(176), 4, 221),
                2 => (self.l.lstr(177), 11, 221),
                3 => (self.l.lstr(178), 5, 221),
                _ => unreachable!(),
            };

            self.g.write_font(30, 6, str1);

            for temp in 0..=entries {
                let str1 = match screen {
                    1 => match temp {
                        1 => {
                            if self.languagenumber < 255 {
                                self.l.lnames[(self.languagenumber - 1) as usize].clone()
                            } else {
                                b"Undecided".to_vec()
                            }
                        }
                        2 => {
                            if self.beeppi {
                                self.l.lstr(6).to_vec()
                            } else {
                                self.l.lstr(7).to_vec()
                            }
                        }
                        3 => self.l.lstr(13 + self.gdetail as u32).to_vec(),
                        4 => {
                            let names = dos_file_name(
                                &[b"NAMES" as &[u8], &[self.namenumber], b".SKI"].concat(),
                            );
                            if self.s.file_exists(&names) {
                                let mut f3 = BufReader::new(self.s.open_file(&names));
                                let str2 = read_line(&mut f3).unwrap();
                                drop(f3);
                                self.g.font_color(241);
                                self.g.write_font(40, 78, &str2);
                            }

                            vec![self.namenumber]
                        }
                        _ => b"".to_vec(),
                    },

                    2 => match temp {
                        1 => self
                            .l
                            .lstr(if self.trainrounds == 0 {
                                9
                            } else {
                                self.trainrounds as u32
                            })
                            .to_vec(),
                        2 => self.l.lstr(if self.lct { 180 } else { 185 }).to_vec(),
                        3 => self.l.lstr(if self.diff { 181 } else { 186 }).to_vec(),
                        4 => self.l.lstr(if self.diffwc { 181 } else { 186 }).to_vec(),
                        5 => self
                            .l
                            .lstr(if self.compactlist { 182 } else { 187 })
                            .to_vec(),
                        6 => self.l.lstr(if self.inv_back { 183 } else { 188 }).to_vec(),
                        7 => self
                            .l
                            .lstr(if self.automatichrr { 182 } else { 185 })
                            .to_vec(),
                        8 => self.l.lstr(if self.goals { 180 } else { 186 }).to_vec(),
                        9 => {
                            if self.seecomps > NUM_PL as u8 {
                                self.l.lstr(self.seecomps as u32).to_vec()
                            } else {
                                [b"#" as &[u8], &txt(self.seecomps as i32)].concat()
                            }
                        }
                        11 => self.l.lstr(if self.kosystem { 182 } else { 185 }).to_vec(),
                        _ => b"".to_vec(),
                    },

                    3 => match temp {
                        1 => self.l.lstr(if self.comphrs { 183 } else { 187 }).to_vec(),
                        2 => self
                            .l
                            .lstr(if self.nosamename { 185 } else { 180 })
                            .to_vec(),
                        _ => b"".to_vec(),
                    },

                    _ => b"".to_vec(),
                };

                self.u.setup_item(temp, screen, entries, &str1);
            }

            // {$IFNDEF REG}
            //   if (screen<3) then
            //    begin
            //     fontcolor(241);
            //     writefont(10,185,lstr(208)+' '+lstr(209));
            //     fontcolor(240);
            //    end;
            // {$ENDIF}

            self.g.draw_screen().await;

            index[screen as usize] = self
                .u
                .make_menu(
                    35,
                    40,
                    length,
                    10,
                    entries as i32,
                    index[screen as usize],
                    243,
                    0,
                    0,
                )
                .await;

            match screen {
                0 => match index[screen as usize] {
                    1 | 2 | 3 => screen = index[screen as usize] as u8,
                    4 => self.u.configure_keys(&mut self.k).await,
                    5 => self.u.set_goals().await,
                    6 => {
                        self.u.write_extras().await;
                        self.u.hill_maker(0).await;
                        self.u.read_extras().await;
                    }
                    0 => out = true,
                    _ => unreachable!(),
                },

                1 => match index[screen as usize] {
                    1 => self.i.welcome_screen(&mut self.languagenumber).await,
                    2 => self.beeppi = !self.beeppi,
                    3 => {
                        self.gdetail += 1;
                        if self.gdetail > 1 {
                            self.gdetail = 0;
                        }
                    }
                    4 => {
                        let mut ch;
                        loop {
                            let ok;
                            self.u.getch(255, 70, 245).await;
                            ch = self.s.get_ch();

                            if ch != 27 {
                                ok = self.s.file_exists(dos_file_name(
                                    &[b"names" as &[u8], &[ch], b".ski"].concat(),
                                ));
                            } else {
                                ok = true;
                            }

                            if ok {
                                break;
                            }
                        }

                        if ch != 27 {
                            self.namenumber = ch;
                            self.i.load_names(
                                self.namenumber,
                                self.jmaara,
                                &mut self.teamlineup,
                                true,
                            );
                        }
                    }
                    0 => screen = 0,
                    _ => unreachable!(),
                },

                2 => match index[screen as usize] {
                    1 => {
                        self.trainrounds += 1;
                        if self.trainrounds > 3 {
                            self.trainrounds = 0;
                        }
                    }
                    2 => self.lct = !self.lct,
                    3 => self.diff = !self.diff,
                    4 => self.diffwc = !self.diffwc,
                    5 => self.compactlist = !self.compactlist,
                    6 => self.inv_back = !self.inv_back,
                    7 => self.automatichrr = !self.automatichrr,
                    8 => self.goals = !self.goals,
                    9 => {
                        self.i.choose_seecomps(&mut self.seecomps).await;
                    }
                    10 => {
                        self.u.choose_wind_place(&mut self.windplace).await;
                    }
                    11 => self.kosystem = !self.kosystem,
                    0 => screen = 0,
                    _ => unreachable!(),
                },

                3 => match index[screen as usize] {
                    1 => self.comphrs = !self.comphrs,
                    2 => self.nosamename = !self.nosamename,
                    3 | 4 => {
                        self.g.fill_box(69, 79, 251, 131, 242);
                        self.g.fill_box(70, 80, 250, 130, 244);
                        self.g.fill_area(70, 80, 250, 130, 63);
                        let str1 = self.l.lstr(if index[screen as usize] == 4 {
                            191
                        } else {
                            190
                        });
                        self.g.write_font(
                            80,
                            90,
                            &[&str1 as &[u8], b" ", &self.l.lstr(192)].concat(),
                        );
                        self.g.write_font(80, 110, self.l.lstr(193));

                        self.u
                            .getch(88 + self.g.font_len(self.l.lstr(193)), 110, 243)
                            .await;

                        if self.s.get_ch().to_ascii_uppercase() == self.l.lch(6, 1) {
                            self.reset_hiscore(4 - index[screen as usize] as u8);
                        }
                    }

                    5 => {
                        self.reset_config();
                    }

                    0 => screen = 0,
                    _ => unreachable!(),
                },

                _ => unreachable!(),
            }

            if out {
                break;
            }
        }

        self.write_config(); //{ pistet��n muutokset levylle }
    }

    fn reset_stats(&mut self) {
        for aaa in 0..=15 {
            for bbb in 0..=NUM_WC_HILLS as usize {
                self.stats[aaa][bbb].comp_pos = 0;
                self.stats[aaa][bbb].wc_pos = 0;
                self.stats[aaa][bbb].comp_pts = 0;

                for ccc in 1..=2 {
                    self.stats[aaa][bbb].round_pts[ccc] = 0;
                    self.stats[aaa][bbb].round_len[ccc] = 0;
                    self.stats[aaa][bbb].reason[ccc] = 0;
                }
            }
        }
    }

    fn makikulma(&self, x: i32) -> i32 {
        let value = self.m.profiili(x + 9)
            + self.m.profiili(x + 8)
            + self.m.profiili(x + 7)
            + self.m.profiili(x + 6);
        let value = value
            - self.m.profiili(x - 3)
            - self.m.profiili(x - 4)
            - self.m.profiili(x - 5)
            - self.m.profiili(x - 2);

        if (x > self.keula_x - 15) && (x <= self.keula_x) {
            0
        } else {
            value
        }
    }

    fn drawkothinfo(&self, top5: &[u8; 7], pel: i32) {
        let str1 = [
            &self.l.lstr(67) as &[u8],
            b" ",
            &txt(1 + self.kothmaara as i32 + self.i.pmaara.get() as i32 - self.mcpisteet[0]),
            b" ",
            &self.l.lstr(8),
            b" ",
            &txt(self.kothmaara as i32 + self.i.pmaara.get() as i32),
        ]
        .concat();

        self.g.e_write_font(308, 9, &str1);

        if top5[1] > 0 {
            let mut str1 = self.l.lstr(68);

            if (self.kierros == 1) && (self.kothrounds > 1) {
                str1 = self.l.lstr(69)
            };

            self.g.e_write_font(308, 19, str1);

            let mut str1 = txtp(self.pisteet[top5[1] as usize]);

            if self.kierros == 2 {
                str1 = txtp(self.pisteet[top5[1] as usize] - self.pisteet[pel as usize]);
            }

            self.g.e_write_font(
                308,
                29,
                &([
                    self.i.nimet.borrow()[top5[1] as usize].as_slice(),
                    b"$",
                    str1.as_slice(),
                ]
                .concat()),
            );
        }
    }

    fn drawkeymap(&self) {
        self.g.draw_anim(227, 2, 64);

        self.g.font_color(246);
        self.g.e_write_font(308, 9, self.l.lstr(330));

        for temp in 1..=5 {
            self.g.e_write_font(
                308,
                temp * 10 + 9,
                &([
                    &self.l.lstr((330 + temp) as u32) as &[u8],
                    b": ",
                    &self.u.keyname(self.k[temp as usize]),
                ]
                .concat()),
            );
        }
        self.g.font_color(247);
    }

    fn drawtop5info(&self, top5: &[u8; 7], index: i32, pel: i32) {
        let text = [&self.act_hill.name as &[u8], b" K", &txt(self.act_hill.kr)].concat();
        self.g.e_write_font(308, 9, &text);

        for temp in 1..=5 {
            if self.pisteet[top5[temp] as usize] > 0 {
                let mut str1 = txtp(self.pisteet[top5[temp] as usize]);

                if self.diff && temp > 1 {
                    str1 = txtp(self.pisteet[top5[temp] as usize] - self.pisteet[top5[1] as usize]);
                }
                if self.jcup {
                    str1 = [
                        &self.i.jnimet.borrow()[top5[temp] as usize] as &[u8],
                        b"$",
                        &str1,
                    ]
                    .concat();
                } else {
                    str1 = [
                        &self.i.nimet.borrow()[top5[temp] as usize] as &[u8],
                        b"$",
                        &str1,
                    ]
                    .concat();
                }

                self.g.e_write_font(308, 13 + temp as i32 * 7, &str1);
            }
        }

        let mut str1 = self.l.lstr(62);
        if (self.kierros == 2) && (index == 1) && (self.wcup) {
            str1 = self.l.lstr(63);
        }

        let temp = self.pisteet[top5[1] as usize] - self.pisteet[pel as usize];
        if temp > 0 {
            self.g.e_write_font(
                308,
                62,
                &([&str1 as &[u8], b": ", &txtp(temp + 1)].concat()),
            );
        }
    }

    fn drawwcinfo(&self) {
        if self.jcup {
            self.g.e_write_font(308, 9, self.l.lstr(71));
        } else {
            self.g.e_write_font(308, 9, self.l.lstr(70));
        }

        for temp in 1..=5 {
            if self.mcpisteet[self.mcluett[temp] as usize] > 0 {
                let mut str1 = txt(self.mcpisteet[self.mcluett[temp] as usize]);

                if self.diffwc && temp > 1 {
                    str1 = txt(self.mcpisteet[self.mcluett[temp] as usize]
                        - self.mcpisteet[self.mcluett[1] as usize]);
                }

                if self.jcup {
                    str1 = [
                        &self.i.jnimet.borrow()[self.mcluett[temp] as usize] as &[u8],
                        b"$",
                        &str1,
                    ]
                    .concat();
                } else {
                    str1 = [
                        &self.i.nimet.borrow()[self.mcluett[temp] as usize] as &[u8],
                        b"$",
                        &str1,
                    ]
                    .concat();
                }

                self.g.e_write_font(308, 13 + temp as i32 * 7, &str1);
            }
        }
    }

    fn drawhrinfo(&self) {
        self.g.e_write_font(
            308,
            9,
            &[&self.act_hill.name as &[u8], b" K", &txt(self.act_hill.kr)].concat(),
        );
        self.g.e_write_font(308, 19, self.l.lstr(65));
        self.g.e_write_font(308, 29, &self.u.hrname(self.nytmaki));
        self.g.e_write_font(
            308,
            39,
            &[&txtp(self.u.hrlen(self.nytmaki)) as &[u8], b"\xab"].concat(),
        );
        if self.goals && self.nytmaki <= NUM_WC_HILLS {
            self.g.e_write_font(
                308,
                49,
                &[
                    &self.l.lstr(242) as &[u8],
                    b": ",
                    &txtp(loadgoal(self.s, self.nytmaki)) as &[u8],
                ]
                .concat(),
            );
        }
    }

    fn drawinfo(&self, l: &mut i32, top5: &[u8; 7], index: i32, pel: i32) {
        if (self.pisteet[top5[1] as usize] == 0) || (self.kierros < 0) {
            //{ ei tuloksia n�ytett�v�n� }
            if (self.wcup || self.jcup) && (self.mcpisteet[self.mcluett[1] as usize] > 0) {
                match *l {
                    0..=130 => self.drawhrinfo(),
                    146..=276 => self.drawwcinfo(),
                    291 => *l = 0,
                    _ => (),
                }
            } else {
                self.drawhrinfo()
            }
        } else {
            //{ on tuloksia }
            if self.koth {
                match *l {
                    0..=130 => self.drawkothinfo(top5, pel),
                    146..=276 => self.drawhrinfo(),
                    291 => *l = 0,
                    _ => (),
                }
            } else {
                match *l {
                    0..=130 => self.drawtop5info(top5, index, pel),
                    146..=276 => self.drawhrinfo(),
                    292..=422 => {
                        if self.mcpisteet[self.mcluett[1] as usize] > 0 {
                            self.drawwcinfo()
                        } else {
                            *l = 0;
                        }
                    }
                    437 => *l = 0,
                    _ => (),
                }
            }
        }
    }

    fn jarjesta5(&self, top5: &mut [u8; 7], index: i32) {
        for apu1 in 0..=5 {
            top5[apu1 as usize] = 0;
        }

        if self.koth {
            let mut apu2 = 30000;
            let mut apu3 = 0; //{ hae huonoin t�h�n menness� }

            for apu1 in 1..=index - 1 {
                if self.mcpisteet[self.mcluett[apu1 as usize] as usize] == 0 {
                    //{ jos pelaaja mukana ja ei itse? KOTH }
                    if self.pisteet[self.mcluett[apu1 as usize] as usize] < apu2 {
                        apu2 = self.pisteet[self.mcluett[apu1 as usize] as usize];
                        apu3 = self.mcluett[apu1 as usize];
                    }
                }
            }

            if apu2 != 30000 {
                top5[1] = apu3;
            }
        } else {
            let num = if self.jcup { NUM_TEAMS } else { NUM_PL } as u8;

            for apu1 in 1..=num {
                let mut apu2 = 1;
                loop {
                    //{ Verrataan jokaiseen }
                    if self.pisteet[apu1 as usize] > self.pisteet[top5[apu2 as usize] as usize] {
                        for apu3 in (apu2 + 1..=5).rev() {
                            top5[apu3 as usize] = top5[apu3 as usize - 1];
                        }
                        top5[apu2 as usize] = apu1;
                        apu2 = num;
                    }
                    apu2 += 1;
                    if apu2 >= 6 {
                        break;
                    }
                }
            }
        }
    }

    async fn hyppy(&mut self, index: i32, pel: i32, team: i32) {
        let mut tempb: u8 = 0;
        let mut temp: i32;
        let mut temp2: i32;

        let mut statsvictim: i32;
        let mut jumper_anim: u8;
        let mut ski_anim: u8;
        let mut ponnphase: u8;

        let mut kr: i32;
        let mut ponnistus: i32;
        let mut qx: f32;

        let mut paras: i32;

        let mut kor: f32;
        let mut matka: f32;
        let mut px: f32;
        let mut pxk: f32;
        let mut py: f32;
        let mut t: f32;
        let mut pl: f32;
        let mut kkor: f32;

        let mut umatka: f32;
        let mut ukor: f32;
        let mut upx: f32;
        let mut ux: i32;

        let mut kulma1: i32;
        let mut kulmas: i32;
        let mut hp: i32;
        let mut x: i32;
        let mut y: i32;
        let mut height: i32;

        let mut delta_h: [i32; 6] = [0; 6];

        let mut sx: i32;
        let mut sy: i32;
        let mut fx: i32;
        let mut fy: i32;
        let mut keula_y: i32;

        let mut wrx: i32;
        let mut wry: i32;
        let mut goalx: i32;
        let mut goaly: i32;

        let mut delta_x: i32;
        let mut delta_y: i32; //{ vaikkakin maki.x ja y ovat longintej� }

        let mut ok: bool;
        let mut out: bool;
        let mut hillrecord: bool;

        let mut kupat: u8;
        let mut clanding: u8;
        let mut landing: u8;
        let mut grade: u8;

        let mut riski: i32;
        let mut laskuri: i32;
        let mut tyylip: [i32; 9] = [0; 9]; // 1-indexed in the original code

        let mut score: i32;

        let mut kulmalaskuri: i32;

        let mut startanim: i32;
        let mut ssuunta: i32;

        let mut namestr: Vec<u8>;
        let mut str1: Vec<u8>;
        let mut str2: Vec<u8>;
        let mut replayfilename: Vec<u8>;
        let mut replayname: Vec<u8>;

        let mut cjumper: bool; // is a computer jumper (?)
        let mut draw: bool;
        let mut mcliivi: bool;
        let mut reflex: u8;
        let mut skill: u8;
        let mut maxspeed: u8;

        let mut rturns: i32;
        let mut rstartx: i32;
        let mut rstarty: i32;
        let mut rflstart: i32;
        let mut rflstop: i32;
        let mut rd: [[u8; 1001]; 5] = [[0; 1001]; 5];

        let mut top5: [u8; 7] = [0; 7];

        let mut actprofile: u8; //{ aktiivine profiili }

        /*
        function writereplay(author,name:string):byte;
        var t1, t2 : integer;
            check : longint;
            result: byte;

        begin

         {$I-}
         Assign(f1,replayfilename+'.SJR');
         Rewrite(f1);
         {$I+}
         Result:=0;

         if (IOResult=0) then
          begin
           check:=0;
           writeln(f1,RStartX); writeln(f1,RStartY); inc(check,RStartx*2+RStarty);
           writeln(f1,RTurns); inc(check,Rturns*3);
           writeln(f1,nytmaki); if (nytmaki<=numwchills) then inc(check,smallint(nytmaki*131));
           str1:='HILLBASE';
           if (nytmaki>NumWCHills) then str1:=hillfile(nytmaki-NumWCHills);
           writeln(f1,str1); inc(check,word(valuestr(str1,3)*3));
           writeln(f1,Acthill.profile); inc(check,Acthill.profile);
           writeln(f1,lmaara);
           writeln(f1,hp);  inc(check,smallint((hp+2)*69));
           writeln(f1,RFLStart);
           writeln(f1,RFLStop);  inc(check,rflstart+rflstop);
           writeln(f1,WRx); writeln(f1,WRy); inc(check,smallint((WRx+WRy)*2));
           writeln(f1,Profile[actprofile].suitcolor);
           writeln(f1,Profile[actprofile].skicolor);

           writeln(f1,author); inc(check,valuestr(author,2));
           writeln(f1,name);
           writeln(f1,dayandtime(Today,Now));
           writeln(f1,integer(mcliivi));
           t1:=0; { kisailmaisu tai 100-startgate }
           if (treeni) then t1:=100-startgate;
           if (wcup) then t1:=cupstyle+1;
           if (jcup) then t1:=4;
           if (koth) then t1:=5;
           writeln(f1,t1); inc(check,smallint(t1*1412));

           writeln(f1,check xor 3675433);
           writeln(f1,'0');

           writeln(f1);
           writeln(f1,'--- Replay Data --- ');

           write(f1,'*');

           for t1:=0 to 1000 do
            for t2:=0 to 4 do
             write(f1,char(RD[t2,t1]));

           Close(f1);

         end else result:=1;

         writereplay:=result;

        end;

        */

        // IMPLEMENTATION
        trace::value("hyppy_alkaa", &format!("i32 {pel}"));

        kr = self.act_hill.kr;
        self.s.set_ch(0);
        rturns = 0;

        if self.eka {
            self.lmaara = random(2) as u16 * random(256) as u16;
            if (self.lmaara > 0) && (self.lmaara < 40) {
                self.lmaara = 41 + random(150) as u16;
            }
            if (self.lmaara > 0) && (random(4) == 0) {
                self.lmaara += 1000; //{ r�nt� }
            }
            //{   Lmaara:=1151; }

            self.lumi.vie_lmaara(self.lmaara);

            /*{   LStyle:=Random(2)*Random(2);
            LStyle:=1; }*/

            if self.gdetail == 1 {
                self.lmaara = 0;
            }

            if self
                .u
                .load_hill(&mut self.keula_x, self.nytmaki, &self.act_hill)
                .await
                != 0
            {
                self.s.set_ch(27);
                self.cupslut = true;
            }

            self.m.x.set(0);
            self.m.y.set(0);
            self.g.draw_hill_screen().await;

            self.tuuli.hae();
        }

        self.p.siirra_standardi_paletti();

        if self.lmaara > 1000 {
            self.p.tumma_lumi();
        }
        if self.eka {
            self.p.aseta_paletti();
        }
        cjumper = true;
        draw = false;

        if pel > NUM_PL as i32 - self.i.pmaara.get() as i32 {
            //{ oma j�tk� }
            cjumper = false;
            draw = true;
        }

        temp = 0;

        if self.seecomps > NUM_PL as u8 {
            //{ jos ei suoraa pelaajavalintaa }
            match self.seecomps {
                //{ ketk� n�ytet��n }
                235 => temp = 1,
                236 => temp = 3,
                237 => temp = 5,
                238 => temp = 10,
                239 => temp = 100,
                _ => {}
            }
            temp2 = index;
            if self.koth || self.jcup {
                temp2 = team; //{ koth: t�nne johdetaan se realindex }
            }
            if temp2 <= temp {
                draw = true;
            }
        } else {
            //{ suora pelaaja }
            if self.seecomps as i32 == pel {
                draw = true;
            }
        }

        statsvictim = 0;
        actprofile = 0;

        self.p.load_suit((pel % NUM_SUITS as i32) as u8, 0);
        self.p.load_skis((pel % NUM_SKIS as i32) as u8, 0);

        if !cjumper {
            statsvictim = NUM_PL as i32 + 1 - pel; /* antaa pelaajan numeron 1..10 */
            actprofile = self.i.profileorder.borrow()[statsvictim as usize];
            self.p
                .load_suit(self.i.profile.borrow()[actprofile as usize].suitcolor, 0);
            self.p
                .load_skis(self.i.profile.borrow()[actprofile as usize].skicolor, 0);
        }

        mcliivi = true;
        if self.mcluett[1] as i32 != pel || self.treeni || self.koth {
            mcliivi = false;
        }
        if !mcliivi {
            self.p.siirra_liivi_pois();
        }

        temp = random(80) as i32; //{ 120 }
        temp -= random(24 * pel as u32) as i32; //{ 34,32, 30, 25,20, 13 ennen muutosta }
        temp -= random(12 * pel as u32) as i32; //{ 17,16,15, 12, 8,  5  e.m. }
        temp -= random(10 * pel as u32) as i32; //{ uusi }
        temp = 63 - temp; //{ 80,100,110,114,119,119  e.m. }

        skill = (17 - round(nsqrt(temp as f32) / 4.0)) as u8;
        if skill > 16 {
            skill = 16;
        }

        reflex = round(((34 + pel + random(10) as i32) as f64 / 5.0) as f32) as u8;
        /*
        {  reflex:=round((33+pel+random(8))/4); }
        {  reflex:=round((45+pel+random(10))/5); }
        {  reflex:=round((30+1+random(30))/5); }
        */

        //{ floppihyppy }
        if random((300 - pel) as u32) == 0 {
            skill = (17 + random((3 + pel / 25) as u32)) as u8;
        }

        self.now = self.u.get_now(); //{ haetaan current time }

        self.m.x.set(0);
        self.m.y.set(0);

        x = 0;
        y = 0;
        sx = 0;
        sy = 0;
        fx = 0;
        fy = 0; //{ Former X & Y }
        hp = 0;
        paras = 0; //{ treenijuttuja }

        keula_y = self.m.profiili(self.keula_x); //{ helpottaa vaan }
        pl = self.act_hill.pl_save; //{ plsave on m�enparametreja }
        maxspeed = self.act_hill.vx_final; //{ n�in voidaan lavaa vaihtaa... }

        trace::record(
            "JumpStart",
            &[
                ("pel", pel.into()),
                ("team", team.into()),
                ("nytmaki", self.nytmaki.into()),
                ("kierros", self.kierros.into()),
                ("startgate", self.startgate.into()),
                ("maxspeed", maxspeed.into()),
                ("kr", kr.into()),
                ("KeulaX", self.keula_x.into()),
                ("KeulaY", keula_y.into()),
                ("pk", self.act_hill.pk.into()),
                ("skill", skill.into()),
                ("reflex", reflex.into()),
                ("cjumper", cjumper.into()),
                ("draw", draw.into()),
                ("lmaara", (self.lmaara as i32).into()),
            ],
        );

        tyylip[1] = 195; //{ ennen 200, v3.0 195 }
        tyylip[6] = 200;
        tyylip[7] = 0; //{ tyylip[6] => pienin, [7] => suurin }

        qx = self.keula_x as f32 + 0.5; //{ emme halua, ett� se toteaa keulalla matkan>0 }

        wrx = 0;
        wry = 0;
        goalx = 0;
        goaly = 0;
        matka = 0.0;

        //{ haetaan m�kienkkakepille mesta! }
        temp = 0;
        if self.nytmaki <= NUM_WC_HILLS {
            temp = loadgoal(self.s, self.nytmaki);
        }

        if draw {
            loop {
                matka += 1.0;

                x = round(matka + qx);
                kor = self.m.profiili(x) as f32;
                kkor = kor - keula_y as f32;
                hp = round(
                    (nsqrt((matka as f64 * matka as f64 + kkor as f64 * kkor as f64) as f32) as f64
                        * self.act_hill.pk as f64
                        * 0.5) as f32,
                ) * 5;

                if hp >= self.u.hrlen(self.nytmaki) && self.u.hrlen(self.nytmaki) > 0 && wrx == 0 {
                    wrx = x;
                    wry = round(kor) - 9;
                }

                if hp >= temp && temp != 0 && goalx == 0 {
                    goalx = x;
                    goaly = round(kor) - 7;
                }

                if x > 1024 {
                    break;
                }
            }
        }

        hp = 0;
        kkor = 0.0;

        matka = (-self.keula_x + 10) as f32;
        qx = self.keula_x as f32 + 0.5; //{ emme halua, ett� se toteaa keulalla matkan>0 }

        x = round(matka + qx);

        kor = self.m.profiili(x) as f32;

        y = round(kor);

        ponnistus = 0;
        out = false;
        ok = true; //{ OK to be used w/ ei ponnistanut }
        height = 0; //{ �ij�n korkeus m�est� }
        for temp in 0..=5 {
            delta_h[temp] = 0;
        }
        riski = 0; //{ kaatumisriski }

        landing = 0; //{ 0 - ei, 1 - telemark, 2 - tasajalka }
        clanding = 0; //{ tuleva lask. 0 - ei, 1 - tele, 2 - tasa }
        kupat = 0; //{ 0 - ei, 1 - oma moka, 2 - huono s�k� }

        ponnphase = 0; //{ t�m� m��r�� animaation asennon ukossa }

        py = 0.0; //{ vauhtia Y-akselin ei oo, tai on mutta summa=0 }

        px = 0.0; //{ T�t� kerrotaan pxk:lla, kunnes px = vxfinal; }
        pxk = 1.016; //{ hyv� kerroin }

        t = 0.0; //{ aika }

        kulma1 = 0; //{ �ij�n kropan kulma }
        kulmas = 0; //{ Suksien kulma }

        ssuunta = 0; //{ sukset eiv�t liiku mihink��n }

        replayname = b"?".to_vec();
        replayfilename = b"TEMP".to_vec();

        self.m.tulosta();

        namestr = self.i.nimet.borrow()[pel as usize].clone();
        str1 = namestr.clone();

        if self.wcup {
            if self.kierros == 2 {
                str1 = [&str1 as &[u8], b" (", &txt(index), b".)"].concat()
            }
            if self.kierros == 0 && self.qual[pel as usize] > 0 {
                str1 = [&str1 as &[u8], b" Q WC"].concat()
            }
        }

        str2 = self.l.lstr(51).to_vec();

        match self.kierros {
            -5..=-1 => {
                str2 = [
                    &self.l.lstr(52) as &[u8],
                    b" ",
                    &txt(i32::abs(self.kierros)),
                ]
                .concat()
            }
            0..=2 => str2 = self.l.lstr(53 + self.kierros as u32).to_vec(),
            _ => {}
        }

        if self.kierros == 2 && self.wcup && draw {
            self.g.font_color(241);
        }

        self.jarjesta5(&mut top5, index);

        delta_x = self.m.x.get();
        delta_y = self.m.y.get();

        laskuri = 0; //{ t�t� k�ytet��n laskurina seuraavassa }

        self.p.aseta_paletti();

        self.tuuli.hae();

        if draw && self.s.get_ch() != 27 {
            loop {
                //{ INFORUUTU LUUPPI! }
                laskuri += 1;

                self.m.tulosta();

                self.draw_lumi(
                    delta_x - self.m.x.get(),
                    delta_y - self.m.y.get(),
                    self.tuuli.value.get(),
                    self.lmaara,
                    true,
                );

                self.g.draw_anim(227, 2, 64);
                self.g.draw_anim(3, 150, 65);

                self.g.font_color(247);

                self.g.write_font(12, 160, &str2);
                self.g.write_font(12, 172, self.l.lstr(56));

                if self.treeni {
                    self.g
                        .write_font(67 + self.g.font_len(self.l.lstr(58)), 27, b"(+/-)");
                }
                if self.jcup {
                    self.g.write_font(
                        14 + self.g.font_len(self.l.lstr(56)),
                        179,
                        &self.i.jnimet.borrow()[team as usize],
                    );
                }
                if !cjumper {
                    self.g.font_color(240);
                }

                self.g
                    .write_font(12 + self.g.font_len(self.l.lstr(56)), 172, &str1); //{ nimi ruutuun }

                if self.treeni {
                    self.g.write_font(64, 19, self.l.lstr(58));
                }

                self.g.font_color(241);

                if self.kierros == 2 && self.wcup {
                    self.g.write_font(
                        14 + self.g.font_len(self.l.lstr(56)),
                        179,
                        &[
                            &txtp(self.pisteet[pel as usize]) as &[u8],
                            b" (",
                            &txtp(self.cstats[1][pel as usize]),
                            b"\xab)",
                        ]
                        .concat(),
                    );
                }
                self.g.write_font(12, 191, self.l.lstr(59));

                self.g.font_color(246);

                if self.treeni {
                    self.g.write_font(
                        70 + self.g.font_len(self.l.lstr(58)),
                        19,
                        &txt(self.startgate),
                    );
                }
                temp = pel;
                if self.jcup {
                    temp = team;
                }

                self.drawinfo(&mut laskuri, &top5, index, temp);

                self.g.draw_screen().await;

                self.s.set_ch(1);

                if self.s.key_pressed().await {
                    self.s.wait_for_key_press().await;

                    if self.s.get_ch() == 0 && self.s.get_ch2() == 68 {
                        self.cupslut = true;
                        self.s.set_ch(27);
                    }

                    if self.s.get_ch().to_ascii_uppercase() == self.l.lch(60, 1) {
                        self.setup_menu().await;
                        self.s.set_ch(1);
                        self.p
                            .load_suit(self.i.profile.borrow()[actprofile as usize].suitcolor, 0);
                        self.p
                            .load_skis(self.i.profile.borrow()[actprofile as usize].skicolor, 0);
                        mcliivi = true;
                        if self.mcluett[1] != pel as u8 || self.treeni || self.koth {
                            mcliivi = false;
                        }
                        if !mcliivi {
                            self.p.siirra_liivi_pois();
                        }
                        self.tuuli.aseta_paikka(self.windplace);
                    }

                    if self.treeni {
                        if self.s.get_ch() == b'+' {
                            self.startgate += 1;
                            self.s.set_ch(1);
                        }
                        if self.s.get_ch() == b'-' {
                            self.startgate -= 1;
                            self.s.set_ch(1);
                        }
                        if self.startgate < 1 {
                            self.startgate = 1;
                        }
                        if self.startgate > 30 {
                            self.startgate = 30;
                        }
                    }
                }

                if self.s.get_ch() != 1 {
                    break;
                }
            }
        }

        if self.treeni {
            maxspeed = (maxspeed as i32 + (self.startgate - 15)) as u8;
        }

        laskuri = 0; //{ t�t� ehk� k�ytet��n laskurina seuraavassa }
        out = false;

        if draw {
            self.p.muuta_logo(6);
            self.p.aseta_paletti();
        }

        ski_anim = suksi_laskussa(self.makikulma(x));
        jumper_anim = 164;

        kulmalaskuri = 0; //{ k�ytet��n t�t� parruanimia }
        px = 0.0;

        if draw {
            self.g.font_color(247);
        }

        rstartx = x;
        rstarty = y;

        if self.s.get_ch() != 27 && draw {
            loop {
                //{ ISTUU PARRULLA LUUPPI }
                if !self.treeni {
                    laskuri += 1;
                }

                self.tuuli.hae();

                self.m.tulosta();

                if kulmalaskuri > 3024 && kulmalaskuri < 4000 {
                    temp = kulmalaskuri - 3024;
                    sx = x - self.m.x.get() + 4;
                    sy = y - self.m.y.get() - 10;

                    if temp == 1 {
                        px = 0.6;
                    }
                    px = (px as f64 + (self.tuuli.value.get() - 20) as f64 / 10000.0) as f32;

                    sx += round(temp as f32 * px);
                    sy += (temp * temp) / 160;

                    if sy < self.m.profiili(sx) {
                        self.g.put_pixel(sx, sy, 225);
                    } else {
                        kulmalaskuri = 0;
                    }
                }

                if laskuri < 350 || laskuri % 40 > 19 {
                    self.g
                        .draw_anim(x - self.m.x.get() + 60, y - self.m.y.get() - 10, 67);
                    //{ liikennevalo1 }
                }

                self.g
                    .draw_anim(x - self.m.x.get(), y - self.m.y.get() - 2, jumper_anim);
                self.g
                    .draw_anim(x - self.m.x.get(), y - self.m.y.get() - 1, ski_anim);

                self.draw_lumi(
                    delta_x - self.m.x.get(),
                    delta_y - self.m.y.get(),
                    self.tuuli.value.get(),
                    self.lmaara,
                    true,
                );

                if cjumper {
                    self.g
                        .write_font(x - self.m.x.get(), y - self.m.y.get() - 20, b"C");
                }

                if self.windplace > 10 {
                    self.tuuli.tuo(x - self.m.x.get(), y - self.m.y.get());
                }

                if laskuri < 700 {
                    self.tuuli.piirra();
                }

                if self.eka && self.osakilpailu == 1 && !cjumper {
                    self.drawkeymap();
                }

                self.g.draw_screen().await;

                self.s.set_ch(0);
                self.s.set_ch2(0);

                if self.s.key_pressed().await {
                    let (ch, ch2) = self.s.wait_for_key_press().await;
                    self.s.set_ch(ch);
                    self.s.set_ch2(ch2);
                    self.s.set_ch(self.s.get_ch().to_ascii_uppercase());
                    if self.s.get_ch() == 0 && self.s.get_ch2() == 68 {
                        self.cupslut = true;
                        self.s.set_ch(27);
                    }
                    if !cjumper && (self.s.kword() == self.k[2] || self.s.get_ch() == 13) {
                        out = true; //{ liikkeelle }
                    }
                    if self.s.get_ch() == 0
                        && self.s.get_ch2() == 63
                        && self.wcup
                        && self.cup_style == 1
                        && index == NUM_PL as i32
                        && self.kierros == 1
                        && self.osakilpailu == 1
                    {
                        self.tuuli.alusta(self.windplace);
                    }
                    if self.s.get_ch() == 27 {
                        out = true;
                    }
                }
                if cjumper && (random(100) == 0 || laskuri > 600) {
                    out = true;
                }
                if cjumper && random(100) == 0 && kulmalaskuri == 0 {
                    kulmalaskuri = (random(3) as i32 + 1) * 1000;
                    sx = 0;
                    sy = 0;
                }

                if kulmalaskuri > 0 {
                    kulmalaskuri += 1;
                }

                if kulmalaskuri == 0 && !cjumper {
                    if self.s.kword() == self.k[3] {
                        kulmalaskuri = 3000;
                    }
                    if self.s.kword() == self.k[4] {
                        kulmalaskuri = 2000;
                    }
                    if self.s.kword() == self.k[1] {
                        kulmalaskuri = 1000;
                    }
                    sx = 0;
                    sy = 0;
                }

                jumper_anim = parru_anim(&mut kulmalaskuri);

                if self.s.get_ch() == 27 && cjumper {
                    jumper_anim = 164; //{ emm� haluu tsiigaa }
                }

                if out && jumper_anim == 164 || laskuri > 700 {
                    break;
                }
            }
        }

        sx = 0;
        sy = 0;

        if laskuri > 700 {
            //{ ei sitte kaveri l�htenytk��n }
            if self.beeppi {
                beep(self.s, 2);
            }
            self.p.muuta_logo(4); //{ vihre� pois }
            self.p.aseta_paletti();
            self.g
                .draw_anim(x - self.m.x.get() + 60, y - self.m.y.get() - 10, 67); //{ liikennevalo }

            self.g.draw_anim(3, 150, 65);

            self.g.font_color(240);
            self.g.write_font(
                12,
                160,
                &[&namestr as &[u8], b" ", &self.l.lstr(79)].concat(),
            );

            self.g.draw_screen().await;

            self.s.wait_for_key().await;
            self.s.set_ch(27);
        }

        out = false;

        laskuri = 0;
        px = 0.0;

        if self.s.get_ch() == 27 && draw && cjumper {
            //{ emm� haluukkaan tsiigata }
            draw = false;
            self.s.set_ch(0);
        }

        if cjumper && !draw {
            //{ noputetaan v�h�n :) }
            px = self.act_hill.vx_final as f32;
            matka = -45.0;
            for _ in 1..=100 {
                self.tuuli.siirra(); //{ veivataan my�s tuulta v�h�n }
            }
        }

        kulmalaskuri = 200;

        if self.s.get_ch() != 27 {
            //{ hyppy - main }
            loop {
                self.tuuli.hae();

                matka = (matka as f64 + px as f64 * 0.01) as f32;

                fx = x; //{ former x & y }
                fy = y;

                x = round(matka + qx);

                if self.s.get_ch() != 27 {
                    self.s.set_ch(0); //{ en tied� muisteleeko se vanhoja }
                }
                self.s.set_ch2(0);

                if self.s.key_pressed().await {
                    let (ch, ch2) = self.s.wait_for_key_press().await;
                    self.s.set_ch(ch.to_ascii_uppercase());
                    self.s.set_ch2(ch2);

                    if self.s.get_ch() == 0 && self.s.get_ch2() == 68 {
                        self.cupslut = true;
                        self.s.set_ch(27);
                        out = true;
                    }

                    if self.s.get_ch() == 27 {
                        if cjumper {
                            draw = false;
                            if px < 37.0 {
                                px = 37.0;
                            }
                        } else {
                            out = true;
                        }
                    }

                    if cjumper {
                        self.s.set_ch(0);
                        self.s.set_ch2(0);
                    }

                    if self.s.get_ch() == b'P' {
                        self.s.wait_for_key_press().await;
                    }
                }

                if matka >= 0.0 {
                    //{ matka => 0 eli kun matka=0 ollaan jo lennossa }
                    laskuri += 1;

                    if kulma1 < kulmalaskuri {
                        kulmalaskuri = kulma1;
                    }

                    if cjumper {
                        //{ automatic jumper angle differentiating }
                        if lento_anim(kulma1) > 107 && laskuri % (reflex as i32) == 0 {
                            self.s.set_ch(((self.k[2] & 0xff00) >> 8) as u8);
                            self.s.set_ch2((self.k[2] & 0xff) as u8);
                        }
                        if lento_anim(kulma1) < 107 && laskuri % (reflex as i32) == 0 {
                            self.s.set_ch(((self.k[3] & 0xff00) >> 8) as u8);
                            self.s.set_ch2((self.k[3] & 0xff) as u8);
                        }

                        temp = find_landing(self.makikulma(x));

                        if clanding == 2 {
                            temp = round((temp as f64 * 0.6) as f32);
                        }

                        //{ kumpualalastulovarmistus! }
                        if landing == 0 && matka > 3.0 && height < 4 {
                            landing = 2;
                        }

                        if landing == 0 && delta_h[0] + delta_h[1] + delta_h[2] > 1 && height < temp
                        {
                            if clanding == 2 {
                                landing = 2;
                            }
                            if clanding == 0 {
                                //{ eka kertaa }
                                if (random(250) as i32) < (temp * (temp / 10)) {
                                    clanding = 2;
                                } else {
                                    landing = 1;
                                }
                            }
                        }
                    } //{ if (cjumper) }

                    if kword(self.s.get_ch(), self.s.get_ch2()) == self.k[4] {
                        landing = 1;
                    }
                    if kword(self.s.get_ch(), self.s.get_ch2()) == self.k[5] {
                        landing = 2;
                    }

                    if kword(self.s.get_ch(), self.s.get_ch2()) == self.k[3] && kulma1 <= 600 {
                        kulma1 += round((kulma1 as f64 / 4.0) as f32);
                    }
                    if kword(self.s.get_ch(), self.s.get_ch2()) == self.k[2]
                        && landing == 0
                        && kulma1 > 0
                    {
                        kulma1 -= round((kulma1 as f64 / 5.0) as f32);
                    }

                    //{ ylirotaatio }
                    if kulma1 < 50 {
                        pl = (pl as f64 + 0.0001 - (kulma1 - 50) as f64 / 18000.0) as f32;
                    }

                    if self.tuuli.value.get() > 0 {
                        pl = (pl as f64 - (1.0 - kulma1 as f64 / 900.0) / 1875.0
                            + f64::sqrt(f64::sqrt((2 * self.tuuli.value.get()) as f64)) / 65500.0)
                            as f32;
                    } else {
                        pl = (pl as f64
                            - (1.0 - kulma1 as f64 / 900.0) / 1875.0
                            - f64::sqrt(f64::sqrt((-2 * self.tuuli.value.get()) as f64)) / 65500.0)
                            as f32;
                    }

                    px = (px as f64 - (kulma1 as f64 / 900.0) / 20.0
                        + (nsqrt((4 * self.tuuli.value.get() + 245) as f32) as f64 - 16.0) / 400.0)
                        as f32;

                    t = (t as f64 + 0.01) as f32;

                    //{ Satunnaispuuska }
                    if (random(30000) as i32) < self.tuuli.windy.get() + 10 + self.tuuli.voim.get()
                    {
                        laskuri = 0; //{ reflexi� varten }

                        if random(2) == 1 {
                            tyylip[1] -= 5;
                        }

                        temp = random(15) as i32; //{ �ij�n kulman muutos... S��D� T�T�! }
                        temp -= 6;

                        kulma1 += temp;

                        if temp > 0 {
                            //{ puuska tuo sukset yl�s }
                            ssuunta = 3;
                            pl = (pl as f64
                                - random(self.tuuli.voim.get() as u32 + 50) as f64 / 15000.0)
                                as f32;
                        }

                        if temp < 0 {
                            //{ puuska vie suksia alas }
                            ssuunta = 6;
                            pl = (pl as f64
                                + random(self.tuuli.voim.get() as u32 + 50) as f64 / 15000.0)
                                as f32;
                        }
                    }

                    if pl < 0.105 {
                        pl = 0.105; //{ leijuntaesto :) }
                    }

                    kor = (kor as f64 + t as f64 * t as f64 * pl as f64 - (py as f64 - 8.0) / 100.0)
                        as f32;

                    if ssuunta > 0 {
                        //{ SUKSIEN HEILUNTA - ponnistusfibat ja puuskat }
                        match ssuunta {
                            2 => {
                                //{ ponn my�h�ss�, k menossa alas }
                                if kulmas < 0 {
                                    kulmas += 2;
                                }
                                if kulmas > 0 {
                                    kulmas = 0;
                                }
                            }
                            1 => {
                                //{ ponn my�h�ss�, k tulossa yl�s }
                                if kulmas == 0 {
                                    //{ first strike }
                                    kulmas = -51 - (16 - ponnistus) * 6;
                                    if kulmas < -105 {
                                        kulmas = -105;
                                    }
                                } else {
                                    kulmas -= 4;
                                }
                                if kulmas < (ponnistus - 16) * 14 {
                                    ssuunta = 2;
                                }
                            }
                            3 => {
                                //{ puuska, k saa iskua (ja tulee alas) }
                                kulmas = kulmas - random(50) as i32 - 30;
                                ssuunta = 2;
                            }
                            5 => {
                                //{ ponn ajoissa, k tulossa yl�s }
                                if kulmas > 0 {
                                    kulmas -= 1;
                                }
                                if kulmas < 0 {
                                    kulmas = 0;
                                }
                            }
                            4 => {
                                //{ ponn ajoissa, k menee alas }
                                if kulmas == 0 {
                                    //{ first strike }
                                    kulmas = 70 + (ponnistus - 16) * 6;
                                    if kulmas > 130 {
                                        //{ ei nyt ihan �lyt�n }
                                        kulmas = 130;
                                    }
                                } else {
                                    kulmas += 3;
                                }

                                if kulmas > (ponnistus - 16) * 14 {
                                    ssuunta = 5; //{ riitt�� }
                                }
                            }
                            6 => {
                                //{ puuska, k saa iskua (ja tulee yl�s) }
                                kulmas = kulmas + random(50) as i32 + 30;
                                ssuunta = 5;
                            }
                            _ => {}
                        }
                        if kulmas == 0 {
                            ssuunta = 0;
                        }
                    }

                    if landing > 0 {
                        //{ alastulolis�ykset }
                        if kulma1 < 600 {
                            kulma1 += 9 + (landing as i32 - 1) * 5;
                            if pl < 1.0 {
                                pl = (pl as f64 + 0.003) as f32;
                            }
                        }
                    }

                    if ok {
                        //{ alkukulma... RUN ONCE (OK) }
                        kulma1 = 158;
                        ok = false;

                        if ponnistus < 16 {
                            ssuunta = 1; //{ ponn liian my�h�� - k�rjet yl�s }
                        }
                        if ponnistus > 16 {
                            ssuunta = 4; //{ ponn liian ajoissa - k�rjet alas }
                        }
                        if ponnistus == 0 {
                            ssuunta = 0; //{ ei sitten ponnistanut ollenkaan }
                        }
                    }

                    jumper_anim = lento_anim(kulma1);

                    if ponnphase < 25 {
                        jumper_anim = ponn_anim(&mut ponnphase);
                    }

                    temp = height;
                    height = self.m.profiili(x) - round(kor);

                    if height < 0 {
                        height = 0;
                    }

                    delta_h[laskuri as usize % 3] = temp - height;

                    if height < 6 && matka > 20.0 {
                        //{ j�nnitt�v� m�en ennakointi! }
                        if kulmas == 0 {
                            ssuunta = 0; //{ meik� m��r�� varmasti asennon }
                        }
                        ski_anim = suksi_laskussa(self.makikulma(x) / (height + 1));
                    } else {
                        ski_anim = suksi_lennossa(kulmas);
                    }

                    if height == 0 {
                        //{ the eagle has landed }
                        out = true;
                    }
                } else {
                    //{ matka < 0 }
                    laskuri += 1;

                    //{ mies ponnistaa jo... }
                    if ponnistus > 0 {
                        ponnistus += 1;
                    }

                    //{ start ponnistus! }
                    if kword(self.s.get_ch(), self.s.get_ch2()) == self.k[1]
                        && matka > -40.0
                        && ponnistus == 0
                    {
                        ponnistus += 1;
                    }

                    //{ Automaattiponnistus }
                    if cjumper
                        && matka as f64 > -(skill as f64 * px as f64 * 0.01)
                        && ponnistus == 0
                    {
                        ponnistus += 1;
                    }

                    ski_anim = suksi_laskussa(self.makikulma(x));

                    jumper_anim = lasku_asento(ski_anim);

                    if draw && laskuri < 28 {
                        jumper_anim = (165 + laskuri / 7) as u8;
                        px = 37.0;
                        if laskuri < 14 {
                            px = 0.0;
                        }
                    }

                    if ponnistus > 0 {
                        jumper_anim = ponn_anim(&mut ponnphase);
                    }

                    //{ pudotetaan �ij� }

                    kor = self.m.profiili(x) as f32;

                    px *= pxk; //{ Hanaa... }

                    if px > maxspeed as f32 {
                        px = maxspeed as f32;
                    }

                    if ponnistus > 0 {
                        //{ ponnistuslis�ys }
                        px = (px as f64 + 0.21) as f32;
                        py = (py as f64 + 1.21) as f32;
                        kulma1 += 12;

                        if ponnistus > 16 {
                            //{ Ponnistus liian aikaisin }
                            if ponnistus == 17 {
                                pl = (pl as f64 + 0.023) as f32;
                                if !cjumper && self.beeppi {
                                    beep(self.s, 1);
                                }
                            }

                            pl = (pl as f64 + 0.013) as f32;
                            py -= 1.0;

                            if self.kierros > 0 {
                                self.stats[statsvictim as usize][self.osakilpailu as usize]
                                    .reason[self.kierros as usize] = 1;
                            }

                            kulma1 = 158; //{ onko tarpeellinen? }
                        }
                    }
                } //{ end matkariippuvaiset }

                x = round(matka + qx);
                y = round(kor);

                trace::record(
                    "JumpState",
                    &[
                        ("frame", trace::rendered_frames().into()),
                        ("matka", matka.into()),
                        ("px", px.into()),
                        ("py", py.into()),
                        ("pl", pl.into()),
                        ("t", t.into()),
                        ("kor", kor.into()),
                        ("x", x.into()),
                        ("y", y.into()),
                        ("kulma1", kulma1.into()),
                        ("kulmas", kulmas.into()),
                        ("ponnistus", ponnistus.into()),
                        ("landing", landing.into()),
                        ("height", height.into()),
                        ("laskuri", laskuri.into()),
                        ("ssuunta", ssuunta.into()),
                        ("out", out.into()),
                    ],
                );

                if draw {
                    delta_x = self.m.x.get();
                    delta_y = self.m.y.get();

                    if x >= 160 && x < 864 {
                        sx += x - fx;
                    }
                    if y >= 100 && y < 412 {
                        sy += y - fy;
                    }

                    self.m.x.set(sx);
                    self.m.y.set(sy);

                    if self.m.x.get() > 704 {
                        self.m.x.set(704);
                    }
                    if self.m.y.get() > 312 {
                        self.m.y.set(312);
                    }

                    self.m.tulosta();

                    self.draw_lumi(
                        delta_x - self.m.x.get(),
                        delta_y - self.m.y.get(),
                        self.tuuli.value.get(),
                        self.lmaara,
                        true,
                    );

                    if self.goals && goalx > 0 {
                        self.g
                            .draw_anim(goalx - self.m.x.get(), goaly - self.m.y.get(), 66);
                        //{ tavoittekeppi }
                    }
                    if wrx > 0 {
                        self.g
                            .draw_anim(wrx - self.m.x.get(), wry - self.m.y.get(), 68);
                        //{ m�kienkkakeppi }
                    }

                    self.g
                        .draw_anim(x - self.m.x.get(), y - self.m.y.get() - 2, jumper_anim);
                    self.g
                        .draw_anim(x - self.m.x.get(), y - self.m.y.get() - 1, ski_anim);

                    if cjumper {
                        self.g
                            .write_font(x - self.m.x.get(), y - self.m.y.get() - 20, b"C");
                    }

                    if self.windplace > 10 {
                        self.tuuli.tuo(x - self.m.x.get(), y - self.m.y.get());
                    }

                    self.tuuli.piirra();

                    rd[0][rturns as usize] = (128 + x - fx) as u8;
                    rd[1][rturns as usize] = (128 + y - fy) as u8;
                    rd[2][rturns as usize] = jumper_anim;
                    rd[3][rturns as usize] = ski_anim;
                    rd[4][rturns as usize] = (self.tuuli.value.get() + 128) as u8;
                    if matka < 0.0 {
                        rflstart = rturns;
                    } else {
                        rflstop = rturns;
                    }
                    rturns += 1;

                    self.g.draw_screen().await;
                }

                if out {
                    break;
                }
                //{ ***  LENTO LOPPUU  *** }
            }
        }

        kkor = kor - keula_y as f32;
        hp = round(
            (nsqrt((matka as f64 * matka as f64 + kkor as f64 * kkor as f64) as f32) as f64
                * self.act_hill.pk as f64
                * 0.5) as f32,
        ) * 5;

        if self.s.get_ch() == 27 {
            //{ painoi ESCi� }
            hp = 0;
            score = 0;
            landing = 1;
            if self.kierros >= 0 {
                self.cstats[self.kierros as usize][pel as usize] = hp;
                self.stats[statsvictim as usize][self.osakilpailu as usize].round_pts
                    [self.kierros as usize] = score;
                self.stats[statsvictim as usize][self.osakilpailu as usize].round_len
                    [self.kierros as usize] = hp;
            }
        } else {
            self.i.profile.borrow_mut()[actprofile as usize].totaljumps += 1;

            grade = 0;
            if kr != 0 {
                grade = (round((hp as f64 / kr as f64) as f32) * 10) as u8;
            }

            temp = self.makikulma(x);
            height = round((temp as f64 * 1.34 + kulma1 as f64 / 10.0) as f32);

            riski = jump_risk(temp);
            if (hp as f64) < (20.0 / 3.0 * kr as f64) {
                riski = 1; //{ lyhyiden hyppyjen riski? }
            }

            if height < 63 {
                riski = round((riski as f64 * (1.0 + (63 - height) as f64 * 0.075)) as f32);
            }

            if landing == 0 || height < 56 {
                if self.kierros > 0 {
                    self.stats[statsvictim as usize][self.osakilpailu as usize].reason
                        [self.kierros as usize] = 2;
                }
                kupat = 2;
                if landing == 0 {
                    kupat = 1;
                }
            }

            if landing == 1 {
                //{ telemark-juttuja }
                riski += 2 * riski;
                if height < 60 {
                    tyylip[1] -= 5;
                }
                if height < 64 {
                    tyylip[1] -= 5;
                }
            }

            if (random(1000) as i32) < riski {
                //{ lian longa hyppy tai vaan kehno sk�g� }
                if self.kierros > 0 {
                    self.stats[statsvictim as usize][self.osakilpailu as usize].reason
                        [self.kierros as usize] = 3;
                }
                kupat = 3;
            }

            for _ in 1..=round(((kr as f64 + kr as f64 / 20.0 - hp as f64 / 10.0) / 6.0) as f32) {
                tyylip[1] -= 5;
            }

            if kupat > 0 {
                tyylip[1] -= 100;
            } else if landing == 2 {
                tyylip[1] -= 15 + random(2) as i32 * 5; //{ tasajalka tuomarirokotus }
            }

            for temp in 2..=5 {
                temp2 = random(4) as i32;
                tyylip[temp] = tyylip[1];
                tyylip[temp] -= (temp2 - 1) * 5; //{ ennen temp2-2 }
            }

            for temp in 1..=5 {
                if tyylip[temp] > 200 {
                    tyylip[temp] = 200;
                }
                if tyylip[temp] < 0 {
                    tyylip[temp] = 0;
                }
                if tyylip[temp] > tyylip[7] {
                    tyylip[7] = tyylip[temp];
                }
                if tyylip[temp] < tyylip[6] {
                    tyylip[6] = tyylip[temp];
                }
            }

            if kupat > 0 {
                self.inj[pel as usize] = injured(); //{ LOUKKAANTUMINEN!!! }
            }

            score = 0;

            for temp in 1..=5 {
                score += tyylip[temp];
            }

            score -= tyylip[6]; //{ pienin ja suurin pois }
            score -= tyylip[7];

            if kr != 0 {
                //{ vanha tyyli }
                score += round(
                    ((hp as f64 / 10.0 - (kr * 2) as f64 / 3.0) * (180.0 / kr as f64) * 10.0)
                        as f32,
                ); //{ pituuspisteet }
            }

            /*(*
              if (kr<>0) then { uusi fis:in mukainen }
               inc(score,round((hp-(kr*10))*(lengthpoint(kr)/10)+600));  { pituuspisteet }

              if (kr>=160) then inc(score,600); { lentom�ist� saa isot pisteet }
            *)*/

            trace::record(
                "JumpResult",
                &[
                    ("hp", hp.into()),
                    ("score", score.into()),
                    ("grade", grade.into()),
                    ("kupat", kupat.into()),
                    ("landing", landing.into()),
                    ("riski", riski.into()),
                    ("height", height.into()),
                ],
            );

            if score > paras {
                //{ harjoituskamaa }
                paras = score;
            }

            if self.kierros == -10 {
                self.pisteet[NUM_PL + 1] = score;
            }

            if !self.jcup && self.kierros >= 0 {
                //{ Oma pelaajan pisteet }
                self.pisteet[pel as usize] += score;
                self.cstats[self.kierros as usize][pel as usize] = hp;
            }

            if self.jcup {
                self.pisteet[team as usize] += score;
            }

            if self.kierros >= 0 {
                self.stats[statsvictim as usize][self.osakilpailu as usize].round_pts
                    [self.kierros as usize] = score;
                self.stats[statsvictim as usize][self.osakilpailu as usize].round_len
                    [self.kierros as usize] = hp;
            }

            for temp in 1..=5 {
                tyylip[temp] += 10000;
            }

            hillrecord = false;

            {
                let mut profile = self.i.profile.borrow_mut();
                if self.wcup
                    && self.cup_style == 0
                    && hp as u16 > profile[actprofile as usize].bestwcjump
                {
                    //{ vain real world cup }
                    profile[actprofile as usize].bestwcjump = hp as u16;
                    profile[actprofile as usize].bestwchill = self.nytmaki as u8;
                }

                if hp as u16 > profile[actprofile as usize].bestjump {
                    profile[actprofile as usize].bestjump = hp as u16;
                    profile[actprofile as usize].besthill = self.nytmaki as u8;
                    profile[actprofile as usize].besthillfile = b"HILLBASE".to_vec();
                    if self.nytmaki > NUM_WC_HILLS as i32 {
                        profile[actprofile as usize].besthillfile =
                            self.u.hillfile(self.nytmaki - NUM_WC_HILLS as i32);
                    }
                }
            }

            if !self.koth && kupat == 0 && !self.treeni && hp > self.u.hrlen(self.nytmaki) {
                hillrecord = true;
            }

            if cjumper && !self.comphrs {
                hillrecord = false; //{ ei saa }
            }

            if hillrecord {
                if cjumper {
                    namestr.push(255);
                }
                self.u.set_hrinfo(
                    self.nytmaki,
                    namestr.clone(),
                    hp,
                    dayandtime(self.today, self.now),
                );
                trace::record(
                    "HillRecord",
                    &[
                        ("hill", self.nytmaki.into()),
                        ("name", (&self.u.hrname(self.nytmaki) as &[u8]).into()),
                        ("length", self.u.hrlen(self.nytmaki).into()),
                        ("time", (&self.u.hrtime(self.nytmaki) as &[u8]).into()),
                    ],
                );

                if self.kierros > 0 {
                    self.stats[statsvictim as usize][self.osakilpailu as usize].reason
                        [self.kierros as usize] = 5;
                }

                self.this_is_a_hill_record = (self.kierros * 1000) + pel;
            }

            laskuri = 0;
            startanim = 100; //{ start "nousepa yl�s"-kuvio }

            ok = random(2) != 0; //{ if OK then sukset j�� kiinni }

            if landing == 2 {
                startanim -= 50;
            }

            umatka = matka; //{ ukon koordinaatit, jos sukset l�htee alta... }
            ukor = kor;
            upx = px;

            self.g.font_color(247);

            out = false;

            if kupat > 0 {
                grade = kupat;
            }

            if draw {
                //{ ***  LASKU  *** }
                loop {
                    self.tuuli.hae();

                    laskuri += 1;

                    matka = (matka as f64 + px as f64 * 0.008) as f32;
                    umatka = (umatka as f64 + upx as f64 * 0.008) as f32;

                    if ok {
                        matka = umatka;
                    }

                    fx = x;
                    fy = y;

                    x = round(matka + qx);
                    ux = round(umatka + qx);

                    kor = self.m.profiili(x) as f32;
                    ukor = self.m.profiili(ux) as f32;

                    trace::record(
                        "LandState",
                        &[
                            ("frame", trace::rendered_frames().into()),
                            ("matka", matka.into()),
                            ("umatka", umatka.into()),
                            ("px", px.into()),
                            ("upx", upx.into()),
                            ("kor", kor.into()),
                            ("ukor", ukor.into()),
                            ("x", x.into()),
                            ("ux", ux.into()),
                            ("laskuri", laskuri.into()),
                        ],
                    );

                    ski_anim = suksi_laskussa(self.makikulma(x));

                    jumper_anim = lasku_anim(ski_anim as i32, landing); //{ t�t� voi my�s sitten muuttaa jos haluaa }

                    if laskuri < 7 && landing > 0 {
                        jumper_anim = 113 + landing; //{ esilaskeutuminen }
                    }

                    if kupat > 0 {
                        if laskuri > 50 && upx > 0.0 {
                            upx = (upx as f64 - 0.8) as f32;
                        }
                        if upx < 0.0 {
                            upx = 0.0;
                            if ok {
                                out = true;
                            }
                        }
                        match kupat {
                            1 | 2 => {
                                //{ ei alas tuloa, �ij� suoraan turvalleen }
                                temp = 2 - (kulma1 / 80);
                                if temp < 0 {
                                    temp = 0;
                                }
                                tempb = (142 + laskuri / 10 + temp) as u8;
                                if kupat == 2 {
                                    tempb = (142 + (laskuri - 6) / 10 + temp) as u8;
                                }

                                if tempb > 145 {
                                    tempb = 144;
                                    match suksi_laskussa(self.makikulma(ux)) - 71 {
                                        4 => tempb = 145,
                                        5 => tempb = 146,
                                        6 => tempb = 147,
                                        7..=12 => tempb = 148,
                                        _ => {}
                                    }
                                }

                                if !(kupat == 2 && laskuri < 6) {
                                    jumper_anim = tempb;
                                }
                            }
                            3 => {
                                if laskuri > 14 {
                                    //{ liian pitk� - esilaskeut. & v�h�n norm. laskua }
                                    tempb = (151 + (laskuri - 14) / 10) as u8;
                                    if tempb > 155 {
                                        tempb = 163;
                                        match suksi_laskussa(self.makikulma(ux)) - 71 {
                                            3..=4 => tempb = 162,
                                            5..=6 => tempb = 161,
                                            7..=12 => tempb = 160,
                                            _ => {}
                                        }
                                    } else if landing == 2 {
                                        tempb += 5;
                                    }
                                    jumper_anim = tempb;
                                }
                            }
                            _ => {}
                        }
                    } else {
                        //{ ei kaatuna }
                        if laskuri > startanim {
                            temp = (laskuri - startanim) / 12;
                            if temp > 6 {
                                temp = 6;
                            }
                            match temp {
                                0 => tempb = 122 + (landing * 6),
                                1 => tempb = 123 + (landing * 6),
                                2 => tempb = 136,
                                3..=6 => {
                                    tempb = 136; //{ peruslasku }
                                    match grade {
                                        0..=75 => {
                                            //{ häpeä vähän }
                                            tempb = 137
                                        }
                                        105..=200 => {
                                            //{ yeah! }
                                            tempb = 139;
                                            if temp > 3 {
                                                if grade > 114 {
                                                    tempb = 141;
                                                } else {
                                                    tempb = 140;
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                _ => {}
                            }
                            jumper_anim = tempb;
                        }
                    }

                    self.s.set_ch(0);

                    if self.s.key_pressed().await {
                        let (ch, ch2) = self.s.wait_for_key_press().await;
                        self.s.set_ch(ch.to_ascii_uppercase());
                        self.s.set_ch2(ch2);
                    }

                    if self.s.get_ch() == 0 && self.s.get_ch2() == 68 {
                        self.cupslut = true;
                        self.s.set_ch(27);
                    }
                    if self.s.get_ch() == 27 || self.s.get_ch() == 13 {
                        out = true;
                    }
                    if self.s.get_ch() == b'P' {
                        let (ch, ch2) = self.s.wait_for_key_press().await;
                        self.s.set_ch(ch);
                        self.s.set_ch2(ch2);
                    }

                    delta_x = self.m.x.get();
                    delta_y = self.m.y.get();

                    x = round(matka + qx);
                    y = round(kor);

                    if x >= 160 && x < 864 {
                        sx += x - fx;
                    }
                    if y >= 100 && y < 412 {
                        sy += y - fy;
                    }

                    self.m.x.set(sx);
                    self.m.y.set(sy);

                    if self.m.x.get() > 704 {
                        self.m.x.set(704);
                    }
                    if self.m.y.get() > 312 {
                        self.m.y.set(312);
                    }

                    self.m.tulosta();

                    self.draw_lumi(
                        delta_x - self.m.x.get(),
                        delta_y - self.m.y.get(),
                        self.tuuli.value.get(),
                        self.lmaara,
                        true,
                    );

                    if self.goals && goalx > 0 {
                        self.g
                            .draw_anim(goalx - self.m.x.get(), goaly - self.m.y.get(), 66);
                        //{ tavoitekeppi }
                    }
                    if wrx > 0 {
                        self.g
                            .draw_anim(wrx - self.m.x.get(), wry - self.m.y.get(), 68);
                        //{ m�kienkkakeppi }
                    }

                    self.g.draw_anim(
                        ux - self.m.x.get(),
                        round(ukor) - self.m.y.get() - 2,
                        jumper_anim,
                    );
                    self.g
                        .draw_anim(x - self.m.x.get(), y - self.m.y.get() - 1, ski_anim);

                    if cjumper {
                        self.g
                            .write_font(x - self.m.x.get(), y - self.m.y.get() - 20, b"C");
                    }

                    if x > 1050 {
                        out = true;
                    }

                    self.g.draw_anim(227, 2, 64); //{ hillrec sapluun }

                    self.g.e_write_font(308, 9, &namestr);

                    temp2 = 308;

                    self.g
                        .e_write_font(temp2, 33, &[&txtp(hp), b"\xab" as &[u8]].concat());

                    if hillrecord && !out {
                        if laskuri % 30 < 15 {
                            self.g.write_font(260, 33, b"HR!");
                        }
                        if random(2) == 0 {
                            // The Pascal reference build calls random for the y argument first
                            let yy = 32 + random(3) as i32;
                            self.g.e_write_font(
                                temp2 - 1 + random(3) as i32,
                                yy,
                                &[&txtp(hp), b"\xab" as &[u8]].concat(),
                            );
                        }
                    }

                    //{ tyylipisteet ruutuun }

                    temp = random(5) as i32 + 1;
                    if random(20) == 1 && tyylip[temp as usize] > 9999 {
                        tyylip[temp as usize] -= 10000;
                    }

                    for temp in 1..=5 {
                        if tyylip[temp as usize] < 9999 {
                            self.g.e_write_font(
                                308 - (temp - 1) * 24,
                                21,
                                &txtp(tyylip[temp as usize]),
                            );
                        }
                    }

                    rd[0][rturns as usize] = (128 + x - fx) as u8;
                    rd[1][rturns as usize] = (128 + y - fy) as u8;
                    rd[2][rturns as usize] = jumper_anim;
                    rd[3][rturns as usize] = ski_anim;
                    rd[4][rturns as usize] = (self.tuuli.value.get() + 128) as u8;

                    rturns += 1;

                    self.g.draw_screen().await;

                    if out {
                        //{ LASKU LOPPUU }
                        break;
                    }
                }
            }

            fx = 0; //{ k�ytet��n effektiivisten tyylipisteiden l�yt�miseen :) }
            fy = 0;

            for temp in 1..=5 {
                //{ tyylipisteet n�ytt��n ja poisj��v�t tummaksi }
                if tyylip[temp as usize] > 9999 {
                    tyylip[temp as usize] -= 10000;
                }

                if draw && (self.gdetail == 0 || !cjumper) {
                    self.g.font_color(247);
                }

                if tyylip[temp as usize] == tyylip[6] && fx == 0 {
                    if draw && (self.gdetail == 0 || !cjumper) {
                        self.g.font_color(252);
                    }
                    fx += 1;
                }

                if tyylip[temp as usize] == tyylip[7] && fy == 0 {
                    if draw && (self.gdetail == 0 || !cjumper) {
                        self.g.font_color(252);
                    }
                    fy += 1;
                }

                if draw {
                    self.g
                        .e_write_font(308 - (temp - 1) * 24, 21, &txtp(tyylip[temp as usize]));
                }
            }

            if self.jcup {
                if draw && (self.gdetail == 0 || !cjumper) {
                    self.g.font_color(241);
                }

                if draw {
                    self.g
                        .e_write_font(302, 14, &self.i.jnimet.borrow()[team as usize]);
                }
            }

            if draw && (self.gdetail == 0 || !cjumper) {
                self.g.font_color(240);
            }

            if draw {
                self.g.e_write_font(308, 9, &namestr);
            } else {
                self.g.e_write_font(296, 9, self.l.lstr(57));
                self.g.e_write_font(308, 9, &txt(index)); //{ computers jumping... }
            }

            if draw && (self.gdetail == 0 || !cjumper) {
                self.g.font_color(247);
            }

            fx = 0; //{ seuraavassa: osallistujat }
            fy = 0; //{ - " - : sijoitus }

            temp2 = 0;

            if !self.koth && self.kierros >= 0 {
                //{ komea "monesko olen nyt?"-laskuri! }
                fx = NUM_PL as i32;
                if self.jcup {
                    fx = NUM_TEAMS as i32;
                }

                for temp in 1..=fx {
                    if self.jcup {
                        temp2 = self.pisteet[team as usize]; //{ omat pisteet }
                    } else {
                        temp2 = self.pisteet[pel as usize];
                    }

                    if temp2 >= self.pisteet[temp as usize] {
                        fy += 1;
                    }
                }

                if draw {
                    self.g.e_write_font(
                        255,
                        45,
                        &[b"($" as &[u8], &txt(fx - fy + 1), b".)"].concat(),
                    );
                }
            }

            fx = 308;

            if draw && hillrecord {
                self.g.write_font(260, 33, b"HR!");
                self.g.font_color(246);
            }

            if draw {
                self.g
                    .e_write_font(fx, 33, &[&txtp(hp), b"\xab" as &[u8]].concat());
            }

            if draw && (self.gdetail == 0 || !cjumper) {
                self.g.font_color(246);
            }

            temp2 = score;

            if self.jcup {
                temp2 = self.pisteet[team as usize];
            }
            if self.wcup || self.koth {
                temp2 = self.pisteet[pel as usize];
            }

            if self.wcup && self.kierros < 0 {
                temp2 = score;
            }

            if draw {
                self.g.e_write_font(308, 45, &txtp(temp2)); //{ score }
                if self.gdetail == 0 || !cjumper {
                    self.g.font_color(241);
                }

                if draw && self.kierros == 2 && self.wcup {
                    self.g.e_write_font(
                        255,
                        33,
                        &[&txtp(self.cstats[1][pel as usize]), b"\xab" as &[u8]].concat(),
                    );
                    self.g.e_write_font(
                        311,
                        55,
                        &[b"(" as &[u8], &txtp(score), b")" as &[u8]].concat(),
                    );
                }

                if self.jcup && index * self.kierros > 1 {
                    self.g.e_write_font(
                        311,
                        55,
                        &[b"(" as &[u8], &txtp(score), b")" as &[u8]].concat(),
                    );
                }

                if self.inj[pel as usize] > 0 && self.wcup {
                    self.g.font_color(239);

                    str1 = match self.inj[pel as usize] {
                        1 => self.l.lstr(77).to_vec(),
                        2 => self.l.lstr(78).to_vec(),
                        _ => [
                            &txt(self.inj[pel as usize] as i32) as &[u8],
                            b" ",
                            &self.l.lstr(76),
                        ]
                        .concat(),
                    };

                    self.g.e_write_font(
                        311,
                        64,
                        &[&self.l.lstr(75) as &[u8], b" ", &str1].concat(),
                    );
                }
            }

            if !draw && self.gdetail == 0 {
                self.draw_lumi(
                    delta_x - self.m.x.get(),
                    delta_y - self.m.y.get(),
                    self.tuuli.value.get(),
                    self.lmaara,
                    false,
                ); //{ pakko piirt�� }
            }

            if draw
                && !cjumper
                && grade > 0
                && self.i.profile.borrow()[actprofile as usize].cstyle > 0
            {
                self.u.do_coach_corner(
                    height,
                    kulmalaskuri,
                    grade,
                    ponnistus as u8,
                    self.i.profile.borrow()[actprofile as usize].cstyle,
                );
            }

            rturns -= 1; //{ niit� on yksi liikaa }

            //{$IFDEF REG}
            if !cjumper && draw {
                self.g.e_write_font(308, 73, self.l.lstr(298));
            }
            //{$ENDIF}

            self.g.draw_screen().await;

            if draw {
                self.s.putsaa();
                self.cupslut = self.s.wait_for_key2().await;
            } else if self.s.key_pressed().await {
                self.s.wait_for_key_press().await;
                if self.s.get_ch() == 0 && self.s.get_ch2() == 68 {
                    self.cupslut = true;
                }
            }
        }

        self.m.lopeta();

        // TODO: Replay writing stuff not converted
        /*
         { KIRJOITA REPLAY! }

        {$IFDEF REG}
          if (draw) and (not cjumper) then
           begin
            str1:=profile[actprofile].realname;
            if (str1='') then str1:=profile[actprofile].name;

            if (upcase(ch)=lch(298,2)) then
             begin
              temp:=replayinfo(replayfilename, str1, replayname, nytmaki, hp);
              if (temp=0) then temp:=writereplay(str1, replayname);
              resultbox(1,temp);
             end;

            if (hillrecord) and (automatichrr) then
             begin
              replayfilename:='HR-'+copy(acthill.name,1,3);
              replayname:='HILL RECORD AT '+acthill.name+' K'+txt(acthill.kr);
              writereplay(str1,replayname);
             end;
           end;
        {$ENDIF}
                 */
    }

    async fn showstats(&mut self) {
        let mut count: i32;
        let mut temp: i32;
        let mut yy: i32;
        let mut xx: i32;
        let mut player: i32;
        let mut col: [i32; 21] = [0; 21];

        let ws = |cln: u8, clr: u8, str2: &[u8], yy: i32, col: &[i32]| {
            let color = if clr == 0 { 240 + (cln % 2) * 6 } else { clr };
            self.g.font_color(color);
            if cln == 1 {
                self.g.write_font(col[cln as usize], yy, str2);
            } else {
                self.g.e_write_font(col[cln as usize], yy, str2);
            }
        };

        if self.jcup {
            for team in 0..=self.jmaara - 1 {
                self.g.new_screen(1, 5).await;

                self.g.font_color(240);
                self.g.write_font(
                    30,
                    6,
                    &[
                        &self.l.lstr(93) as &[u8],
                        b" ",
                        &self.i.jnimet.borrow()[(15 - team) as usize],
                    ]
                    .concat(),
                );

                count = 0;

                col[0] = 11;
                col[1] = 12;
                col[2] = 62;
                col[3] = 90;
                col[4] = 103;

                xx = 0;

                for temp in 1..=3 {
                    xx += 70;
                    col[(temp * 3) + 2] = col[2] + xx;
                    col[(temp * 3) + 3] = col[3] + xx;
                    col[(temp * 3) + 4] = col[4] + xx;
                }

                yy = 23;

                self.g.font_color(241);

                self.g.write_font(col[1], yy, self.l.lstr(106));

                for temp in 0..=3 {
                    self.g.e_write_font(col[2 + temp * 3], yy, self.l.lstr(108));
                    self.g.e_write_font(col[3 + temp * 3], yy, self.l.lstr(109));
                    self.g.e_write_font(col[4 + temp * 3], yy, b"?");
                }

                self.g.font_color(247);
                yy += 7;

                for temp in 1..=4 {
                    self.g.write_font(
                        col[2 + (temp - 1) * 3] - 15,
                        yy,
                        &self.g.nsh(
                            &self.i.nimet.borrow()[NUM_PL + 1 - (team * 4) as usize - temp],
                            64,
                        ),
                    );
                }

                yy += 7;

                for cup in 1..=self.osakilpailu {
                    ws(0, 240, &[&txt(cup) as &[u8], b"."].concat(), yy, &col);

                    temp = self.hill_order[cup as usize];

                    let hillname = self.u.hillname(temp);
                    ws(
                        1,
                        246,
                        &[
                            &hillname[..hillname.len().min(3)] as &[u8],
                            b" ",
                            &txt(self.u.hillkr(temp)),
                        ]
                        .concat(),
                        yy,
                        &col,
                    );

                    for player in 1..=4 {
                        //{ b on player, a on cup, d on team }
                        temp = (player + team * 4) as i32;
                        xx = ((player - 1) * 3) as i32;

                        yy = (cup - 1) * 21 + 37;

                        for kierros in 1..=2 {
                            ws(
                                (2 + xx) as u8,
                                246,
                                &[
                                    &txt(self.stats[temp as usize][cup as usize].round_len
                                        [kierros as usize])
                                        as &[u8],
                                    b".",
                                ]
                                .concat(),
                                yy,
                                &col,
                            );
                            ws(
                                (3 + xx) as u8,
                                240,
                                &txtp(
                                    self.stats[temp as usize][cup as usize].round_pts
                                        [kierros as usize],
                                ),
                                yy,
                                &col,
                            );
                            ws(
                                (4 + xx) as u8,
                                247,
                                makeletter(
                                    self.stats[temp as usize][cup as usize].reason[kierros as usize]
                                        as i32,
                                ),
                                yy,
                                &col,
                            );
                            yy += 7;
                        }
                    }

                    if self.stats[team as usize][cup as usize].comp_pos < 9 {
                        count +=
                            TEAM_POINTS[self.stats[team as usize][cup as usize].comp_pos as usize];
                    }

                    self.g.font_color(247);
                    self.g.write_font(col[1], yy, self.l.lstr(107));
                    self.g.e_write_font(82, yy, self.l.lstr(108));
                    self.g.font_color(246);
                    self.g.e_write_font(
                        100,
                        yy,
                        &[
                            &txt(self.stats[team as usize][cup as usize].comp_pos as i32)
                                as &[u8],
                            b".",
                        ]
                        .concat(),
                    );

                    self.g.font_color(247);
                    self.g.e_write_font(122, yy, self.l.lstr(110));

                    let str1 = if count > 0 {
                        [
                            &txt(self.stats[team as usize][cup as usize].wc_pos as i32) as &[u8],
                            b".",
                        ]
                        .concat()
                    } else {
                        b"- ".to_vec()
                    };

                    self.g.font_color(246);
                    self.g.e_write_font(140, yy, &str1);

                    self.g.font_color(247);
                    self.g.e_write_font(162, yy, self.l.lstr(109));
                    self.g.font_color(246);
                    self.g.e_write_font(186, yy, &txt(count));

                    yy += 7;
                }

                self.g.draw_screen().await;
                self.cupslut = self.s.wait_for_key2().await;
            }
        } else {
            player = 1;

            self.ls
                .reset_list(
                    self.i.pmaara.get() as i32,
                    NUM_PL as i32,
                    7,
                    self.l.lstr(89),
                    self.inv_back,
                )
                .await;

            loop {
                self.g.font_color(240);

                self.g.write_font(
                    36 + self.g.font_len(self.l.lstr(89)),
                    6,
                    &self
                        .g
                        .nsh(&self.i.nimet.borrow()[NUM_PL + 1 - player as usize], 110),
                );

                count = 0;

                col[0..=12].copy_from_slice(&[
                    15, 16, 70, 90, 110, 140, 170, //{ R1 }
                    210, //{ m }
                    230, //{ pos after R1 }
                    240, 268, //{ R2 }
                    308, //{ m }
                    316,
                ]);

                yy = 23;

                self.g.font_color(247);

                self.g.write_font(col[1], yy, self.l.lstr(106));
                self.g.e_write_font(col[2], yy, self.l.lstr(108));
                self.g.e_write_font(col[3], yy, self.l.lstr(109));
                self.g.e_write_font(col[4], yy, self.l.lstr(98));
                self.g.e_write_font(col[5], yy, self.l.lstr(97));
                self.g.e_write_font(col[6], yy, b"R 1");
                self.g.e_write_font(col[8], yy, self.l.lstr(108));
                self.g.e_write_font(col[9], yy, b"?");
                self.g.e_write_font(col[10], yy, b"R 2");
                self.g.e_write_font(col[12], yy, b"?");

                self.g.font_color(246);

                yy = 25;

                for cup in 1..=self.osakilpailu {
                    yy += 7;

                    ws(0, 0, &[&txt(cup) as &[u8], b"."].concat(), yy, &col);

                    let hillname = self.u.hillname(cup);
                    ws(
                        1,
                        0,
                        &[
                            &hillname[..hillname.len().min(3)],
                            b" ",
                            &txt(self.u.hillkr(cup)),
                        ]
                        .concat(),
                        yy,
                        &col,
                    );

                    let mut str1 = b"dq".to_vec(); //{ didn't qualify }

                    temp = self.stats[player as usize][cup as usize].comp_pos as i32;

                    if temp > 0 && temp < 51 {
                        str1 = [&txt(temp) as &[u8], b"."].concat();
                    }

                    ws(2, 0, &str1, yy, &col); //{ Kisa Pos }

                    str1 = b"0".to_vec();
                    if temp > 0 && temp < 31 {
                        str1 = txt(WC_POINTS[temp as usize] as i32);
                        count += WC_POINTS[temp as usize] as i32;
                    }

                    ws(3, 0, &str1, yy, &col); //{ Kisasta pisteit� }

                    str1 = if count > 0 {
                        [
                            &txt(self.stats[player as usize][cup as usize].wc_pos as i32)
                                as &[u8],
                            b".",
                        ]
                        .concat()
                    } else {
                        b"-".to_vec()
                    };

                    ws(4, 0, &str1, yy, &col); //{ WC sija t�n skaban j�lkeen }

                    if temp > 0 && temp < 51 {
                        ws(
                            5,
                            0,
                            &txtp(
                                self.stats[player as usize][cup as usize].round_pts[1]
                                    + self.stats[player as usize][cup as usize].round_pts[2],
                            ),
                            yy,
                            &col,
                        );
                        ws(
                            6,
                            0,
                            &txtp(self.stats[player as usize][cup as usize].round_pts[1]),
                            yy,
                            &col,
                        );
                        str1 = [
                            b"(",
                            &txtp(self.stats[player as usize][cup as usize].round_len[1])
                                as &[u8],
                            b"\xab)",
                        ]
                        .concat();
                        ws(7, 247, &str1, yy, &col);

                        ws(
                            8,
                            0,
                            &[
                                &txt(self.stats[player as usize][cup as usize].round1_pos as i32)
                                    as &[u8],
                                b".",
                            ]
                            .concat(),
                            yy,
                            &col,
                        );
                    }

                    temp = self.stats[player as usize][cup as usize].round_pts[2] as i32;

                    if temp > 0 {
                        ws(10, 0, &txtp(temp), yy, &col);
                        ws(
                            11,
                            247,
                            &[
                                b"(",
                                &txtp(self.stats[player as usize][cup as usize].round_len[2])
                                    as &[u8],
                                b"\xab)",
                            ]
                            .concat(),
                            yy,
                            &col,
                        );
                    }

                    temp = self.stats[player as usize][cup as usize].reason[1] as i32;
                    if temp > 0 {
                        ws(9, 247, makeletter(temp), yy, &col);
                    }

                    temp = self.stats[player as usize][cup as usize].reason[2] as i32;
                    if temp > 0 {
                        ws(12, 247, makeletter(temp), yy, &col);
                    }

                    if cup == 12 && self.osakilpailu != 12 {
                        yy += 7;

                        if !self.inv_back {
                            self.g.fill_box(0, yy as u16, 319, (yy + 6) as u16, 245);
                            self.g.fill_area(0, yy as u16, 319, (yy + 6) as u16, 63);
                        }

                        self.g.font_color(240);
                        self.g.write_font(col[1], yy, self.l.lstr(99));

                        self.g.e_write_font(
                            col[2],
                            yy,
                            &[
                                &txt(self.stats[player as usize][0].comp_pos as i32) as &[u8],
                                b".",
                            ]
                            .concat(),
                        );

                        self.g.e_write_font(
                            col[5],
                            yy,
                            &txtp(self.stats[player as usize][0].round_pts[1]),
                        );
                    }
                }

                yy += 8;

                self.g.fill_box(2, yy as u16, 316, yy as u16, 241);
                self.g
                    .fill_box(3, (yy + 1) as u16, 317, (yy + 1) as u16, 242);

                yy += 3;

                ws(1, 0, &self.l.lstr(100), yy, &col);

                ws(3, 0, &txt(count), yy, &col);

                if count > 0 {
                    ws(
                        4,
                        0,
                        &[
                            &txt(self.stats[player as usize][self.osakilpailu /* was: cup (but it's not defined) */ as usize].wc_pos as i32)
                                as &[u8],
                            b".",
                        ]
                            .concat(),
                        yy,
                        &col,
                    );
                }

                count = 0;
                for cup in 1..=self.osakilpailu {
                    count += self.stats[player as usize][cup as usize].round_pts[1]
                        + self.stats[player as usize][cup as usize].round_pts[2];
                }

                self.g.font_color(251);
                self.g
                    .e_write_font(col[5], yy, &txtp(count / self.osakilpailu));

                count = 0;
                for cup in 1..=self.osakilpailu {
                    count += self.stats[player as usize][cup as usize].round_pts[1];
                }
                self.g.font_color(241);
                self.g
                    .e_write_font(col[6], yy, &txtp(count / self.osakilpailu));

                count = 0;
                for cup in 1..=self.osakilpailu {
                    count += self.stats[player as usize][cup as usize].round_pts[2];
                }
                self.g
                    .e_write_font(col[10], yy, &txtp(count / self.osakilpailu));

                let str1 = if player == self.i.pmaara.get() as i32 {
                    b"L"
                } else {
                    b"X"
                };
                player = self.ls.entry(player, 0, 0, b"", 0, 0, 0, str1).await;

                if player == -2 {
                    self.cupslut = true;
                }

                if player < 0 || player > self.i.pmaara.get() as i32 {
                    break;
                }
            }
        }
    }

    async fn dokothrecords(&self, high: u8) {
        let mut yy: i32;
        let mut col: [i32; 5] = [0; 5];
        let mut str1: Vec<u8>;

        col[1] = 30;
        col[2] = 55;
        col[3] = 175;
        col[4] = 290;

        self.g.font_color(240);
        self.g.write_font(30, 6, self.l.lstr(160));

        yy = 12;

        for temp in 1..=6 {
            yy += 18;

            self.g.font_color(246);

            self.g.write_font(col[1], yy, &self.u.kothtitle(temp));

            yy += 10;

            self.g.font_color(241);
            str1 = self.l.lstr(161).to_vec();

            let hi = self.i.top.borrow()[(temp + 35) as usize].clone();
            if hi.score > 0 {
                self.g.write_font(col[3], yy, &hi.time);

                if high == 0 || high as i32 == temp {
                    self.g.font_color(240);
                }
                str1 = hi.name;
                self.g
                    .write_font(col[4], yy, &[&txt(hi.score) as &[u8], b" X"].concat());
                //{ times completed }
            }

            self.g.write_font(col[2], yy, &str1);
        }

        self.g.draw_screen().await;
    }

    //{ h0 = main menusta,muuten kertoo mik� highlightataan }
    async fn dolist(&mut self, high: u8, phase: u8) {
        let mut yy: i32;
        let entries: u8;
        let start: u8;
        let mut sortby: u8;
        let mut str1: Vec<u8>;
        let mut col: [i32; 6] = [0; 6];

        let mut hi: Hiscore;
        let mut num_customs: i32 = 0;
        let mut files: Vec<Vec<u8>> = vec![Vec::new(); MAX_CUSTOMS as usize + 1];

        yy = 6;

        col[0] = 30;
        col[1] = 146;
        col[2] = 173;
        col[3] = 215;

        self.g.font_color(240);
        sortby = 0; //{ 0-WC, 1 - Total points }

        match phase {
            0 => {
                str1 = self.l.lstr(163).to_vec();
                entries = 20;
                start = 1;
            }
            1 => {
                str1 = self.l.lstr(164).to_vec();
                entries = 10;
                start = 21;
            }
            2 => {
                str1 = self.l.lstr(165).to_vec();
                entries = 5;
                start = 31;
                if high == 0 {
                    yy = 126;
                }
                sortby = 1;
            }
            _ => {
                self.i.find_customs(&mut num_customs, &mut files);
                str1 = self.l.lstr(162).to_vec();
                start = (phase - 3).wrapping_mul(20);
                entries = (num_customs as u8).wrapping_sub(start).min(20);
                col[0] = 50;
                col[1] = 157;
                col[2] = 184;
            }
        } //{case}

        self.g.write_font(30, yy, &str1);

        yy += 17;

        self.g.font_color(246);

        if phase > 2 {
            self.g.write_font(5, yy, self.l.lstr(173)); //{ file }
            self.g.write_font(col[0], yy, self.l.lstr(171)); //{ name -> who? }
            self.g.e_write_font(col[1] + 10, yy, self.l.lstr(167));
            self.g.write_font(col[2] + 9, yy, self.l.lstr(109)); //{ points -> pts }
        } else {
            //{ muutoin normaalisti }
            self.g.write_font(col[0], yy, self.l.lstr(166));
            self.g.write_font(col[1], yy, self.l.lstr(167));
            self.g.write_font(col[2], yy, self.l.lstr(168));
        }

        self.g.write_font(col[3], yy, self.l.lstr(169));

        for temp in start as i32..=start as i32 + entries as i32 - 1 {
            yy += 8;

            hi = self.i.top.borrow()[temp as usize].clone();

            if phase > 2 {
                self.i.load_custom(
                    &files[(temp - start as i32 + 1) as usize],
                    &mut sortby,
                    &mut self.hill_order,
                    &mut self.cup_hills,
                );
                hi = self.i.top.borrow()[42].clone();
            }

            if high == 0 {
                self.g.font_color(246);
                if phase < 3 {
                    self.g.e_write_font(
                        24,
                        yy,
                        &[&txt(temp - start as i32 + 1) as &[u8], b". "].concat(),
                    );
                }

                self.g.font_color(240);
                if hi.name.last() == Some(&b'\xff') {
                    self.g.font_color(247);
                }
            } else {
                self.g.font_color(241);
                if temp - start as i32 + 1 == high as i32 {
                    self.g.font_color(240);
                }
                self.g.e_write_font(
                    24,
                    yy,
                    &[&txt(temp - start as i32 + 1) as &[u8], b". "].concat(),
                );
            }

            self.g.write_font(col[0], yy, &self.g.nsh(&hi.name, 110));
            self.g.e_write_font(
                col[1] + 14,
                yy,
                &[&txt(hi.pos as i32) as &[u8], b"."].concat(),
            );

            str1 = txt(hi.score);
            if sortby == 1 {
                str1 = txtp(hi.score);
            }
            self.g.e_write_font(col[2] + 24, yy, &str1);

            self.g.font_color(241);
            self.g.write_font(col[3], yy, &hi.time);
            if phase > 2 {
                self.g
                    .write_font(5, yy, &files[(temp - start as i32 + 1) as usize]);
            }
        }

        self.g.draw_screen().await;
    }

    //{ phase: 0 - normals, 1...  - extras }
    async fn dohillrec(&self, phase: u8) {
        let mut yy: i32;
        let start: i32;
        let mut loop_: i32;
        let mut apu1: i32;
        let mut str1: Vec<u8>;
        let mut col: [i32; 6] = [0; 6];
        let mut ahi: i32;
        let mut laskuri: i32;

        col[0] = 3;
        col[1] = 71;
        col[2] = 183;
        col[3] = 200;
        col[4] = 216;

        start = phase as i32 * 20;

        self.g.font_color(240);

        str1 = self.l.lstr(156).to_vec();

        ahi = 0;

        if phase == 0 {
            str1 = self.l.lstr(170).to_vec();
        }

        loop_ = NUM_WC_HILLS + self.u.num_extra_hills.get() as i32 - start;
        if loop_ > 20 {
            loop_ = 20;
        }

        self.g.write_font(30, 6, &str1);

        self.g.write_font(col[0], 23, self.l.lstr(106));
        self.g.write_font(col[1], 23, self.l.lstr(171));
        self.g.e_write_font(col[2], 23, self.l.lstr(172));
        self.g.write_font(col[3], 23, b"(K)");
        self.g.write_font(col[4], 23, self.l.lstr(169));

        for aa in 1..=loop_ {
            yy = (aa - 1) * 8 + 32;

            self.g.font_color(246);
            self.g
                .write_font(col[0], yy, &self.g.nsh(&self.u.hillname(aa + start), 64));

            self.g.font_color(240);
            str1 = self.u.hrname(aa + start);

            apu1 = self.u.hrlen(aa + start);

            if str1.last() == Some(&b'\xff') {
                self.g.font_color(247);
                apu1 = self.u.hillkr(aa + start) * 10;
            }

            ahi += apu1;

            self.g.write_font(col[1], yy, &self.g.nsh(&str1, 80));
            if str1.last() != Some(&b'\xff') {
                self.g.font_color(246);
            }
            self.g
                .e_write_font(col[2], yy, &txtp(self.u.hrlen(aa + start)));
            self.g.e_write_font(
                col[3] + 11,
                yy,
                &[b"(" as &[u8], &txt(self.u.hillkr(aa + start)), b")"].concat(),
            );

            self.g.font_color(241);

            self.g.write_font(col[4], yy, &self.u.hrtime(aa + start));
        }

        if phase == 0 {
            laskuri = 0;
            for apu1 in 1..=NUM_WC_HILLS {
                laskuri += self.u.hillkr(apu1) * 10;
            }

            ahi = round(((ahi as f64 / laskuri as f64) * 10000.0) as f32);
            str1 = txt(ahi);
            let index = str1.len() - 1;
            insert(b".", &mut str1, index);
            str1.extend_from_slice(b" %");

            self.g.write_font(130, 192, b"A.H.I.");
            self.g.e_write_font(197, 192, &str1);
        }

        self.g.draw_screen().await;
    }

    //{ phase: 0 - hiscores, 1 - hillrecs+extras, 2 - just hillrecs }
    async fn showtops(&mut self, phase: u8) {
        let mut index: i32;
        let mut num: i32;
        let mut numcustoms: i32 = 0;
        let mut str1: Vec<u8>;
        let mut good: bool;
        let mut leave: bool;
        let mut filenames: Vec<Vec<u8>> = vec![Vec::new(); MAX_CUSTOMS as usize + 1];

        index = 1;
        num = 4;

        if phase == 0 {
            self.i.find_customs(&mut numcustoms, &mut filenames);
        }

        match phase {
            0 => {
                if numcustoms > 0 {
                    num = (numcustoms - 1) / 20 + 4;
                } else {
                    num = 3;
                }
            }
            1 => {
                num = (self.u.num_extra_hills.get() as i32 - 1) / 20 + 2;
            }
            2 => {
                num = 1;
            }
            _ => {}
        }

        loop {
            match phase {
                0 => match index {
                    1 => {
                        self.g.new_screen(1, 0).await;
                        self.dolist(0, 0).await;
                    }
                    2 => {
                        self.g.new_screen(4, 0).await;
                        self.dolist(0, 1).await;
                        self.dolist(0, 2).await;
                    }
                    3 => {
                        self.g.new_screen(1, 2).await;
                        self.dokothrecords(0).await;
                    }
                    _ => {
                        self.g.new_screen(1, 0).await;
                        self.dolist(0, (index - 1) as u8).await;
                    }
                },
                1 => {
                    if index == 1 {
                        self.g.new_screen(1, 0).await;
                        self.dohillrec(0).await;
                    } else {
                        self.g.new_screen(1, 5).await;
                        self.dohillrec((index - 1) as u8).await;
                    }
                }
                2 => {
                    self.g.new_screen(1, 0).await;
                    self.dohillrec(0).await;
                }
                _ => {}
            }

            self.g.font_color(241);
            if index > 1 {
                self.g
                    .e_write_font(319, 5, &[b"(-" as &[u8], &self.l.lstr(246)].concat());
            }

            str1 = self.l.lstr(247).to_vec();
            if index == num {
                str1 = self.l.lstr(248).to_vec();
            }

            self.g
                .e_write_font(319, 13, &[&str1 as &[u8], b"-)"].concat());

            self.g.draw_screen().await;

            leave = false;

            loop {
                good = false;

                let (ch, ch2) = self.s.wait_for_key_press().await;
                self.s.set_ch(ch);
                self.s.set_ch2(ch2);

                match ch {
                    27 => {
                        index = -1;
                        good = true;
                    }
                    b' ' | 13 => {
                        index += 1;
                        good = true;
                    }
                    3 => {
                        //{ CTRL-C }
                        index = -2;
                        good = true;
                    }
                    _ => {}
                } //{ case }

                if ch == 0 {
                    match ch2 {
                        68 | 45 => {
                            //{ ALT-X, F10 }
                            index = -2;
                            good = true;
                        }
                        77 | 81 => {
                            //{ PgDn, A_Right }
                            index += 1;
                            good = true;
                        }
                        71 => {
                            //{ HOME }
                            if index > 1 {
                                index = 1;
                                good = true;
                            }
                        }
                        73 | 75 => {
                            //{ PgUp, A_Left }
                            if index > 1 {
                                index -= 1;
                                good = true;
                            }
                        }
                        _ => {}
                    } //{ case }
                }

                if good {
                    break;
                }
            }

            if index < 0 || index > num {
                leave = true;
            }

            if leave {
                break;
            }
        }
    }

    //{ vaiheet: 0 - WCquali  1 - WC   2 - 4hills }
    fn donewheader(&self, vaihe: u8) -> Vec<u8> {
        match vaihe {
            4 => {
                if self.osakilpailu == self.cup_hills {
                    [
                        &self.l.lstr(90) as &[u8],
                        b" ",
                        &self.l.lstr(27 + self.cup_style as u32),
                    ]
                    .concat()
                } else {
                    [
                        &self.l.lstr(27 + self.cup_style as u32) as &[u8],
                        b" ",
                        &self.l.lstr(87),
                        b" ",
                        &txt(self.osakilpailu),
                        b" ",
                        &self.l.lstr(8),
                        b" ",
                        &txt(self.cup_hills),
                    ]
                    .concat()
                }
            }

            3 => {
                match self.cup_style {
                    0 | 2 => {
                        if self.osakilpailu == 12 || self.osakilpailu == self.cup_hills {
                            self.l.lstr(85).to_vec()
                        } else {
                            let mut temp3 = self.osakilpailu;
                            if self.osakilpailu > 4 {
                                temp3 = self.osakilpailu - 8; //{ joko yksin��n tai osana WC:t� }
                            }
                            [
                                &self.l.lstr(84) as &[u8],
                                b" ",
                                &txt(temp3),
                                b" ",
                                &self.l.lstr(83),
                                b" - ",
                                &self.act_hill.name,
                                b" K",
                                &txt(self.act_hill.kr),
                            ]
                            .concat()
                        }
                    }
                    1 => {
                        if self.osakilpailu == self.cup_hills {
                            [&self.l.lstr(90) as &[u8], b" ", &self.l.lstr(28)].concat()
                        } else {
                            [
                                &self.l.lstr(28) as &[u8],
                                b" ",
                                &self.l.lstr(87),
                                b" ",
                                &txt(self.osakilpailu),
                                b" ",
                                &self.l.lstr(8),
                                b" ",
                                &txt(self.cup_hills),
                            ]
                            .concat()
                        }
                    }
                    _ => panic!("Invalid cup style: {}", self.cup_style),
                }
            }
            0 | 1 | 2 => {
                let str1 = if vaihe == 0 {
                    self.l.lstr(82)
                } else {
                    self.l.lstr(81)
                };
                let mut str2 = if self.cup_style == 2 {
                    [&txt(self.osakilpailu) as &[u8], b" ", &self.l.lstr(83)].concat()
                } else {
                    [
                        &txt(self.osakilpailu) as &[u8],
                        b" ",
                        &self.l.lstr(8),
                        b" ",
                        &txt(self.cup_hills),
                    ]
                    .concat()
                };
                str2 = [
                    &str2 as &[u8],
                    b" - ",
                    &self.act_hill.name,
                    b" K",
                    &txt(self.act_hill.kr),
                ]
                .concat();
                if self.kierros > 0 {
                    str2 = [&str2 as &[u8], b" - R ", &txt(self.kierros)].concat();
                }
                [&str1 as &[u8], b" ", &str2].concat()
            }
            _ => panic!("Invalid vaihe: {}", vaihe),
        }
    }

    fn compactsign(&self) {
        self.g.font_color(241);
        self.g.write_font(30, 190, self.l.lstr(86));
    }

    //{ what: 0 - WCStats, 1 - normal, 2 - 4H }
    fn updatestats(&mut self, what: u8) {
        let mut who: i32;
        let mut victim: i32;
        let mut str1: Vec<u8>;

        for temp in 1..=NUM_PL {
            who = if what == 0 {
                self.mcluett[temp]
            } else {
                self.luett[temp]
            } as i32;

            if who > NUM_PL as i32 - self.i.pmaara.get() as i32 {
                //{ oma j�tk� }
                victim = NUM_PL as i32 + 1 - who;

                match what {
                    0 => {
                        if self.cup_style == 0 {
                            //{ WC pohjaisia statseja }
                            self.stats[victim as usize][self.osakilpailu as usize].wc_pos =
                                self.sija[who as usize];
                            if self.osakilpailu == self.cup_hills {
                                if self.sija[who as usize] == 1 {
                                    self.i.profile.borrow_mut()
                                        [self.i.profileorder.borrow()[victim as usize] as usize]
                                        .wcswon += 1;
                                }
                                self.i.profile.borrow_mut()
                                    [self.i.profileorder.borrow()[victim as usize] as usize]
                                    .wcs += 1;
                            }

                            if self.mcpisteet[who as usize]
                                >= self.i.profile.borrow()
                                    [self.i.profileorder.borrow()[victim as usize] as usize]
                                    .bestpoints as i32
                            {
                                self.i.profile.borrow_mut()
                                    [self.i.profileorder.borrow()[victim as usize] as usize]
                                    .bestpoints = self.mcpisteet[who as usize] as u16;
                                str1 = if self.osakilpailu == self.cup_hills {
                                    [&txt(self.sija[who as usize] as i32) as &[u8], b"."].concat()
                                } else {
                                    b"-".to_vec()
                                };
                                self.i.profile.borrow_mut()
                                    [self.i.profileorder.borrow()[victim as usize] as usize]
                                    .bestresult = [
                                    &txt(self.mcpisteet[who as usize]) as &[u8],
                                    b" (",
                                    &str1,
                                    b")",
                                ]
                                .concat();
                            }
                        }
                    }
                    1 => {
                        //{ perus kisanaikaisia statseja }
                        self.stats[victim as usize][self.osakilpailu as usize].comp_pos =
                            self.sija[who as usize];
                        if self.kierros == 1 {
                            self.stats[victim as usize][self.osakilpailu as usize].round1_pos =
                                self.sija[who as usize];
                        }
                        if self.cup_style == 0 && self.kierros == 2 && self.sija[who as usize] == 1
                        {
                            self.i.profile.borrow_mut()
                                [self.i.profileorder.borrow()[victim as usize] as usize]
                                .legswon += 1;
                        }
                    }

                    2 => {
                        //{ fourhills tulos talteen }
                        self.stats[victim as usize][0].round_pts[1] = self.fourpts[who as usize];
                        self.stats[victim as usize][0].comp_pos = self.sija[who as usize];
                    }

                    _ => {}
                }
            }
        }
    }

    //{ vaihe: 0 - WC, 1 - TC, 2 - 4H, 3 - KOTH, 4 - CC_MC, 5 - CC_4P }
    async fn sorthighs(&mut self, vaihe: u8) -> u8 {
        //{ highstart: 0 - WC, 20 - TC, 30 - 4H, 35 - KOTH, 41 - CC  }

        let mut localnosamename: bool = self.nosamename;
        let mut leave: bool = false;
        let mut who: i32;
        let mut players: i32 = self.i.pmaara.get() as i32;
        let mut pts: i32;
        let mut highstart: u8 = 0;
        let mut highlength: u8 = 20;
        let mut victimname: Vec<u8>;
        let mut return_: u8 = 0;
        let mut datestr: Vec<u8> = dayandtime(self.today, self.now);

        match vaihe {
            1 => {
                players = self.jmaara as i32;
                highstart = 20;
                highlength = 10;
            }
            2 => {
                highstart = 30;
                highlength = 5;
            }
            4 | 5 => {
                highstart = 41;
                localnosamename = true; //{ ei milloinkaan useampia nimi� }
            }
            _ => {}
        }

        for t1 in 1..=players {
            for t2 in 1..=highlength {
                if vaihe == 1 {
                    who = NUM_TEAMS as i32 + 1 - t1;
                    victimname = self.i.jnimet.borrow()[who as usize].clone();
                } else {
                    who = NUM_PL as i32 + 1 - t1;
                    victimname = self.i.nimet.borrow()[who as usize].clone();
                }

                pts = match vaihe {
                    0 | 1 | 4 => self.mcpisteet[who as usize],
                    2 | 5 => self.fourpts[who as usize],
                    _ => 0,
                };

                if pcomp(pts, self.sija[who as usize] as i32)
                    > pcomp(
                        self.i.top.borrow()[(t2 + highstart) as usize].score,
                        self.i.top.borrow()[(t2 + highstart) as usize].pos as i32,
                    )
                {
                    leave = true; //{ tekeek� nimet vai ei }

                    if localnosamename {
                        for t3 in 1..=t2 - 1 {
                            if self.i.top.borrow()[(t3 + highstart) as usize].name == victimname {
                                leave = false;
                            }
                        }
                        if leave {
                            //{ ei ollut parempaa samaa nime� }
                            let mut top = self.i.top.borrow_mut();
                            for t3 in t2..=highlength {
                                if top[(t3 + highstart) as usize].name == victimname {
                                    for t4 in t3..=highlength - 1 {
                                        top[(t4 + highstart) as usize] =
                                            top[(t4 + highstart + 1) as usize].clone();
                                    }
                                    break;
                                }
                            }
                        }
                    }

                    if leave {
                        if t2 < highlength {
                            let mut top = self.i.top.borrow_mut();
                            for t3 in ((t2 + 1)..=highlength).rev() {
                                top[(t3 + highstart) as usize] =
                                    top[(t3 + highstart - 1) as usize].clone();
                            }
                        }

                        if vaihe > 3 && self.cup_hills == 1 {
                            datestr = [
                                b"(",
                                &txtp(self.stats[t1 as usize][1].round_len[1]) as &[u8],
                                b"m-",
                                &txtp(self.stats[t1 as usize][1].round_len[2]),
                                b"m)",
                            ]
                            .concat();
                        }

                        {
                            let mut top = self.i.top.borrow_mut();
                            top[(t2 + highstart) as usize].score = pts;
                            top[(t2 + highstart) as usize].name = victimname;
                            top[(t2 + highstart) as usize].pos = self.sija[who as usize];
                            top[(t2 + highstart) as usize].time = datestr.clone();
                            let entry = &top[(t2 + highstart) as usize];
                            trace::record(
                                "TopUpdate",
                                &[
                                    ("index", (t2 + highstart).into()),
                                    ("name", (&entry.name as &[u8]).into()),
                                    ("score", entry.score.into()),
                                    ("pos", entry.pos.into()),
                                    ("time", (&entry.time as &[u8]).into()),
                                ],
                            );
                        }

                        return_ = t2; //{ pyrkii palauttamaan lis�tyn pelaajan sijan }

                        if vaihe < 4 {
                            self.g.new_screen(1, 0).await;
                            self.dolist(t2, vaihe).await;
                            self.u.wait_for_key3(305, 6).await;
                        }
                    }
                    break;
                }
            }
        }
        return_
    }

    async fn updaterecords(&mut self, mut sortby: u8) {
        let mut final_: bool = false;
        let mut cscreen: bool = false;
        let mut str1: Vec<u8>;
        let mut who: i32;
        let mut temp2: i32;
        let mut oldhi = Hiscore::default();

        if self.osakilpailu == self.cup_hills {
            final_ = true;
        }

        for t1 in 1..=self.i.pmaara.get() as i32 {
            who = NUM_PL as i32 + 1 - t1;

            //{ pari profilesin parasta tulosta }

            if ((self.cup_style == 0 && self.osakilpailu == 12) || (self.cup_style == 2 && final_))
                && self.fourpts[who as usize]
                    >= self.i.profile.borrow()[self.i.profileorder.borrow()[t1 as usize] as usize]
                        .best4points as i32
            {
                self.i.profile.borrow_mut()[self.i.profileorder.borrow()[t1 as usize] as usize]
                    .best4points = self.fourpts[who as usize] as u16;
                self.i.profile.borrow_mut()[self.i.profileorder.borrow()[t1 as usize] as usize]
                    .best4result = [
                    &txtp(self.fourpts[who as usize]) as &[u8],
                    b" (",
                    &txt(self.sija[who as usize] as i32),
                    b".)",
                ]
                .concat();
            }

            if self.cup_style == 0
                && self.mcpisteet[who as usize]
                    >= self.i.profile.borrow()
                        [self.i.profileorder.borrow()[NUM_PL + 1 - who as usize] as usize]
                        .bestpoints as i32
            {
                self.i.profile.borrow_mut()
                    [self.i.profileorder.borrow()[NUM_PL + 1 - who as usize] as usize]
                    .bestpoints = self.mcpisteet[who as usize] as u16;
                str1 = if final_ {
                    [&txt(self.sija[who as usize] as i32) as &[u8], b"."].concat()
                } else {
                    b"-".to_vec()
                };
                self.i.profile.borrow_mut()
                    [self.i.profileorder.borrow()[NUM_PL + 1 - who as usize] as usize]
                    .bestresult = [
                    &txt(self.mcpisteet[who as usize]) as &[u8],
                    b" (",
                    &str1,
                    b")",
                ]
                .concat();
            }

            if self.cup_style == 1 && final_ {
                self.i.load_custom(
                    &self.setfile,
                    &mut sortby,
                    &mut self.hill_order,
                    &mut self.cup_hills,
                );
                oldhi = self.i.top.borrow()[42].clone();
                temp2 = self.sorthighs(4 + sortby).await as i32;

                if temp2 == 1 && self.setfile != b"TEMP" {
                    cscreen = true;
                }

                if temp2 > 0 {
                    self.i
                        .write_custom(&self.setfile, sortby, &self.hill_order, self.cup_hills);
                }
            }
        }

        if !self.cupslut {
            match self.cup_style {
                0 => {
                    if self.osakilpailu == 12 {
                        self.sorthighs(2).await;
                    }
                    if final_ {
                        self.sorthighs(0).await;
                    }
                }
                1 => {
                    if cscreen {
                        self.i
                            .newcrecordscreen(
                                &self.setfile,
                                &self.i.top.borrow()[42],
                                &oldhi,
                                sortby,
                            )
                            .await;
                    }
                }
                2 => {
                    if final_ {
                        self.sorthighs(2).await;
                    }
                }
                _ => {}
            }
        }
    }

    fn shouldishow(&self, player: i32, vaihe: u8, templuett: &[u8]) -> u8 {
        let who2 = templuett[player as usize];

        let pts = match vaihe {
            0 | 1 | 2 => self.pisteet[who2 as usize],
            3 => self.fourpts[who2 as usize],
            4 => self.mcpisteet[who2 as usize],
            _ => panic!("Invalid vaihe"),
        };

        if who2 as usize > NUM_PL - self.i.pmaara.get() as usize {
            //{ oma j�tk� }
            match vaihe {
                0 => 1,
                1 | 2 if self.qual[who2 as usize] > 0 => 1,
                3 | 4 if pts > 0 => 1,
                _ => 0,
            }
        } else {
            0
        }
    }

    // { oldvaiheet: 0 - WCquali  1 - WC, 2 - 4hills }
    // { newvaiheet: 0 - WCquali, 1 - WCk1, 2 - WCk2, 3 - 4H, 4 - MClista }
    // { monta :         kaikki,    qualit,    qualit,  >4pts     >MCpist }
    async fn lista(&mut self, vaihe: u8) {
        let mut t1: i32 = 0;
        let mut t2: i32 = 0;
        let mut pts: i32;
        let mut valiviiva: bool;
        let mut leave: bool;
        let mut who: i32;
        let mut prev: i32;
        let mut next: i32;
        let mut temp: i32;
        let mut temp2: i32;
        let mut shown: i32;
        let mut needtoshow: i32 = 0; //{ monta omaa j�tk�� on edes tarkoitus n�ytt�� }
        let mut extra: Vec<u8>;
        let mut oknow: bool;
        let mut templuett: [u8; NUM_PL + 2] = [0; NUM_PL + 2];

        for temp in 1..=NUM_PL {
            //{ t��ll� k�ytet��n templuettia }
            if vaihe == 4 {
                templuett[temp] = self.mcluett[temp];
            } else {
                templuett[temp] = self.luett[temp];
            }
        }

        for temp in 1..=NUM_PL {
            needtoshow += self.shouldishow(temp as i32, vaihe, &templuett) as i32;
        }

        temp = match vaihe {
            1 | 2 => 1,
            3 => 2,
            4 => 3,
            _ => 0,
        };

        self.ls
            .reset_list(
                self.i.pmaara.get() as i32,
                NUM_PL as i32,
                temp,
                &self.donewheader(vaihe),
                self.inv_back,
            )
            .await;
        if self.compactlist {
            self.compactsign();
        }

        leave = false;
        valiviiva = false; //{ ei ole sit� piirretty viel� }

        temp = 1;
        shown = 0;

        loop {
            who = templuett[temp as usize] as i32;
            prev = templuett[(temp - 1) as usize] as i32;
            next = templuett[(temp + 1) as usize] as i32;

            extra = b"X".to_vec(); //{ ett� extra[1] on jotain j�rkev�� }

            temp2 = 0;

            oknow = false;

            if self.compactlist {
                //{ compakti tuloslista. }
                loop {
                    who = templuett[temp as usize] as i32;
                    prev = templuett[(temp - 1) as usize] as i32;
                    next = templuett[(temp + 1) as usize] as i32;

                    if self.shouldishow(temp, vaihe, &templuett) == 1 {
                        //{ t�� mun pitikin n�ytt�� }
                        shown += 1;
                        oknow = true;
                    } else {
                        //{ muu j�tk�, mutta kympin sis�ll� on my�s ok }
                        if !(self.sija[who as usize] > 10) {
                            oknow = true;
                        }
                    }

                    if shown == needtoshow && self.sija[next as usize] > 10 {
                        //{ kaikki omat n�ytetty ja seuraava ei kuulu en�� t�nne }
                        oknow = true;
                        extra = b"L".to_vec();
                    }

                    if !oknow {
                        temp += 1;
                    }

                    if oknow {
                        break;
                    }
                }
            }

            if self.sija[who as usize] > 30
                && !self.compactlist
                && self.sija[prev as usize] <= 30
                && vaihe == 1
            {
                //{ 1. kierron v�liviiva }
                temp2 = self.ls.entry(0, 0, 2, b"- - -", 0, 0, 0, b"X").await;
            }

            if temp2 >= 0 {
                match vaihe {
                    0 | 1 | 2 => {
                        t1 = self.pisteet[who as usize];
                        t2 = self.pisteet[templuett[1] as usize];
                    }
                    3 => {
                        t1 = self.fourpts[who as usize];
                        t2 = self.fourpts[templuett[1] as usize];
                    }
                    4 => {
                        t1 = self.mcpisteet[who as usize];
                        t2 = self.mcpisteet[templuett[1] as usize];
                    }
                    _ => {}
                }

                pts = t1;

                if self.diff && temp > 1 {
                    pts = t1 - t2;
                }

                if vaihe == 0 && self.qual[who as usize] > 0 {
                    extra.push(b'Q');
                    if self.sija[who as usize] > 50 {
                        extra.push(b'W');
                    }
                }

                if vaihe < 2 && self.sija[who as usize] <= 30 {
                    extra.push(b'Q');
                }

                if self.inj[who as usize] > 0 && vaihe < 3 {
                    extra = [
                        &extra as &[u8],
                        b"I",
                        &txt((self.inj[who as usize] - 1) as i32),
                    ]
                    .concat();
                }

                if vaihe < 3 && self.this_is_a_hill_record == (self.kierros * 1000) + who {
                    extra.push(b'R');
                }

                match vaihe {
                    //{ ett� tiedet��n loppuiko tulostettavat }
                    0 => {
                        if temp >= NUM_PL as i32 {
                            extra.insert(0, b'L');
                        }
                    }
                    1 | 2 => {
                        if self.qual[next as usize] == 0 {
                            extra.insert(0, b'L');
                        }
                    }
                    3 => {
                        if self.fourpts[next as usize] == 0 {
                            extra.insert(0, b'L');
                        }
                    }
                    4 => {
                        if self.mcpisteet[next as usize] == 0 {
                            extra.insert(0, b'L');
                        }
                    }
                    _ => {}
                }

                if next == 0 || next > NUM_PL as i32 {
                    extra.insert(0, b'L');
                }

                match vaihe {
                    0 => {
                        temp = self
                            .ls
                            .entry(
                                temp,
                                self.sija[who as usize] as i32,
                                who,
                                &self.i.nimet.borrow()[who as usize],
                                pts,
                                self.cstats[0][who as usize],
                                0,
                                &extra,
                            )
                            .await;
                    }
                    1 | 2 => {
                        temp = self
                            .ls
                            .entry(
                                temp,
                                self.sija[who as usize] as i32,
                                who,
                                &self.i.nimet.borrow()[who as usize],
                                pts,
                                self.cstats[1][who as usize],
                                self.cstats[2][who as usize],
                                &extra,
                            )
                            .await;
                    }
                    3 => {
                        temp = self
                            .ls
                            .entry(
                                temp,
                                self.sija[who as usize] as i32,
                                who,
                                &self.i.nimet.borrow()[who as usize],
                                pts,
                                0,
                                0,
                                &extra,
                            )
                            .await;
                    }
                    4 => {
                        if self.mcpisteet[who as usize] > 0 {
                            temp = self
                                .ls
                                .entry(
                                    temp,
                                    self.sija[who as usize] as i32,
                                    who,
                                    &self.i.nimet.borrow()[who as usize],
                                    pts,
                                    0,
                                    0,
                                    &extra,
                                )
                                .await;
                        }
                    }
                    _ => {}
                }

                if self.compactlist
                    && !valiviiva
                    && self.sija[next as usize] > 10
                    && self.sija[who as usize] <= 10
                {
                    valiviiva = true;
                    temp2 = self.ls.entry(0, 0, 2, b"...", 0, 0, 0, b"X").await;
                    //{ v�liviiva }
                }
            }

            if temp < 0 || temp2 < 0 {
                leave = true;
            }

            if leave {
                break;
            }
        }

        if temp == -2 {
            self.cupslut = true;
        }

        //{$IFDEF REG}
        if !self.cupslut
            && self.cup_style == 0
            && self.s.get_ch() != 27
            && vaihe == 4
            && (self.lct || self.osakilpailu == NUM_WC_HILLS)
        {
            self.showstats().await;
        }
        //{$ENDIF}
    }

    //{ num - monesko hyppääjä }
    async fn jlista(&mut self, num: i32) {
        let mut str1: Vec<u8>;
        let mut temp: i32;
        let mut who: i32;
        let mut pts: i32;
        let mut extra: Vec<u8>;
        let mut ldiff: bool;
        let mut leave: bool;

        temp = 5;
        ldiff = self.diff;
        str1 = [
            &self.l.lstr(81) as &[u8],
            b" ",
            &txt(self.osakilpailu),
            b" ",
            &self.l.lstr(8),
            b" 6 - R ",
            &txt(self.kierros),
            b" - ",
            &self.l.lstr(88),
            b" ",
            &txt(num),
        ]
        .concat();

        if num == 0 {
            //{ cuplista }
            temp = 6;
            ldiff = self.diffwc;
            str1 = [
                &self.l.lstr(91) as &[u8],
                b" ",
                &txt(self.osakilpailu),
                b" ",
                &self.l.lstr(8),
                b" 6",
            ]
            .concat();
            if self.osakilpailu == 6 {
                str1 = self.l.lstr(92).to_vec();
            }
        }

        self.ls
            .reset_list(
                self.jmaara as i32,
                NUM_TEAMS as i32,
                temp,
                &str1,
                self.inv_back,
            )
            .await;

        leave = false;

        temp = 1;

        loop {
            who = self.luett[temp as usize] as i32;
            if num == 0 {
                who = self.mcluett[temp as usize] as i32;
            }

            extra = b"X".to_vec();
            if temp == 15 {
                extra = b"L".to_vec();
            }
            if num == 0 {
                if self.mcpisteet[self.mcluett[(temp + 1) as usize] as usize] == 0 {
                    extra = b"L".to_vec();
                }
                self.stats[NUM_TEAMS][self.osakilpailu as usize].wc_pos = self.sija[NUM_TEAMS]; //{ ?? }
                self.stats[NUM_TEAMS - 1][self.osakilpailu as usize].wc_pos = self.sija[NUM_TEAMS];
            }

            if who > 15 - self.jmaara as i32 {
                self.sija[0] = self.sija[who as usize];
                if num == 0 {
                    self.stats[(15 - who) as usize][self.osakilpailu as usize].wc_pos =
                        self.sija[who as usize];
                } else {
                    self.stats[(num + (15 - who) * 4) as usize][self.osakilpailu as usize]
                        .round_len[self.kierros as usize] = self.sija[who as usize] as i32;
                    self.stats[(15 - who) as usize][self.osakilpailu as usize].comp_pos =
                        self.sija[who as usize];
                }
            }

            pts = self.pisteet[who as usize];
            if num == 0 {
                pts = self.mcpisteet[who as usize];
            }

            if ldiff && temp > 1 {
                if num == 0 {
                    pts -= self.mcpisteet[self.mcluett[1] as usize];
                } else {
                    pts -= self.pisteet[self.luett[1] as usize];
                }
            }

            temp = self
                .ls
                .entry(
                    temp,
                    self.sija[who as usize] as i32,
                    who,
                    &self.i.jnimet.borrow()[who as usize],
                    pts,
                    0,
                    0,
                    &extra,
                )
                .await;

            if temp < 0 {
                leave = true;
            }

            if leave {
                break;
            }
        }

        if temp == -2 {
            self.cupslut = true;
        }

        if num == 0 {
            //{$IFDEF REG}
            if self.s.get_ch() == b's' || self.s.get_ch() == b'S' || self.lct {
                if !self.cupslut {
                    self.showstats().await;
                }
            }
            //{$ENDIF}
            if self.osakilpailu == 6 {
                self.sorthighs(1).await;
            }
        }
    }

    fn jarjestys(&mut self, fromarray: u8, toarray: u8, num: u8) {
        // { from: 0 - MCpist, 1 - fourpts, 2 - pisteet }
        // { to: 0 - mcluett, 1 - luett }
        let mut score1: i32;
        let mut score2: i32;
        let mut templuett: [u8; NUM_PL + 2] = [0; NUM_PL + 2];

        for t1 in 1..=NUM_PL + 1 {
            //{ kaikki nolliin }
            templuett[t1] = 0;
            self.sija[t1] = 0;
        }
        for t1 in 1..=num {
            //{ Jokainen hypp��j� k�yd��n l�pi }
            let mut t2 = 1;

            loop {
                match fromarray {
                    0 => {
                        score1 = self.mcpisteet[t1 as usize];
                        score2 = self.mcpisteet[templuett[t2 as usize] as usize];
                    }
                    1 => {
                        score1 = self.fourpts[t1 as usize];
                        score2 = self.fourpts[templuett[t2 as usize] as usize];
                    }
                    2 => {
                        score1 = self.pisteet[t1 as usize];
                        score2 = self.pisteet[templuett[t2 as usize] as usize];
                    }
                    _ => panic!("Invalid fromarray"),
                }
                if score1 > score2 {
                    for t3 in (t2..=num as i32).rev() {
                        templuett[t3 as usize] = templuett[t3 as usize - 1];
                    }

                    templuett[t2 as usize] = t1;
                    t2 = 100;
                } else if t2 == num as i32 {
                    templuett[t1 as usize] = t1;
                }

                t2 += 1;

                if t2 > num as i32 {
                    break;
                }
            }
        }

        for t1 in 1..=num {
            //{ k��nteistaulukko eli sija[pelaaja]? }
            self.sija[templuett[t1 as usize] as usize] = t1;
        }

        for t1 in 2..=num {
            //{ tasapiste-sijoittelija }
            match fromarray {
                0 => {
                    score1 = self.mcpisteet[templuett[t1 as usize] as usize];
                    score2 = self.mcpisteet[templuett[t1 as usize - 1] as usize];
                }
                1 => {
                    score1 = self.fourpts[templuett[t1 as usize] as usize];
                    score2 = self.fourpts[templuett[t1 as usize - 1] as usize];
                }
                2 => {
                    score1 = self.pisteet[templuett[t1 as usize] as usize];
                    score2 = self.pisteet[templuett[t1 as usize - 1] as usize];
                }
                _ => panic!("Invalid fromarray"),
            }

            if score1 == score2 {
                self.sija[templuett[t1 as usize] as usize] =
                    self.sija[templuett[t1 as usize - 1] as usize];
            }
        }

        match toarray {
            0 => {
                for t1 in 1..=num {
                    self.mcluett[t1 as usize] = templuett[t1 as usize];
                }
            }
            1 => {
                for t1 in 1..=num {
                    self.luett[t1 as usize] = templuett[t1 as usize];
                }
            }
            _ => panic!("Invalid toarray"),
        }
    }

    fn kothjarj(&mut self) {
        let mut t2: i32;
        let mut score1: i32;
        let mut score2: i32;

        let players = self.kothmaara as i32 + self.i.pmaara.get() as i32;

        for t1 in 1..=players {
            //{ Jokainen hyppääjä käydään läpi }
            t2 = 1;
            loop {
                //{ Verrataan jokaiseen }
                score1 = self.pisteet[self.mcluett[t1 as usize] as usize];
                score2 = self.pisteet[self.luett[t2 as usize] as usize];

                if self.mcpisteet[self.mcluett[t1 as usize] as usize] > 0 {
                    //{ pelaaja on jo laulukuorossa }
                    score1 = -self.mcpisteet[self.mcluett[t1 as usize] as usize] * 100;
                }

                if self.mcpisteet[self.luett[t2 as usize] as usize] > 0 {
                    //{ pelaaja on jo laulukuorossa }
                    score2 = -self.mcpisteet[self.luett[t2 as usize] as usize] * 100;
                }

                if score1 > score2 {
                    for t3 in (t2..=players).rev() {
                        self.luett[t3 as usize] = self.luett[t3 as usize - 1];
                    }

                    self.luett[t2 as usize] = self.mcluett[t1 as usize];
                    t2 = 100;
                } else if t2 == players {
                    self.luett[t1 as usize] = self.mcluett[t1 as usize];
                }

                t2 += 1;

                if t2 > players {
                    break;
                }
            }
        }
    }

    async fn jcupluettelo(&mut self) {
        for loop1 in 1..=NUM_TEAMS {
            if self.sija[loop1] < 9 {
                self.mcpisteet[loop1] += TEAM_POINTS[self.sija[loop1] as usize];
            }
        }

        self.jarjestys(0, 0, NUM_TEAMS as u8);

        self.jlista(0).await;
    }

    async fn kothlista(&mut self) {
        let mut temp: i32;
        let mut temp2: i32;
        let mut who: i32;
        let mut leave: bool = false;
        let mut extra: Vec<u8>;
        let mut str1: Vec<u8>;

        // { fillline(14,'"'); }

        self.kothjarj();

        let players = self.kothmaara as i32 + self.i.pmaara.get() as i32;

        for temp in 1..=players - 1 {
            //{ tasapisteissä arvotaan }
            if self.pisteet[self.luett[temp as usize] as usize]
                == self.pisteet[self.luett[temp as usize + 1] as usize]
                && random(2) == 0
                && self.mcpisteet[self.luett[temp as usize] as usize] == 0
                && self.mcpisteet[self.luett[temp as usize + 1] as usize] == 0
            {
                // { beep(2); }
                self.luett.swap(temp as usize, temp as usize + 1);
            }
        }

        temp = players + 1 - self.mcpisteet[0];

        self.mcpisteet[self.luett[temp as usize] as usize] = temp;

        for temp in 1..=NUM_PL + 1 {
            self.luett[temp] = 0;
        }

        self.kothjarj();

        let remaining = players - self.mcpisteet[0];
        let names: Vec<(String, String)> = (1..=remaining)
            .map(|t| (format!("jumper{t}"), format!("score{t}")))
            .collect();
        let mut fields: Vec<(&str, trace::Value)> = vec![
            ("round", self.mcpisteet[0].into()),
            (
                "out",
                (self.luett[(players + 1 - self.mcpisteet[0]) as usize] as i32).into(),
            ),
        ];
        for (t, (jumper, score)) in names.iter().enumerate() {
            let pel = self.luett[t + 1] as usize;
            fields.push((jumper, (pel as i32).into()));
            fields.push((score, self.pisteet[pel].into()));
        }
        trace::record("KothRound", &fields);

        // {  for a:=1 to 50 do pisteet[a]:=1000; }

        str1 = [b" " as &[u8], &self.l.lstr(95)].concat();

        if players - self.mcpisteet[0] == 1 {
            str1 = b"!".to_vec();
        }

        self.ls
            .reset_list(
                self.i.pmaara.get() as i32,
                NUM_PL as i32,
                4,
                &[&self.l.lstr(31) as &[u8], &str1].concat() as &[u8],
                self.inv_back,
            )
            .await;

        temp = 1;

        loop {
            // {    DrawScreen;
            //      ch:=readkey; }

            temp2 = 0;
            extra = b"X".to_vec();

            if temp == players + 1 - self.mcpisteet[0] {
                str1 = self.l.lrstr(101, 105).to_vec();

                temp2 = self.ls.entry(0, 0, 3, &str1, 0, 0, 0, b"X").await;

                // {       drawscreen; readkey; }
            }

            if temp2 >= 0 {
                if temp >= players {
                    extra = b"L".to_vec(); //{ vikaa viedään }
                }

                if temp == 1 && players - self.mcpisteet[0] == 1 {
                    extra.push(b'K'); //{ is KING OF THE HILL }
                }

                who = self.luett[temp as usize] as i32;

                let name = self.i.nimet.borrow()[who as usize].clone();
                temp = self
                    .ls
                    .entry(
                        temp,
                        temp,
                        who,
                        &name,
                        self.pisteet[who as usize],
                        self.cstats[1][who as usize],
                        self.cstats[2][who as usize],
                        &extra,
                    )
                    .await;

                leave = false;
            }

            if temp < 0 || temp2 < 0 {
                leave = true;
            }

            if leave {
                break;
            }
        }

        // {     DrawScreen; }

        if temp == -2 {
            self.cupslut = true;
        }

        if players - self.mcpisteet[0] == 1
            && self.luett[1] as i32 > NUM_PL as i32 - self.i.pmaara.get() as i32
            && self.kothpack > 0
            && !self.cupslut
        {
            //{ completion! }
            {
                let mut top = self.i.top.borrow_mut();
                // Esc in the challenge menu sets kothpack to 255. Pascal then
                // writes past the end of the top list; this skips the write.
                if let Some(entry) = top.get_mut(35 + self.kothpack as usize) {
                    entry.score += 1;
                    entry.name = self.i.nimet.borrow()[self.luett[1] as usize].clone();
                    entry.time = dayandtime(self.today, self.now);
                    trace::record(
                        "TopUpdate",
                        &[
                            ("index", (35 + self.kothpack as i32).into()),
                            ("name", (&entry.name as &[u8]).into()),
                            ("score", entry.score.into()),
                            ("pos", entry.pos.into()),
                            ("time", (&entry.time as &[u8]).into()),
                        ],
                    );
                }
            }

            {
                let actprofile =
                    self.i.profileorder.borrow()[NUM_PL + 1 - self.luett[1] as usize] as usize;
                let mut profile = self.i.profile.borrow_mut();
                temp2 = profile[actprofile].kothlevel as i32;

                if temp2 == 0 || (self.kothpack as i32) < temp2 {
                    profile[actprofile].kothlevel = self.kothpack; //{ profilen kothlevel }
                }
            }

            self.g.new_screen(1, 2).await;

            self.dokothrecords(self.kothpack).await;

            self.u.wait_for_key3(305, 6).await;
        }
    }

    //{ 0-esittely, 1-tuloksien kanssa }
    async fn showpairs(&mut self, phase: u8) {
        let draw_stuff = |column: u8, row: u8, item: u8, who: i32| {
            let xx = 145 + (column as i32 * 30);
            let yy = 17 + (row as i32 * 7);
            let mut plus = self
                .g
                .font_len(&self.g.nsh(&self.i.nimet.borrow()[who as usize], 100))
                + 5;
            if plus > 105 {
                plus = 105;
            }

            let extra: &[u8] = if self.qual[who as usize] == 1 {
                b"Q"
            } else if self.qual[who as usize] == 2 {
                b"LL"
            } else {
                b""
            };

            match item {
                0 => self.g.e_write_font(
                    xx,
                    yy,
                    &self.g.nsh(&self.i.nimet.borrow()[who as usize], 100),
                ),
                1 => self.g.write_font(
                    xx,
                    yy,
                    &self.g.nsh(&self.i.nimet.borrow()[who as usize], 100),
                ),
                2 => self.g.e_write_font(
                    xx - plus,
                    yy,
                    &[b"(" as &[u8], &txt(self.qual[who as usize] as i32), b")"].concat(),
                ),
                3 => self.g.write_font(
                    xx + plus,
                    yy,
                    &[b"(" as &[u8], &txt(self.qual[who as usize] as i32), b")"].concat(),
                ),

                4 => self
                    .g
                    .e_write_font(40, yy, &txtp(self.pisteet[who as usize])),
                5 => self
                    .g
                    .e_write_font(303, yy, &txtp(self.pisteet[who as usize])),

                6 => self.g.e_write_font(12, yy, extra),
                7 => self.g.write_font(308, yy, extra),

                10 => self.g.write_font(154, yy, b"vs."),

                _ => {}
            }
        };
        self.g.new_screen(1, 0).await;
        self.g.font_color(240);
        self.g.write_font(30, 6, self.l.lstr(94));
        self.g.font_color(241);

        for temp in (1..=25).rev() {
            for temp2 in 0..=1 {
                let who = if temp2 == 1 {
                    self.luett[temp] as i32
                } else {
                    self.luett[51 - temp] as i32
                };

                if self.kierros == 1 && who > NUM_PL as i32 - self.i.pmaara.get() as i32 {
                    self.stats[NUM_PL + 1 - who as usize][self.osakilpailu as usize].round1_pos =
                        self.sija[who as usize];
                }

                self.g.font_color(241);
                draw_stuff(temp2, (26 - temp) as u8, 10, who); //{ vs. }

                if phase == 1 {
                    match self.qual[who as usize] {
                        1 => self.g.font_color(251),
                        2 => self.g.font_color(252),
                        _ => {}
                    }
                }

                if who > NUM_PL as i32 - self.i.pmaara.get() as i32 {
                    self.g.font_color(240);
                }

                draw_stuff(temp2, (26 - temp) as u8, temp2, who); //{ nimi ruutuun }
                if phase == 1 {
                    draw_stuff(temp2, (26 - temp) as u8, temp2 + 4, who);
                    draw_stuff(temp2, (26 - temp) as u8, temp2 + 6, who);
                } else {
                    draw_stuff(temp2, (26 - temp) as u8, temp2 + 2, who); //{ qual }
                }
            }
        }
        self.cupslut = self.u.wait_for_key3(305, 6).await;
    }

    async fn jumpalku(&mut self) {
        self.u.load_info(self.nytmaki, &mut self.act_hill).await;

        for temp in 0..=NUM_PL {
            self.pisteet[temp] = 0;
            self.luett[temp] = 0;
            self.mcluett[temp] = 0;
            self.cstats[0][temp] = 0;
            self.cstats[1][temp] = 0;
            self.cstats[2][temp] = 0;
        }
        self.this_is_a_hill_record = 0;
    }

    //{ 0 - train, 1 - koth, 2 - morehills, 3 - morehills KOTH }
    //{ uusi: 0 - train, 1 - koth }
    async fn selecthill(&mut self, phase: u8) -> u8 {
        let mut temp: i32;
        let mut gothill: i32;
        let mut hill: i32;
        let mut str1: Rc<Vec<u8>>;
        let mut items: i32;
        let mut leave: bool;
        let mut start: i32;

        start = 0;

        loop {
            self.g.new_screen(2, 0).await;

            self.g.font_color(240);

            self.g.write_font(30, 31, &*self.l.lstr(151));
            self.g.write_font(30, 41, &*self.l.lstr(152));
            self.g.write_font(30, 51, &*self.l.lstr(153));

            items = self.u.num_extra_hills.get() as i32 + NUM_WC_HILLS - start;
            if items > 20 {
                items = 20;
            }

            temp = 0;
            for t in 1..=items {
                temp = t;
                hill = temp + start;

                self.g.font_color(246);
                self.g.e_write_font(
                    130,
                    (temp - 1) * 8 + 10,
                    &[&txt(temp) as &[u8], b"."].concat(),
                );

                self.g.font_color(247);
                self.g.write_font(
                    145 + self.g.font_len(&self.u.hillname(hill)),
                    (temp - 1) * 8 + 10,
                    &[b"K" as &[u8], &txt(self.u.hillkr(hill))].concat(),
                );
                self.g.font_color(240);
                self.g
                    .write_font(140, (temp - 1) * 8 + 10, &self.u.hillname(hill));
            }

            if self.u.num_extra_hills.get() > 0 {
                //{ * more hills * }
                temp += 1;
                items += 1;
                self.g.font_color(247);
                self.g
                    .write_font(140, (temp - 1) * 8 + 10, &*self.l.lstr(156));
                self.g.font_color(240);
            }

            temp += 2;
            self.g.e_write_font(130, (temp - 1) * 8 + 10, b"0.");
            str1 = self.l.lstr(154);
            if phase == 1 {
                str1 = self.l.lstr(155);
            }
            self.g.write_font(140, (temp - 1) * 8 + 10, &*str1);

            self.g.draw_screen().await;

            gothill = self.u.make_menu(110, 11, 170, 8, items, 1, 243, 0, 0).await;

            leave = true;

            if gothill == items && self.u.num_extra_hills.get() > 0 {
                start += 20;
                if start >= self.u.num_extra_hills.get() as i32 + NUM_WC_HILLS {
                    start = 0;
                }
                leave = false;
            }

            if leave {
                break;
            }
        }

        temp = gothill + start;

        if gothill <= 0 {
            temp = gothill;
        }

        temp as u8
    }

    async fn training(&mut self) {
        self.osakilpailu = 0;
        self.kierros = -10;
        self.treeni = true;
        self.cupslut = false;
        self.s.set_ch(1);
        self.startgate = 15;

        loop {
            self.eka = true;

            self.nytmaki = self.selecthill(0).await as i32;

            if self.nytmaki == 0 {
                self.cupslut = true;
            }

            if self.nytmaki > 0 {
                self.jumpalku().await;
            }

            // { kr:=kri[nytmaki]; }
            if self.s.get_ch() != 27 && self.nytmaki > 0 {
                loop {
                    self.tuuli.alusta(self.windplace);

                    self.hyppy(1, NUM_PL as i32 + 1, 0).await;

                    self.eka = false;
                    if self.s.get_ch() == 27 || self.cupslut {
                        break;
                    }
                }
            }

            self.s.set_ch(1);

            if self.cupslut {
                break;
            }
        }

        self.treeni = false;
        self.startgate = 0; //{ varmuuden vuoksi }
    }

    fn getkoth(&mut self) {
        match self.kothpack {
            1 => {
                self.kothmaara = 20;
                for temp in 1..=self.kothmaara {
                    self.kothpel[temp as usize] = temp;
                }
                self.kothwind = false;
                self.kothrounds = 2;
                self.kothmaki = 0;
            }
            2 => {
                self.kothmaara = 14;
                for temp in 1..=self.kothmaara {
                    self.kothpel[temp as usize] = temp * 3 - 1;
                }
                self.kothwind = true;
                self.kothrounds = 1;
                self.kothmaki = 0;
            }
            3 => {
                self.kothmaara = 10;
                for temp in 1..=self.kothmaara {
                    self.kothpel[temp as usize] = temp * 4 - 1;
                }
                self.kothwind = false;
                self.kothrounds = 2;
                self.kothmaki = 0;
            }
            4 => {
                self.kothmaara = 8;
                for temp in 1..=self.kothmaara {
                    self.kothpel[temp as usize] = temp * 5 + 1;
                }
                self.kothwind = true;
                self.kothrounds = 1;
                self.kothmaki = 0;
            }
            5 => {
                self.kothmaara = 7;
                for temp in 1..=self.kothmaara {
                    self.kothpel[temp as usize] = temp * 6 + 13;
                }
                self.kothwind = false;
                self.kothrounds = 2;
                self.kothmaki = 0;
            }
            6 => {
                self.kothmaara = 6;
                for temp in 1..=self.kothmaara {
                    self.kothpel[temp as usize] = temp * 7 + 15;
                }
                self.kothwind = true;
                self.kothrounds = 1;
                self.kothmaki = 0;
            }
            _ => {}
        }
    }

    async fn startkoth(&mut self) {
        let mut realindex: i32;

        self.koth = true;
        self.cupslut = false;
        self.startgate = 0;

        if self.kothmaki == 0 {
            self.nytmaki = random(NUM_WC_HILLS as u32) as i32 + 1;
        } else {
            self.nytmaki = self.kothmaki;
        }

        if self.nytmaki > 0 {
            self.jumpalku().await;
        }

        if self.kothwind {
            self.tuuli.alusta(self.windplace);
        } else {
            self.tuuli.voim.set(0);
        }

        // {  if (not cupslut) and (preview1) then preview(nytmaki); }

        let kothmaara = self.kothmaara as i32;
        let pmaara = self.i.pmaara.get() as i32;

        for temp in 1..=kothmaara {
            self.mcluett[temp as usize] = self.kothpel[temp as usize];
        }

        for temp in 1..=pmaara {
            self.mcluett[(kothmaara + temp) as usize] = (NUM_PL as i32 + 1 - temp) as u8;
        }

        for temp in 1..=NUM_PL {
            self.mcpisteet[temp] = 0;
        }

        self.eka = true;

        if !self.cupslut {
            for kisa in 1..=kothmaara + pmaara - 1 {
                //{ kisaluuppi }
                self.mcpisteet[0] = kisa;

                for temp in 1..=kothmaara + pmaara {
                    if self.mcpisteet[self.mcluett[temp as usize] as usize] == 0 {
                        self.pisteet[self.mcluett[temp as usize] as usize] = 0;
                    }
                }

                for temp in 1..=NUM_PL {
                    self.luett[temp] = 0;
                }

                // {  eka:=true; } { kaivanvertanenästä }
                self.kierros = 1;
                realindex = 0; //{ monta äijää oikeasti hypännyt }

                for index in 1..=kothmaara + pmaara {
                    if self.mcpisteet[self.mcluett[index as usize] as usize] == 0 && !self.cupslut {
                        realindex += 1;
                        self.hyppy(index, self.mcluett[index as usize] as i32, realindex)
                            .await;
                        self.eka = false;
                    }
                }

                // {    eka:=true;  } { TSEKKAA TÄMÄ!!! }

                realindex = 0;

                if self.kothrounds == 2 {
                    self.kierros = 2;

                    for index in 1..=kothmaara + pmaara {
                        if self.mcpisteet[self.mcluett[index as usize] as usize] == 0
                            && !self.cupslut
                        {
                            realindex += 1;
                            self.hyppy(index, self.mcluett[index as usize] as i32, realindex)
                                .await;
                            self.eka = false;
                        }
                    }
                }

                if !self.cupslut {
                    self.kothlista().await;
                }
            }
        }

        self.koth = false;
    }

    async fn choosekoth(&mut self) {
        let mut y: i32;

        self.kothmaara = 0;

        self.g.fill_box(170, 0, 319, 199, 243);
        self.g.fill_area(170, 0, 319, 199, 63);

        self.g.font_color(240);
        self.g.write_font(180, 2, &*self.l.lstr(138));

        self.g.font_color(241);
        self.g.write_font(180, 9, &*self.l.lstr(139));
        self.g.write_font(180, 16, &*self.l.lstr(140));

        self.g.font_color(247);
        self.g.write_font(180, 24, &*self.l.lstr(141));
        self.g.write_font(295, 24, &*self.l.lstr(142));

        let maxpel = NUM_PL as i32 - self.i.pmaara.get() as i32;

        loop {
            self.kothmaara += 1;
            let k = self.kothmaara as usize;

            loop {
                y = self.kothmaara as i32 * 8 + 25;

                self.g
                    .fill_box(178, (y - 2) as u16, 315, (y + 8) as u16, 243);
                self.g.font_color(240);
                self.g.write_font(
                    180,
                    y,
                    &self
                        .g
                        .nsh(&self.i.nimet.borrow()[self.kothpel[k] as usize], 110),
                );
                self.g.e_write_font(
                    310,
                    y,
                    &[b"#" as &[u8], &txt(self.kothpel[k] as i32)].concat(),
                );

                self.g.draw_screen().await;

                let (ch, ch2) = self.s.wait_for_key_press().await;
                self.s.set_ch(ch);
                self.s.set_ch2(ch2);

                if self.s.get_ch() == 13 {
                    //{ tsekataan ettei sitä pelaajaa ole jo }
                    for temp in 1..k {
                        if self.kothpel[temp] == self.kothpel[k] {
                            if (self.kothpel[k] as i32) < maxpel {
                                self.kothpel[k] += 1;
                            }
                            if self.beeppi {
                                beep(self.s, 2);
                            }
                            self.s.set_ch(1);
                        }
                    }
                }

                if self.s.get_ch() == 0 {
                    let ch2 = self.s.get_ch2();
                    if ch2 == 71 {
                        self.kothpel[k] = 1;
                    }
                    if ch2 == 79 {
                        self.kothpel[k] = maxpel as u8;
                    }
                    if (ch2 == 72 || ch2 == 75) && self.kothpel[k] > 1 {
                        self.kothpel[k] -= 1;
                    }
                    if (ch2 == 80 || ch2 == 77) && (self.kothpel[k] as i32) < maxpel {
                        self.kothpel[k] += 1;
                    }
                    if ch2 == 73 && self.kothpel[k] > 10 {
                        self.kothpel[k] -= 10;
                    }
                    if ch2 == 81 && (self.kothpel[k] as i32) < maxpel - 9 {
                        self.kothpel[k] += 10;
                    }
                }

                if self.s.get_ch() == 9 {
                    self.s.set_ch(27);
                }

                if self.s.get_ch() == 13 || self.s.get_ch() == 27 {
                    break;
                }
            }

            if self.s.get_ch() == 27 {
                self.kothmaara -= 1;
            }
            if self.s.get_ch() == 13 {
                self.g.font_color(246);
                self.g.write_font(
                    180,
                    y,
                    &self
                        .g
                        .nsh(&self.i.nimet.borrow()[self.kothpel[k] as usize], 110),
                );
                self.g.e_write_font(
                    310,
                    y,
                    &[b"#" as &[u8], &txt(self.kothpel[k] as i32)].concat(),
                );
            }

            if self.s.get_ch() == 27 || self.kothmaara == 20 {
                break;
            }
        }
    }

    async fn newkingofthehill(&mut self) {
        let mut x: i32;
        let mut y: i32;
        let mut col1: u8;
        let mut col2: u8;
        let mut str1: Rc<Vec<u8>>;
        let mut index: i32;

        self.getkoth();

        loop {
            self.g.new_screen(3, 0).await;

            self.g.font_color(246);

            if self.kothmaara > 0 {
                for temp in 1..=self.kothmaara as usize {
                    let pel = self.kothpel[temp];
                    self.g.write_font(
                        180,
                        20 + temp as i32 * 8,
                        &[
                            &self.g.nsh(&self.i.nimet.borrow()[pel as usize], 110) as &[u8],
                            b" #",
                            &txt(pel as i32),
                        ]
                        .concat(),
                    );
                }
            } else {
                self.g.write_font(180, 30, &*self.l.lstr(9));
            }

            x = 10;
            y = 10;

            self.g.font_color(240);

            self.g.write_font(180, 10, &*self.l.lstr(120));

            col1 = 240;
            col2 = 246;
            if self.kothpack > 0 {
                col1 = 241;
                col2 = 241;
            }

            self.g
                .write_font(x, y, &[b"1 - " as &[u8], &self.l.lstr(121)].concat());
            y += 10;

            self.g
                .write_font(x, y, &[b"2 - " as &[u8], &self.l.lstr(122)].concat());
            y += 10;

            self.g.font_color(col1);
            self.g
                .write_font(x, y, &[b"3 - " as &[u8], &self.l.lstr(123)].concat());
            y += 10;

            self.g
                .write_font(x, y, &[b"4 - " as &[u8], &self.l.lstr(124)].concat());
            self.g.font_color(col2);
            self.g
                .write_font(x + 70, y, &self.g.nsh(&self.u.hillname(self.kothmaki), 80));
            y += 10;
            self.g.font_color(col1);

            self.g
                .write_font(x, y, &[b"5 - " as &[u8], &self.l.lstr(125)].concat());
            str1 = self.l.lstr(7);
            if self.kothwind {
                str1 = self.l.lstr(6);
            }
            self.g.font_color(col2);
            self.g.write_font(x + 70, y, &*str1);
            y += 10;

            self.g.font_color(col1);
            self.g
                .write_font(x, y, &[b"6 - " as &[u8], &self.l.lstr(126)].concat());
            self.g.font_color(col2);
            self.g
                .write_font(x + 70, y, &*self.l.lstr(self.kothrounds as u32));
            y += 20;

            self.g.font_color(240);
            self.g
                .write_font(x, y, &[b"0 - " as &[u8], &self.l.lstr(127)].concat());

            self.u.kothchallenge(x, self.kothpack);

            self.g.draw_screen().await;

            index = self.u.make_menu(10, 10, 160, 10, 6, 1, 245, 0, 3).await;

            match index {
                1 => self.startkoth().await,
                2 => {
                    //{ select challenge }
                    self.u.kothchallenge(x, 255); //{ kaikki esiin }
                    self.g.draw_screen().await;

                    self.kothpack =
                        self.u.make_menu(10, 121, 160, 8, 6, 1, 244, 0, 254).await as u8;
                    // {$IFNDEF REG}
                    //  if ((kothpack<1) or (kothpack>6)) then kothpack:=temp;
                    // {$ENDIF}

                    self.getkoth();
                }
                //{$IFDEF REG}
                3 => {
                    self.kothpack = 0;
                    self.choosekoth().await;
                } // { else if (beeppi) then beep(2); }
                4 => {
                    self.kothpack = 0;
                    self.kothmaki = self.selecthill(1).await as i32;
                } // { else if (beeppi) then beep(2); }
                5 => {
                    self.kothpack = 0;
                    self.kothwind = !self.kothwind;
                } // { else if (beeppi) then beep(2); }
                6 => {
                    self.kothpack = 0;
                    match self.kothrounds {
                        1 => self.kothrounds = 2,
                        2 => self.kothrounds = 1,
                        _ => {}
                    }
                } // { else if (beeppi) then beep(2);}
                //{$ENDIF}
                _ => {}
            }

            if index == 0 {
                break;
            }
        }

        // {  if (index=1) then startkoth; }

        self.write_config(); //{ kirjoitetaan muutokset levylle }
    }

    //{ 0 - tekee ekan, 1 - tokan... }
    async fn getteam(&mut self, num: u8) {
        let mut xx: i32;

        xx = 30;

        if num == 1 {
            xx = 160;
        }

        // { FontColor(240); }

        // { Writefont(xx,38,'TEAM'+txt(num+1)); }
        // { Writefont(xx,0,jnimet[15-num]); }

        self.g.font_color(246);

        for temp in 1..=4 {
            self.g.write_font(
                xx + 13,
                (temp - 1) * 10 + 56,
                &self.i.nimet.borrow()[NUM_PL + 1 - (num as usize * 4) - temp as usize],
            );
        }

        self.g.font_color(240);

        self.g.write_font(
            xx,
            30,
            &[&self.l.lstr(113) as &[u8], b" ", &txt(num as i32 + 1), b":"].concat(),
        );
        // {  FillBox(40,65,200,81,243); }

        // { Fontcolor(240); }

        self.g.draw_screen().await;

        let oldstr = self.i.jnimet.borrow()[15 - num as usize].clone();
        let newstr = self.u.getstr(xx, 42, 120, &oldstr, 242).await;
        self.i.jnimet.borrow_mut()[15 - num as usize] = newstr;

        self.g
            .fill_box((xx - 10) as u16, 30, (xx + 124) as u16, 54, 243);
        self.g
            .fill_area((xx - 10) as u16, 30, (xx + 124) as u16, 54, 63);

        self.g.font_color(241);

        self.g.write_font(
            xx,
            30,
            &[&self.l.lstr(114) as &[u8], b" ", &txt(num as i32 + 1), b":"].concat(),
        );

        self.g.font_color(240);
        self.g
            .write_font(xx, 42, &self.i.jnimet.borrow()[15 - num as usize]);
    }

    async fn teamsready(&mut self) {
        let mut temp: i32;

        self.g.new_screen(1, 1).await;

        self.g.font_color(240);
        self.g.write_font(30, 6, self.l.lstr(111));

        // { FontColor(247); }

        self.g.write_font(30, 110, self.l.lstr(112));

        for temp in 1..=40 {
            self.hill_order[temp] = temp as i32;
        }

        self.hill_order[1] = random(NUM_WC_HILLS as u32) as i32 + 1;

        temp = 2;
        while temp <= 6 {
            self.hill_order[temp as usize] = random(NUM_WC_HILLS as u32) as i32 + 1;

            for temp2 in 1..=temp - 1 {
                if self.hill_order[temp as usize] == self.hill_order[temp2 as usize] {
                    temp -= 1;
                }
            }

            temp += 1;
        }

        self.g.font_color(246);

        for temp in 1..=6 {
            self.g.write_font(
                30,
                (temp * 10) + 114,
                &[
                    &txt(temp) as &[u8],
                    b". ",
                    &self.u.hillname(self.hill_order[temp as usize]),
                ]
                .concat(),
            );
        }

        // { Writefont(30,36,'THE TEAM LINEUPS ARE:'); }

        self.getteam(0).await;

        if self.jmaara > 1 {
            self.getteam(1).await;
        }

        /*{  ShowTeam(0);

        if (jmaara>1) then ShowTeam(1);

         FontColor(240); }*/

        for temp in 1..=4 {
            self.teamlineup[14 * 4 + temp] = (NUM_PL + 1 - temp) as u8;
        }

        if self.jmaara > 1 {
            for temp in 1..=4 {
                self.teamlineup[13 * 4 + temp] = (NUM_PL - 3 - temp) as u8;
            }
        }

        self.g.draw_screen().await;

        self.cupslut = self.u.wait_for_key3(305, 180).await;

        // { ReadKey; }
    }

    async fn showteams(&mut self, teamorder: &[i32; 16]) {
        let mut x: i32;
        let mut y: i32;
        let mut team: i32;
        let mut t3: i32;

        self.g.new_screen(1, 1).await;
        self.g.write_font(30, 6, self.l.lstr(111));
        x = 5;
        y = 24;

        for t1 in (1..=15).rev() {
            team = teamorder[t1];
            self.g.font_color(240);
            self.g.write_font(
                x,
                y,
                &self.g.nsh(&self.i.jnimet.borrow()[team as usize], 95),
            );

            if team > 15 - self.jmaara as i32 {
                self.g.font_color(246);
            } else {
                self.g.font_color(241);
            }

            for t2 in 1..=4 {
                t3 = self.teamlineup[((team - 1) * 4 + t2) as usize] as i32;
                self.g.write_font(
                    x + 4,
                    y + 1 + (t2 * 6),
                    &self.g.nsh(&self.i.nimet.borrow()[t3 as usize], 90),
                );
            }

            x += 102;
            if x > 240 {
                x = 5;
                y += 35;
            }
            self.g.draw_screen().await;
        }

        self.cupslut = self.u.wait_for_key3(305, 6).await;
    }

    async fn startteam(&mut self) {
        let mut temp: i32;
        let mut index: i32;
        let mut team: i32;
        let mut teamorder: [i32; 16] = [0; 16];

        let swap = |order: &mut [i32; 16], i1: usize, i2: usize| {
            let temp = order[i1];
            order[i1] = order[i2];
            order[i2] = temp;
        };

        self.osakilpailu = 0;
        self.startgate = 0;

        for temp in 1..=15 {
            self.mcpisteet[temp] = 0;
            teamorder[temp] = 16 - temp as i32;
        }

        for _temp2 in 1..=3 {
            for temp in 1..=14 {
                if random(2) == 0 {
                    swap(&mut teamorder, temp, temp + 1);
                }
            }
        }

        let names: Vec<String> = (1..=NUM_TEAMS)
            .map(|t| format!("order{t}"))
            .chain((1..=NUM_TEAMS * 4).map(|t| format!("lineup{t}")))
            .collect();
        let values = teamorder[1..=NUM_TEAMS]
            .iter()
            .copied()
            .chain(self.teamlineup[1..=NUM_TEAMS * 4].iter().map(|&t| t as i32));
        let fields: Vec<(&str, trace::Value)> = names
            .iter()
            .map(String::as_str)
            .zip(values.map(Into::into))
            .collect();
        trace::record("TeamLineup", &fields);

        self.reset_stats();

        self.showteams(&teamorder).await;

        loop {
            self.tuuli.alusta(self.windplace);

            self.osakilpailu += 1;

            self.nytmaki = self.hill_order[self.osakilpailu as usize];

            self.jumpalku().await;

            // {    Stats[0,osakilpailu].Reason[1]:=nytmaki; }

            // {    if (preview1) then preview(nytmaki); }

            self.kierros = 1;

            // {  team:=1; }

            self.eka = true;

            index = 1;
            loop {
                //{ jumpers 1..4 }
                for temp in 1..=15 {
                    self.luett[temp] = 0;
                }

                // {       eka:=true; }

                // { if (not cupslut) then jmuidenhypyt(index); }

                for temp2 in 1..=NUM_TEAMS {
                    team = teamorder[temp2];

                    temp = self.teamlineup[((team - 1) * 4 + index) as usize] as i32;

                    /*{         temp:=round((5-index)*2.5+(team*1.5)-3);
                    if (team > NumTeams-jmaara) then temp:=NumPl+1-(15-team)*4-index; }*/

                    if !self.cupslut {
                        self.hyppy(index, temp, team).await;
                    }

                    self.eka = false;
                }

                self.jarjestys(2, 1, NUM_TEAMS as u8);
                self.trace_team_standings(index);
                if !self.cupslut {
                    self.jlista(index).await;
                }

                if index == 4 {
                    break;
                }
                index += 1;
            }

            self.kierros += 1;

            // {    eka:=true; }  { hullu! }

            if self.s.get_ch() != b'S' && self.s.get_ch() != b's' {
                //{ 2. kierros }
                index = 1;
                loop {
                    for temp in 1..=15 {
                        self.luett[temp] = 0;
                    }

                    // {       eka:=true; }  { TARKASTA! (OK) }

                    for temp2 in 1..=NUM_TEAMS {
                        team = teamorder[temp2];
                        temp = self.teamlineup[((team - 1) * 4 + index) as usize] as i32;

                        /*{        temp:=round((5-index)*2.5+(team*1.5)-3);
                        if (team > NumTeams-jmaara) then temp:=NumPl+1-(15-team)*4-index; }*/

                        if !self.cupslut {
                            self.hyppy(index, temp, team).await;
                        }
                        self.eka = false;
                    }

                    self.jarjestys(2, 1, NUM_TEAMS as u8);
                    self.trace_team_standings(index);
                    if !self.cupslut {
                        self.jlista(index).await;
                    }

                    if index == 4 {
                        break;
                    }
                    index += 1;
                }
            }

            // {       eka:=true; }

            if self.s.get_ch() == b'S' || self.s.get_ch() == b's' {
                //{ skippaus (?) }
                for temp in 1..=15 {
                    self.luett[temp] = 0;
                }
                self.jarjestys(2, 1, NUM_TEAMS as u8);
                self.trace_team_standings(index);
                self.jlista(index).await;
            }

            if !self.cupslut {
                self.jcupluettelo().await;
            }

            if self.cupslut || self.osakilpailu >= 6 {
                break;
            }
        }

        // { end; }
    }

    fn trace_team_standings(&self, index: i32) {
        let names: Vec<(String, String)> = (1..=NUM_TEAMS)
            .map(|t| (format!("team{t}"), format!("score{t}")))
            .collect();
        let mut fields: Vec<(&str, trace::Value)> = vec![
            ("osakilpailu", self.osakilpailu.into()),
            ("kierros", self.kierros.into()),
            ("index", index.into()),
        ];
        for (t, (team, score)) in names.iter().enumerate() {
            let team_number = self.luett[t + 1] as usize;
            fields.push((team, (team_number as i32).into()));
            fields.push((score, self.pisteet[team_number].into()));
        }
        trace::record("TeamStandings", &fields);
    }

    async fn teamcup(&mut self) {
        self.jcup = true;
        /*{ textbackground(1);
        clrscr; }*/
        self.cupslut = false;
        self.startgate = 0;

        // { putsaa; }
        let pmaara = self.i.pmaara.get();
        if pmaara == 4 || pmaara == 8 {
            //{ Ehkä vähän turha }
            self.i
                .load_names(self.namenumber, self.jmaara, &mut self.teamlineup, false);

            self.teamsready().await;

            if !self.cupslut {
                self.startteam().await;
            }
        } else {
            if self.beeppi {
                beep(self.s, 1);
            }

            self.u.teamwarning().await;
        }

        self.jcup = false;

        self.i
            .load_names(self.namenumber, self.jmaara, &mut self.teamlineup, true);
    }

    //{ cupstyle: 0 - SJ3 WC, 1 - Custom WC, 2 - Just 4Hills }
    async fn cup(&mut self) {
        let mut temp2 = 0;
        let mut index = 0;
        let mut sortby = 0u8; //{ sortby: 0 - WC, 1 - t_points }
        let mut skipquali = 0u8;
        let mut fourhills = false;
        let mut dokosystem = false;
        let mut skip = false;

        self.wcup = true;

        self.osakilpailu = 0;
        self.startgate = 0;

        self.cupslut = false;

        for temp in 0..=NUM_PL + 1 {
            self.mcpisteet[temp] = 0;
            self.fourpts[temp] = 0;
            self.inj[temp] = 0;
        }

        self.reset_stats();

        match self.cup_style {
            0 => {
                for temp in 0..=40 {
                    self.hill_order[temp as usize] = temp;
                }
                self.cup_hills = NUM_WC_HILLS;
                sortby = 0;
            }
            1 => {
                // TODO
                //SelectCustomHills(sortby, CupHills, HillOrder, SetFile);
                panic!("SelectCustomHills not implemented");
            }
            2 => {
                for temp in 0..=4 {
                    self.hill_order[temp as usize] = temp + 8;
                }
                self.cup_hills = 4;
                sortby = 1;
            }
            _ => {}
        }

        if self.cup_hills == 0 {
            self.cupslut = true;
        }

        self.s.set_ch(1);

        loop {
            self.osakilpailu += 1;
            fourhills = false;
            dokosystem = false;
            if (self.cup_style == 2)
                || (self.cup_style == 0 && self.osakilpailu > 8 && self.osakilpailu < 13)
            {
                fourhills = true;
            }
            if fourhills && self.kosystem {
                dokosystem = true;
            }

            self.nytmaki = self.hill_order[self.osakilpailu as usize];

            self.tuuli.alusta(self.windplace);
            self.jumpalku().await;
            self.kierros = 0;

            for temp in 1..=NUM_PL {
                //{ uusi skaba alkaa, v�hennet��n inj }
                if self.inj[temp] > 0 {
                    self.inj[temp] -= 1;
                }
            }

            self.jarjestys(sortby, 0, NUM_PL as u8); //{ tehd��n eka j�rkk� mcluettiin }
            for temp in 0..=NUM_PL {
                self.qual[temp] = 0;
            }

            if self.osakilpailu > 1 {
                for temp in 1..=NUM_PL {
                    // { suoraan MC pisteill� sis��n }
                    if self.sija[temp] < 11 && self.inj[temp] == 0 {
                        self.qual[temp] = 2;
                        self.qual[0] += 1;
                    }
                }
            }

            self.eka = true;

            //{ TRAINING ROUNDS (kierros-1,-2,-3) }

            if !self.cupslut && self.trainrounds > 0 {
                for temp in 1..=self.trainrounds {
                    self.kierros = -(temp as i32);

                    for index in (1..=NUM_PL).rev() {
                        if self.inj[self.mcluett[index] as usize] == 0 {
                            //{ onko loukkaantunut? }
                            if !self.cupslut {
                                self.hyppy(index as i32, self.mcluett[index] as i32, 0)
                                    .await;
                                self.eka = false;
                            }
                        }
                    }
                }
            }

            if self.cup_style != 1 {
                //{ v3.13 - customissa skipataan quali }
                self.kierros = 0;

                //{ QUAL ROUND  kierros:=0; }

                if !self.cupslut {
                    //{ NEW! }
                    for index in (1..=NUM_PL).rev() {
                        skip = false;
                        if self.mcluett[index] as usize > NUM_PL - self.i.pmaara.get() as usize {
                            skipquali = self.i.profile.borrow()[self.i.profileorder.borrow()
                                [NUM_PL + 1 - self.mcluett[index] as usize]
                                as usize]
                                .skipquali;
                            if skipquali == 2 || (!fourhills && skipquali == 1) {
                                skip = true;
                            }
                        }

                        if self.inj[self.mcluett[index] as usize] == 0
                            && !self.cupslut
                            && !((self.mcluett[index] as usize
                                > NUM_PL - self.i.pmaara.get() as usize)
                                && skip
                                && (self.qual[self.mcluett[index] as usize] > 0))
                        {
                            //{ skipquali mahis }
                            self.hyppy(index as i32, self.mcluett[index] as i32, 0)
                                .await;
                            self.eka = false;
                        }
                    }
                }

                self.jarjestys(2, 1, NUM_PL as u8);

                if !self.cupslut && self.osakilpailu > 1 {
                    //{ ei eka kisa }
                    temp2 = 1;

                    for _ in 1..=50 - self.qual[0] {
                        //{ yleens� 30 }
                        while (self.qual[self.luett[temp2] as usize] > 0
                            || self.inj[self.luett[temp2] as usize] != 0)
                            && temp2 < NUM_PL
                        {
                            temp2 += 1;
                        }
                        self.qual[self.luett[temp2] as usize] = 1;
                    }

                    if !dokosystem {
                        //{ ko systeemiss� tarvitaan tasan 50. }
                        for temp in 51 - self.qual[0] as usize..=NUM_PL {
                            if self.sija[self.luett[temp] as usize]
                                == self.sija[self.luett[temp2] as usize]
                                && self.inj[self.luett[temp] as usize] == 0
                            {
                                self.qual[self.luett[temp] as usize] = 1;
                            }
                        }
                    }
                } else {
                    //{ * eka kerta * }
                    if dokosystem {
                        for temp in 1..=50 {
                            self.qual[self.luett[temp] as usize] = 3; //{ vain tasan 50 tarvitaan KO systeemiin }
                        }
                    } else {
                        for temp in 1..=NUM_PL {
                            //{ 50 parasta qualifieria, ei v�li� vaikka loukkaant. }
                            if self.sija[temp] < 51 {
                                self.qual[temp] = 1;
                            }
                        }
                    }
                }

                temp2 = 1;

                if dokosystem {
                    for temp in 1..=50 {
                        //{ pit�isi tehd� parit }
                        while self.qual[self.luett[temp2] as usize] == 0 && temp2 < NUM_PL {
                            temp2 += 1;
                        }
                        self.qual[self.luett[temp2] as usize] = temp; //{ qualista tulikin l�ht�j�rjestysnumero }
                        temp2 += 1;
                    }
                }

                if !self.cupslut {
                    //{ karsinnan tulokset }
                    self.jarjestys(2, 1, NUM_PL as u8);
                    self.lista(0).await;
                }

                if !self.cupslut && dokosystem {
                    //{ tehd��n l�ht�j�rjestyslista }
                    for temp in 1..=NUM_PL {
                        self.luett[self.qual[temp] as usize] = temp as u8;
                    }
                    self.showpairs(0).await;
                }
            } else {
                for temp in 1..=NUM_PL {
                    self.qual[temp] = 1;
                }
            }

            for temp in 0..=NUM_PL {
                self.pisteet[temp] = 0;
            }

            for temp in 1..=NUM_PL {
                if self.qual[temp] == 0 {
                    self.pisteet[temp] = -5555; //{ pistet��n niille niin huonot pisteet ettei ne kummittele }
                }
            }

            self.kierros = 1;

            //{ *** 1. KIERROS *** }
            if dokosystem {
                for temp in (1..=25).rev() {
                    for temp2 in 0..=1 {
                        index = 51 - temp;
                        if temp2 == 1 {
                            index = temp;
                        }

                        if self.inj[self.luett[index] as usize] == 0 && !self.cupslut {
                            self.hyppy(index as i32, self.luett[index] as i32, 0).await;
                            self.eka = false;
                        }
                    }
                }
            } else {
                for index in (1..=NUM_PL).rev() {
                    if self.qual[self.mcluett[index] as usize] > 0
                        && self.inj[self.mcluett[index] as usize] == 0
                        && !self.cupslut
                    {
                        self.hyppy(index as i32, self.mcluett[index] as i32, 0)
                            .await;
                        self.eka = false;
                    }
                }
            }

            if !dokosystem && !self.cupslut {
                //{ 1. kierroksen tulokset }
                self.jarjestys(2, 1, NUM_PL as u8);
                self.updatestats(1);
                self.lista(1).await;
            }

            for temp in 0..=NUM_PL {
                self.qual[temp] = 0;
            }

            if !dokosystem {
                //{ tavallinen }
                for temp in 1..=NUM_PL {
                    //{ n�m� ovat luettelon j�lkeen, ett� siell� tiedet��n koska hypp��j�t loppuu }
                    if self.sija[temp] < 31 {
                        self.qual[temp] = 1;
                    }
                }
            } else {
                //{ ko system }
                for temp in (1..=25).rev() {
                    if self.pisteet[self.luett[temp] as usize]
                        >= self.pisteet[self.luett[51 - temp] as usize]
                    {
                        self.qual[self.luett[temp] as usize] = 1;
                    } else {
                        self.qual[self.luett[51 - temp] as usize] = 1;
                    }
                }
                for temp in 1..=NUM_PL {
                    self.mcluett[temp] = self.luett[temp]; //{ laitetaan talteen l�ht�j�rjestys }
                }

                self.jarjestys(2, 1, NUM_PL as u8); //{ tehd��n pisteist� j�rkk� }
                temp2 = 1;
                for _ in 1..=5 {
                    //{ 5 lucky loseria }
                    while self.qual[self.luett[temp2] as usize] != 0 && temp2 < NUM_PL {
                        temp2 += 1;
                    }
                    self.qual[self.luett[temp2] as usize] = 2;
                }

                for temp in 1..=NUM_PL {
                    self.luett[temp] = self.mcluett[temp]; //{ palautetaan l�ht�j�rjestys }
                }

                if !self.cupslut {
                    self.showpairs(1).await;
                }

                self.jarjestys(0, 0, NUM_PL as u8); //{ ett� mcliivi on oikealla miehell� }
                self.jarjestys(2, 1, NUM_PL as u8); //{ taas pisteist� j�rkk� n�et toinen kierros normaali on }
            }

            self.kierros = 2;

            //{ *** 2. KIERROS *** }
            if !self.cupslut {
                for index in (1..=NUM_PL).rev() {
                    if self.inj[self.luett[index] as usize] == 0
                        && self.qual[self.luett[index] as usize] > 0
                        && !self.cupslut
                    {
                        self.hyppy(index as i32, self.luett[index] as i32, 0).await;
                        self.eka = false;
                    }
                }
            }

            self.s.set_ch(1); //{ ett� hypyn keskeytt�minen sotke liikaa }

            self.eka = true;

            if self.cup_style > 0 || fourhills {
                for temp in 1..=NUM_PL {
                    //{ custom tai fourhills }
                    if self.pisteet[temp] != -5555 {
                        self.fourpts[temp] += self.pisteet[temp];
                    }
                }
            }

            if !self.cupslut {
                //{ 2.kierros tulokset }
                self.jarjestys(2, 1, NUM_PL as u8);
                self.updatestats(1);
                self.lista(2).await;
            }

            if !self.cupslut && sortby == 0 {
                //{ MC tulokset, jos niist� kisataan }
                for temp in 1..=NUM_PL {
                    if self.sija[temp] < 31 {
                        self.mcpisteet[temp] += WC_POINTS[self.sija[temp] as usize] as i32;
                    }
                }

                self.jarjestys(0, 0, NUM_PL as u8);
                self.updatestats(0);
                self.lista(4).await;
            }

            for temp in 0..=NUM_PL + 1 {
                self.luett[temp] = 0;
            }

            for temp in 1..=NUM_PL {
                if self.pisteet[temp] == -5555 {
                    self.pisteet[temp] = 0;
                }
            }

            if !self.cupslut && (sortby == 1 || fourhills) {
                //{ ja yhteispiste tulokset }
                self.jarjestys(1, 1, NUM_PL as u8);
                self.updatestats(2);
                self.lista(3).await;
            }

            self.updaterecords(sortby).await; //{ enn�tystauluja jos tarvis }

            if (self.osakilpailu == self.cup_hills) || (self.s.get_ch() == 27) || (self.cupslut) {
                break;
            }
        }

        self.mcpisteet[15] = 0; //{ ????? mik� 15? }

        self.wcup = false;

        //{ testing alleolevia! }

        self.i.write_profiles();
        self.write_records();
        self.make_send_me().await;
    }

    fn check_param(&mut self) {
        let str1 = self.s.param_str(1).into_bytes();

        if str1.len() > 1 {
            match str1[1] {
                b'r' => self.reset_hiscore(1),
                b'z' => self.reset_hiscore(0),
                b'c' => self.reset_config(),
                _ => {}
            }
        }
    }

    async fn draw_full_main(&mut self) {
        self.i.draw_main_menu().await;

        // TODO
        //{$IFDEF REG}
        self.u.new_reg_text(REGNAME, REGNUMBER).await;
        // {$ELSE}
        //self.u.new_unreg_text();
        // {$ENDIF}
    }

    async fn jump_menu(&mut self) {
        let mut index = 1;

        while index != 0 {
            self.u.main_menu_text(1, VERSION_FULL);

            index = self.u.make_menu(11, 97, 108, 12, 6, index, 8, 4, 0).await;

            match index {
                1 => {
                    self.cup_style = 0;
                    self.cup().await;
                }
                2 => {
                    self.cup_style = 1;
                    self.cup().await;
                }
                3 => {
                    self.cup_style = 2;
                    self.cup().await;
                }
                //{$IFDEF REG}
                4 => {
                    self.teamcup().await;
                }
                //{$ENDIF}
                5 => {
                    self.newkingofthehill().await;
                }
                6 => {
                    self.training().await;
                }
                _ => {}
            }

            if index != 0 {
                self.draw_full_main().await;
            }
        }
    }

    pub async fn main_menu(&mut self) {
        self.g.fill_box(0, 0, 319, 199, 0);

        let mut index = 1;

        if self.languagenumber == 255 {
            self.i.welcome_screen(&mut self.languagenumber).await;
            // TODO
            // Replays(true, version, gdetail);
        }

        loop {
            self.draw_full_main().await;
            self.u.main_menu_text(0, VERSION_FULL);
            index = self.u.make_menu(11, 97, 108, 12, 6, index, 8, 4, 2).await;

            match index {
                1 => {
                    self.jump_menu().await;
                }
                2 => {
                    //{ signupplayers; }
                    self.i
                        .load_names(self.namenumber, self.jmaara, &mut self.teamlineup, false);
                    self.i.profiles().await;
                    if self.i.pmaara.get() == 8 {
                        self.jmaara = 2;
                    } else {
                        self.jmaara = 1;
                    }
                    self.i.jnimet.borrow_mut()[NUM_TEAMS - 1] = b"Team 2".to_vec();
                    self.i
                        .load_names(self.namenumber, self.jmaara, &mut self.teamlineup, true);
                }
                3 => {
                    self.setup_menu().await;
                }
                4 => {
                    self.showtops(0).await;
                }
                5 => {
                    if self.u.num_extra_hills.get() > 0 {
                        self.showtops(1).await;
                    } else {
                        self.showtops(2).await;
                    }
                }
                6 => {
                    // replays(false, version, gdetail);
                }
                _ => {}
            }

            if index == 0 {
                index = self.u.quitting(0).await as i32;
            }

            if index == 0 {
                break;
            }
        }
    }

    pub async fn alku(&mut self) {
        writeln(b"Ported to SDL2 by Suomipelit (https://suomipelit.github.io)");
        writeln(b"");
        writeln(b"New shortkeys:");
        writeln(b" * Alt+Enter          : Toggle fullscreen");
        writeln(b" * Alt+(Keypad) Plus  : Increase window size");
        writeln(b" * Alt+(Keypad) Minus : Decrease window size");
        writeln(b" * Alt+R              : Reset window if stretched");
        writeln(b" * Alt+A              : Toggle 4:3 aspect ratio");
        writeln(b"");
        writeln(b"-------");
        writeln(b"");
        // This line of the Pascal source is in Latin-1, and the rest in CP437
        writeln_latin1(&[b"SJ3 v", VERSION, b" by Ville K\xF6n\xF6nen 2011"].concat());

        write(b"- Loading ANIM.SKI");
        self.g.load_anim("ANIM.SKI");

        self.eka = true;
        self.jcup = false;
        self.wcup = false;
        self.koth = false;
        self.cup_style = 0;

        self.check_param();

        write(b", HISCORE.SKI");
        self.read_records().await;

        write(b", CONFIG.SKI");
        self.read_config();

        write(b", PLAYERS.SKI");
        self.i.load_profiles();

        writeln(&[b", NAMES" as &[u8], &[self.namenumber], b".SKI"].concat());
        self.i
            .load_names(self.namenumber, self.jmaara, &mut self.teamlineup, true);

        write(b"  LANGBASE.SKI");
        if self.languagenumber != 255 {
            self.l.load_language(self.languagenumber);
        }

        self.u.check_extra_hills(); //{ lukee ylim. SJH:t ja kirjoittaa MOREHILL.SKI:n }

        write(b", MOREHILL.SKI");
        self.u.load_hill_info().await; //{ mnimet[],kri[] ja NumExtraHills kuntoon }

        write(b", Extra Hillrecords");
        if self.u.num_extra_hills.get() > 0 {
            self.u.read_extras().await;
        }

        self.m.alusta();
    }

    async fn loppu(&mut self) {
        // This line of the Pascal source is in Latin-1, and the rest in CP437
        writeln_latin1(&[b"SJ3 v", VERSION, b" by Ville K\xF6n\xF6nen 2013"].concat());

        self.now = self.u.get_now();

        write(b"- Writing HISCORE.SKI");
        self.write_records();

        write(b", CONFIG.SKI");
        self.write_config();

        write(b", PLAYERS.SKI");
        self.i.write_profiles();

        if self.u.num_extra_hills.get() > 0 {
            write(b", Extra Hillrecords");
            self.u.write_extras().await;
        }

        self.s.putsaa();

        self.make_send_me().await;

        write(b", SENDME.TXT.");

        start_end_text();
        textbackground(0);
        clrscr();

        dowindow(1, 2, 80, 14, 7);
        self.u.regendtext(REGNAME.as_bytes(), REGNUMBER.as_bytes());

        self.u.lopputext(VERSION_FULL, self.start, self.now);

        // The SDL port waits here until Crt.KeyPressed sees a key in the
        // terminal. The standard library cannot read a single key from the
        // terminal, so the remake exits without the wait.
    }

    pub async fn close(&mut self) {
        self.s.deinit_graphics();

        self.loppu().await;
    }

    pub async fn close_and_halt(&mut self) -> ! {
        self.close().await;
        halt().await;
    }

    // originally in SJ3UNIT.PAS
    fn draw_lumi(&mut self, delx: i32, dely: i32, wind: i32, lmaara: u16, draw: bool) {
        if lmaara > 0 {
            self.lumi.update(
                self.m.video.borrow_mut(),
                delx * 2,
                dely * 2,
                wind * 8,
                draw,
            );
        }
    }
}
