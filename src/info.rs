use crate::graph::GraphModule;
use crate::lang::LangModule;
use crate::pcx::PcxModule;
use crate::unit::{Hiscore, NUM_PL};

pub struct Profile {
    name: Vec<u8>,
    realname: Vec<u8>,
    suitcolor: u8,
    skicolor: u8,
    kothlevel: u8,
    replace: u8,
    bestwchill: u8,
    besthill: u8,
    cstyle: u8,
    skipquali: u8,
    bestwcjump: u16,
    bestpoints: u16,
    best4points: u16,
    bestjump: u16,
    besthillfile: String,
    bestresult: String,
    best4result: String,
    wcs: i32,
    legswon: i32,
    wcswon: i32,
    totaljumps: i32,
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
            besthillfile: String::from("HILLBASE"),
            bestresult: String::from("-"),
            best4result: String::from("-"),
            totaljumps: 0,

            skipquali: 0,
            bestpoints: 0,
            best4points: 0,
        }
    }
}

const NUM_HISCORES: u32 = 61; //{ 1-20=WC + 21-30=TC + 31-35=4h + 36-41=KOTH + 42-61=CC }

pub struct InfoModule<'g, 'l, 'm, 'p, 's, 'si> {
    g: &'g GraphModule<'m, 's, 'si>,
    l: &'l LangModule,
    p: &'p PcxModule<'m, 's, 'si>,
    top: Vec<Hiscore>,
    pub nimet: Vec<Vec<u8>>,
    pub jnimet: Vec<Vec<u8>>,
    pub pmaara: u8,
    num_profiles: u8, //{ piilota alas }
    profileorder: [u8; 21],
    profile: Vec<Profile>, //{ ehk� my�s }
}

impl<'g, 'l, 'm, 'p, 's, 'si> InfoModule<'g, 'l, 'm, 'p, 's, 'si> {
    pub fn new(
        g: &'g GraphModule<'m, 's, 'si>,
        l: &'l LangModule,
        p: &'p PcxModule<'m, 's, 'si>,
    ) -> Self {
        let profiles = vec![Profile::new()];
        InfoModule {
            g,
            l,
            p,
            top: vec![],
            nimet: profiles.iter().map(|p| p.name.clone()).collect(),
            jnimet: vec![],
            pmaara: 1,
            num_profiles: 1,
            profileorder: [0; 21],
            profile: profiles,
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

        for temp in 1..=self.pmaara {
            let y = (temp - 1) * 9 + 64;
            self.g
                .e_write_font(x + 12, y as i32, (temp.to_string() + ".").as_bytes());
            self.g
                .write_font(x + 20, y as i32, &self.nimet[(temp - 1) as usize]);

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
