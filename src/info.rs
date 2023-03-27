use crate::graph::GraphModule;
use crate::lang::LangModule;
use crate::pcx::PcxModule;
use crate::rs_util::{parse_line, read_line};
use crate::unit::{valuestr, Hiscore, NUM_PL, NUM_TEAMS};
use std::cell::{Cell, RefCell};
use std::fs::File;
use std::io::BufReader;

pub struct Profile {
    name: Vec<u8>,
    realname: Vec<u8>,
    pub suitcolor: u8,
    pub skicolor: u8,
    kothlevel: u8,
    replace: u8,
    pub bestwchill: u8,
    pub besthill: u8,
    pub cstyle: u8, // coach style?
    skipquali: u8,
    pub bestwcjump: u16,
    bestpoints: u16,
    best4points: u16,
    pub bestjump: u16,
    pub besthillfile: Vec<u8>,
    bestresult: Vec<u8>,
    best4result: Vec<u8>,
    wcs: i32,
    legswon: i32,
    wcswon: i32,
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

pub struct InfoModule<'g, 'l, 'm, 'p, 's, 'si> {
    g: &'g GraphModule<'m, 's, 'si>,
    l: &'l LangModule,
    p: &'p PcxModule<'m, 's, 'si>,
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

impl<'g, 'l, 'm, 'p, 's, 'si> InfoModule<'g, 'l, 'm, 'p, 's, 'si> {
    pub fn new(
        g: &'g GraphModule<'m, 's, 'si>,
        l: &'l LangModule,
        p: &'p PcxModule<'m, 's, 'si>,
    ) -> Self {
        let profiles = (0..=20).map(|_| Profile::new()).collect::<Vec<_>>();
        InfoModule {
            g,
            l,
            p,
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

    pub fn draw_main_menu(&self) {
        self.g.fill_box(0, 0, 319, 199, 0);
        self.g.draw_screen();
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

    fn default_profile(&self, index: i32, phase: u8) {
        /*
        procedure defaultprofile(index:integer;phase:byte);
        var temp : integer;
            count, count2 : byte;
            ok : boolean;

        begin
        */

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

    pub fn load_names(&self, whatset: i8, teamdiscount: u8, teamlineup: &mut [u8], replaces: bool) {
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

        if whatset == 0 {
            let l2: i32 = parse_line(&mut f4).unwrap(); //{ haetaan se names:n checksum }
            l1 += 5661;
            if l1 != l2 {
                panic!("Error #92: Something wrong with the namefile. Shit.");
            }
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
