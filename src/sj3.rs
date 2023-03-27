use crate::graph::GraphModule;
use crate::help::{nsqrt, txt, txtp};
use crate::info::InfoModule;
use crate::lang::LangModule;
use crate::lumi::LumiModule;
use crate::maki::MakiModule;
use crate::pcx::{PcxModule, NUM_SKIS, NUM_SUITS};
use crate::regfree::{REGNAME, REGNUMBER};
use crate::rs_util::{parse_line, random, read_line};
use crate::sdlport::SDLPortModule;
use crate::table::{
    find_landing, jump_risk, lasku_anim, lasku_asento, lento_anim, parru_anim, ponn_anim,
    suksi_laskussa, suksi_lennossa,
};
use crate::tuuli::TuuliModule;
use crate::unit::{
    dayandtime_now, defaultkeys, injured, kword, loadgoal, uncrypt, valuestr, Hill, Hiscore, Stat,
    Time, UnitModule, NUM_PL, NUM_TEAMS, NUM_WC_HILLS,
};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::str::from_utf8;

const VERSION: &[u8] = b"3.13-remake0";
const VERSION_FULL: &[u8] = b"3.13-remake0";

pub struct SJ3Module<'g, 'i, 'l, 'm, 'p, 's, 'si, 't, 'u> {
    g: &'g GraphModule<'m, 's, 'si>,
    i: &'i InfoModule<'g, 'l, 'm, 'p, 's, 'si>,
    l: &'l LangModule,
    lumi: LumiModule,
    m: &'m MakiModule,
    p: &'p PcxModule<'m, 's, 'si>,
    s: &'s SDLPortModule<'si>,
    tuuli: &'t TuuliModule<'g, 'm, 's, 'si>,
    u: &'u UnitModule<'g, 'l, 'm, 'p, 's, 'si>,

    act_hill: Hill,
    nosamename: bool,
    automatichrr: bool,
    jmaara: u8,
    nytmaki: i32,
    wcup: bool,
    jcup: bool,
    cupslut: bool,
    //today: Date,
    now: Time,
    start: Time,
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
    namenumber: i8,
    languagenumber: u8,
    gdetail: u8,
    seecomps: u8,
    teamlineup: [u8; 61],
    keula_x: i32, /*{ oltava global?
                  juu, koska se lukee sen vain m�en latauksessa,
                  ja muuten se unohtuu... }*/
    this_is_a_hill_record: i32,
}

impl<'g, 'h, 'i, 'l, 'm, 'p, 's, 'si, 't, 'u> SJ3Module<'g, 'i, 'l, 'm, 'p, 's, 'si, 't, 'u> {
    pub fn new(
        g: &'g GraphModule<'m, 's, 'si>,
        i: &'i InfoModule<'g, 'l, 'm, 'p, 's, 'si>,
        l: &'l LangModule,
        lumi: LumiModule,
        m: &'m MakiModule,
        p: &'p PcxModule<'m, 's, 'si>,
        s: &'s SDLPortModule<'si>,
        tuuli: &'t TuuliModule<'g, 'm, 's, 'si>,
        u: &'u UnitModule<'g, 'l, 'm, 'p, 's, 'si>,
    ) -> Self {
        SJ3Module {
            g,
            i,
            l,
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
            namenumber: 0,
            languagenumber: 0,
            gdetail: 0,
            seecomps: 0,
            teamlineup: [0; 61],
            keula_x: 0,
            this_is_a_hill_record: 0,
        }
    }

    fn setupmenu(&self) {
        unimplemented!();
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

    fn drawwcinfo(&self) {
        if self.jcup {
            self.g.e_write_font(308, 9, self.l.lstr(71));
        } else {
            self.g.e_write_font(308, 9, self.l.lstr(70));
        }

        for temp in 1..=5 {
            if self.mcpisteet[self.mcluett[temp] as usize] > 0 {
                let mut str1 = txtp(self.mcpisteet[self.mcluett[temp] as usize]);

                if self.diffwc && temp > 1 {
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
                    self.l.lstr(242),
                    b": ",
                    &txtp(loadgoal(self.nytmaki)) as &[u8],
                ]
                .concat(),
            );
        }
    }

    fn hyppy(&mut self, index: i32, pel: i32, team: i32) {
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

        let mut lmaara: u16 = 0;

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

        let mut f1: File;

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

        kr = self.act_hill.kr;
        self.s.ch.set(0);
        rturns = 0;

        if self.eka {
            lmaara = random(2) as u16 * random(256) as u16;
            if (lmaara > 0) && (lmaara < 40) {
                lmaara = 41 + random(150) as u16;
            }
            if (lmaara > 0) && (random(4) == 0) {
                lmaara += 1000; //{ r�nt� }
            }
            //{   Lmaara:=1151; }

            self.lumi.vie_lmaara(lmaara);

            /*{   LStyle:=Random(2)*Random(2);
            LStyle:=1; }*/

            if self.gdetail == 1 {
                lmaara = 0;
            }

            if self
                .u
                .load_hill(&mut self.keula_x, self.nytmaki, &self.act_hill)
                != 0
            {
                self.s.ch.set(27);
                self.cupslut = true;
            }

            self.m.x.set(0);
            self.m.y.set(0);
            self.g.draw_hill_screen();

            self.tuuli.hae();
        }

        self.p.siirra_standardi_paletti();

        if lmaara > 1000 {
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

        skill = 17 - f32::round(nsqrt(temp as f32) / 4f32) as u8;
        if skill > 16 {
            skill = 16;
        }

        reflex = f32::round((34 + pel + random(10) as i32) as f32 / 5f32) as u8;
        /*
        {  reflex:=round((33+pel+random(8))/4); }
        {  reflex:=round((45+pel+random(10))/5); }
        {  reflex:=round((30+1+random(30))/5); }
        */

        //{ floppihyppy }
        if random((300 - pel) as u32) == 0 {
            skill = (17 + random((3 + pel / 25) as u32)) as u8;
        }

        self.now = Time::now(); //{ haetaan current time }

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
            temp = loadgoal(self.nytmaki);
        }

        if draw {
            loop {
                matka += 1.0;

                x = f32::round(matka + qx) as i32;
                kor = self.m.profiili(x) as f32;
                kkor = kor - keula_y as f32;
                hp = f32::round(nsqrt((matka * matka) + (kkor * kkor)) * self.act_hill.pk * 0.5)
                    as i32
                    * 5;

                if hp >= self.u.hrlen(self.nytmaki) && self.u.hrlen(self.nytmaki) > 0 && wrx == 0 {
                    wrx = x;
                    wry = f32::round(kor) as i32 - 9;
                }

                if hp >= temp && temp != 0 && goalx == 0 {
                    goalx = x;
                    goaly = f32::round(kor) as i32 - 7;
                }

                if x > 1024 {
                    break;
                }
            }
        }

        hp = 0;
        kkor = 0.0;

        matka = -self.keula_x as f32 + 10.0;
        qx = self.keula_x as f32 + 0.5; //{ emme halua, ett� se toteaa keulalla matkan>0 }

        x = f32::round(matka + qx) as i32;

        kor = self.m.profiili(x) as f32;

        y = f32::round(kor) as i32;

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

        // jajesta5
        {
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
                        if self.pisteet[apu1 as usize] > self.pisteet[top5[apu2 as usize] as usize]
                        {
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
        };

        delta_x = self.m.x.get();
        delta_y = self.m.y.get();

        laskuri = 0; //{ t�t� k�ytet��n laskurina seuraavassa }

        self.p.aseta_paletti();

        self.tuuli.hae();

        if draw && self.s.ch.get() != 27 {
            loop {
                //{ INFORUUTU LUUPPI! }
                laskuri += 1;

                self.m.tulosta();

                self.draw_lumi(
                    delta_x - self.m.x.get(),
                    delta_y - self.m.y.get(),
                    self.tuuli.value.get(),
                    lmaara,
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
                            &txt(self.pisteet[pel as usize]) as &[u8],
                            b" (",
                            &txt(self.cstats[1][pel as usize]),
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

                // drawinfo
                {
                    if (self.pisteet[top5[1] as usize] == 0) || (self.kierros < 0) {
                        //{ ei tuloksia n�ytett�v�n� }
                        if (self.wcup || self.jcup)
                            && (self.mcpisteet[self.mcluett[1] as usize] > 0)
                        {
                            match laskuri {
                                0..=130 => self.drawhrinfo(),
                                146..=276 => self.drawwcinfo(),
                                291 => laskuri = 0,
                                _ => (),
                            }
                        } else {
                            self.drawhrinfo()
                        }
                    } else {
                        //{ on tuloksia }
                        if self.koth {
                            match laskuri {
                                0..=130 => {
                                    // drawkothinfo
                                    let str1 = [
                                        self.l.lstr(67),
                                        b" ",
                                        &txt(1
                                            + self.kothmaara as i32
                                            + self.i.pmaara.get() as i32
                                            - self.mcpisteet[0]),
                                        b" ",
                                        self.l.lstr(8),
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
                                            str1 = txtp(
                                                self.pisteet[top5[1] as usize]
                                                    - self.pisteet[pel as usize],
                                            );
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
                                146..=276 => self.drawhrinfo(),
                                291 => laskuri = 0,
                                _ => (),
                            }
                        } else {
                            match laskuri {
                                0..=130 => {
                                    // drawtop5info
                                    let text = [
                                        &self.act_hill.name as &[u8],
                                        b" K",
                                        &txt(self.act_hill.kr),
                                    ]
                                    .concat();
                                    self.g.e_write_font(308, 9, &text);

                                    for temp in 0..5 {
                                        if self.pisteet[top5[temp] as usize] > 0 {
                                            let mut str1 = txtp(self.pisteet[top5[temp] as usize]);

                                            if self.diff && temp > 1 {
                                                str1 = txtp(
                                                    self.pisteet[top5[temp] as usize]
                                                        - self.pisteet[top5[1] as usize],
                                                );
                                            }
                                            if self.jcup {
                                                str1 = [
                                                    &self.i.jnimet.borrow()[top5[temp] as usize]
                                                        as &[u8],
                                                    b"$",
                                                    &str1,
                                                ]
                                                .concat();
                                            } else {
                                                str1 = [
                                                    &self.i.nimet.borrow()[top5[temp] as usize]
                                                        as &[u8],
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

                                    let temp =
                                        self.pisteet[top5[1] as usize] - self.pisteet[pel as usize];
                                    if temp > 0 {
                                        self.g.e_write_font(
                                            308,
                                            62,
                                            &([str1, b": ", &txtp(temp + 1)].concat()),
                                        );
                                    }
                                }
                                146..=276 => self.drawhrinfo(),
                                291..=422 => {
                                    if self.mcpisteet[self.mcluett[1] as usize] > 0 {
                                        self.drawwcinfo()
                                    } else {
                                        laskuri = 0;
                                    }
                                }
                                437 => laskuri = 0,
                                _ => (),
                            }
                        }
                    }
                }

                self.g.draw_screen();

                self.s.ch.set(1);

                if self.s.key_pressed() {
                    self.s.wait_for_key_press();

                    if self.s.ch.get() == 0 && self.s.ch2.get() == 68 {
                        self.cupslut = true;
                        self.s.ch.set(27);
                    }

                    if self.s.ch.get().to_ascii_uppercase() == self.l.lch(60, 1) {
                        self.setupmenu();
                        self.s.ch.set(1);
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
                        if self.s.ch.get() == b'+' {
                            self.startgate += 1;
                            self.s.ch.set(1);
                        }
                        if self.s.ch.get() == b'-' {
                            self.startgate -= 1;
                            self.s.ch.set(1);
                        }
                        if self.startgate < 1 {
                            self.startgate = 1;
                        }
                        if self.startgate > 30 {
                            self.startgate = 30;
                        }
                    }
                }

                if self.s.ch.get() != 1 {
                    break;
                }
            }
        }

        if self.treeni {
            maxspeed += (self.startgate - 15) as u8;
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

        if self.s.ch.get() != 27 && draw {
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
                    px += (self.tuuli.value.get() - 20) as f32 / 10000.0;

                    sx += f32::round(temp as f32 * px) as i32;
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
                    lmaara,
                    true,
                );

                self.g.write_font(
                    x - self.m.x.get() + 15,
                    y - self.m.y.get() - 15,
                    &txt(statsvictim),
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
                    // drawkeymap
                    self.g.draw_anim(227, 2, 64);

                    self.g.font_color(246);
                    self.g.e_write_font(308, 9, self.l.lstr(330));

                    for temp in 1..=5 {
                        self.g.e_write_font(
                            308,
                            temp * 10 + 9,
                            &([
                                self.l.lstr((330 + temp) as u32),
                                b": ",
                                &self.u.keyname(self.k[temp as usize]),
                            ]
                            .concat()),
                        );
                    }
                    self.g.font_color(247);
                }

                self.g.draw_screen();

                self.s.ch.set(0);
                self.s.ch2.set(0);

                if self.s.key_pressed() {
                    let (ch, ch2) = self.s.wait_for_key_press();
                    self.s.ch.set(ch);
                    self.s.ch2.set(ch2);
                    self.s.ch.set(self.s.ch.get().to_ascii_uppercase());
                    if self.s.ch.get() == 0 && self.s.ch2.get() == 68 {
                        self.cupslut = true;
                        self.s.ch.set(27);
                    }
                    if !cjumper && (self.s.kword() == self.k[2] || self.s.ch.get() == 13) {
                        out = true; //{ liikkeelle }
                    }
                    if self.s.ch.get() == 0
                        && self.s.ch2.get() == 63
                        && self.wcup
                        && self.cup_style == 1
                        && index == NUM_PL as i32
                        && self.kierros == 1
                        && self.osakilpailu == 1
                    {
                        self.tuuli.alusta(self.windplace);
                    }
                    if self.s.ch.get() == 27 {
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
                        kulmalaskuri = 4000;
                    }
                    if self.s.kword() == self.k[1] {
                        kulmalaskuri = 1000;
                    }
                    sx = 0;
                    sy = 0;
                }

                jumper_anim = parru_anim(&mut kulmalaskuri);

                if self.s.ch.get() == 27 && cjumper {
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
            //if (beeppi) then beep(2);
            self.p.muuta_logo(4); //{ vihre� pois }
            self.p.aseta_paletti();
            self.g
                .draw_anim(x - self.m.x.get() + 60, y - self.m.y.get() - 10, 67); //{ liikennevalo }

            self.g.draw_anim(3, 150, 65);

            self.g.font_color(240);
            self.g.write_font(
                12,
                160,
                &[&namestr as &[u8], b" ", self.l.lstr(79)].concat(),
            );

            self.g.draw_screen();

            self.s.wait_for_key_press();
            self.s.ch.set(27);
        }

        out = false;

        laskuri = 0;
        px = 0.0;

        if self.s.ch.get() == 27 && draw && cjumper {
            //{ emm� haluukkaan tsiigata }
            draw = false;
            self.s.ch.set(0);
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

        if self.s.ch.get() != 27 {
            //{ hyppy - main }
            loop {
                self.tuuli.hae();

                matka += px * 0.01;

                fx = x; //{ former x & y }
                fy = y;

                x = f32::round(matka + qx) as i32;

                if self.s.ch.get() != 27 {
                    self.s.ch.set(0); //{ en tied� muisteleeko se vanhoja }
                }
                self.s.ch2.set(0);

                if self.s.key_pressed() {
                    let (ch, ch2) = self.s.wait_for_key_press();
                    self.s.ch.set(ch.to_ascii_uppercase());
                    self.s.ch2.set(ch2);

                    if self.s.ch.get() == 0 && self.s.ch2.get() == 68 {
                        self.cupslut = true;
                        self.s.ch.set(27);
                        out = true;
                    }

                    if self.s.ch.get() == 27 {
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
                        self.s.ch.set(0);
                        self.s.ch2.set(0);
                    }

                    if self.s.ch.get() == b'P' {
                        self.s.wait_for_key_press();
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
                            self.s.ch.set(((self.k[2] & 0xff00) >> 8) as u8);
                            self.s.ch2.set((self.k[2] & 0xff) as u8);
                        }
                        if lento_anim(kulma1) < 107 && laskuri % (reflex as i32) == 0 {
                            self.s.ch.set(((self.k[3] & 0xff00) >> 8) as u8);
                            self.s.ch2.set((self.k[3] & 0xff) as u8);
                        }

                        temp = find_landing(self.makikulma(x));

                        if clanding == 2 {
                            temp = f32::round(temp as f32 * 0.6) as i32;
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
                                    clanding = 1;
                                }
                            }
                        }
                    } //{ if (cjumper) }

                    if kword(self.s.ch.get(), self.s.ch2.get()) == self.k[4] {
                        landing = 1;
                    }
                    if kword(self.s.ch.get(), self.s.ch2.get()) == self.k[5] {
                        landing = 2;
                    }

                    if kword(self.s.ch.get(), self.s.ch2.get()) == self.k[3] && kulma1 <= 600 {
                        kulma1 += kulma1 / 4;
                    }
                    if kword(self.s.ch.get(), self.s.ch2.get()) == self.k[2]
                        && landing == 0
                        && kulma1 > 0
                    {
                        kulma1 -= kulma1 / 5;
                    }

                    //{ ylirotaatio }
                    if kulma1 < 50 {
                        pl += 0.0001 - (kulma1 - 50) as f32 / 18000.0;
                    }

                    if self.tuuli.value.get() > 0 {
                        pl -= (1.0 - kulma1 as f32 / 900.0) / 1875.0
                            + f32::sqrt(f32::sqrt(2.0 * self.tuuli.value.get() as f32)) / 65500.0;
                    } else {
                        pl -= (1.0 - kulma1 as f32 / 900.0) / 1875.0
                            - f32::sqrt(f32::sqrt(-2.0 * self.tuuli.value.get() as f32)) / 65500.0;
                    }

                    px -= kulma1 as f32 / 900.0 / 20.0
                        + (nsqrt(4.0 * self.tuuli.value.get() as f32 + 245.0) - 16.0) / 400.0;

                    t += 0.01;

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
                            pl -= random(self.tuuli.voim.get() as u32 + 50) as f32 / 15000.0;
                        }

                        if temp < 0 {
                            //{ puuska vie suksia alas }
                            ssuunta = 6;
                            pl += random(self.tuuli.voim.get() as u32 + 50) as f32 / 15000.0;
                        }
                    }

                    if pl < 0.105 {
                        pl = 0.105; //{ leijuntaesto :) }
                    }

                    kor += (t * t * pl) - ((py - 8.0) / 100.0);

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
                                pl += 0.003;
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
                    height = self.m.profiili(x) - f32::round(kor) as i32;

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
                    if kword(self.s.ch.get(), self.s.ch2.get()) == self.k[1]
                        && matka > -40.0
                        && ponnistus == 0
                    {
                        ponnistus += 1;
                    }

                    //{ Automaattiponnistus }
                    if cjumper && matka > -(skill as f32 * px * 0.01) && ponnistus == 0 {
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
                        px += 0.21;
                        py += 1.21;
                        kulma1 += 12;

                        if ponnistus > 16 {
                            //{ Ponnistus liian aikaisin }
                            if ponnistus == 17 {
                                pl += 0.023;
                                //if (not cjumper) and (beeppi) then beep(1);
                            }

                            pl += 0.013;
                            py -= 1.0;

                            if self.kierros > 0 {
                                self.stats[statsvictim as usize][self.osakilpailu as usize]
                                    .reason[self.kierros as usize] = 1;
                            }

                            kulma1 = 158; //{ onko tarpeellinen? }
                        }
                    }
                } //{ end matkariippuvaiset }

                x = f32::round(matka + qx) as i32;
                y = f32::round(kor) as i32;

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
                        lmaara,
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

                    self.g.draw_screen();
                }

                if out {
                    break;
                }
                //{ ***  LENTO LOPPUU  *** }
            }
        }

        kkor = kor - keula_y as f32;
        hp = f32::round(nsqrt((matka * matka) + (kkor * kkor)) * self.act_hill.pk * 0.5) as i32 * 5;

        if self.s.ch.get() == 27 {
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
                grade = (hp / kr * 10) as u8;
            }

            temp = self.makikulma(x);
            height = f32::round(temp as f32 * 1.34 + kulma1 as f32 / 10.0) as i32;

            riski = jump_risk(temp);
            if hp < 20 * kr / 3 {
                riski = 1; //{ lyhyiden hyppyjen riski? }
            }

            if height < 63 {
                riski = f32::round(riski as f32 * (1.0 + ((63 - height) as f32 * 0.075))) as i32;
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

            for temp in 1..=((kr + (kr / 20) - (hp / 10)) / 6) {
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
                score += f32::round(((hp / 10) - (kr * 2 / 3)) as f32 * (180.0 / kr as f32) * 10.0)
                    as i32; //{ pituuspisteet }
            }

            /*(*
              if (kr<>0) then { uusi fis:in mukainen }
               inc(score,round((hp-(kr*10))*(lengthpoint(kr)/10)+600));  { pituuspisteet }

              if (kr>=160) then inc(score,600); { lentom�ist� saa isot pisteet }
            *)*/

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
                        profile[actprofile as usize].besthillfile = self.u.hillfile(self.nytmaki);
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
                self.u
                    .set_hrinfo(self.nytmaki, namestr.clone(), hp, dayandtime_now());

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

            out = false;

            if kupat > 0 {
                grade = kupat;
            }

            if draw {
                //{ ***  LASKU  *** }
                loop {
                    self.tuuli.hae();

                    laskuri += 1;

                    matka += px * 0.008;
                    umatka += upx * 0.008;

                    if ok {
                        matka = umatka;
                    }

                    fx = x;
                    fy = y;

                    x = f32::round(matka + qx) as i32;
                    ux = f32::round(umatka + qx) as i32;

                    kor = self.m.profiili(x) as f32;
                    ukor = self.m.profiili(ux) as f32;

                    ski_anim = suksi_laskussa(self.makikulma(x));

                    jumper_anim = lasku_anim(ski_anim as i32, landing); //{ t�t� voi my�s sitten muuttaa jos haluaa }

                    if laskuri < 7 && landing > 0 {
                        jumper_anim = 113 + landing; //{ esilaskeutuminen }
                    }

                    if kupat > 0 {
                        if laskuri > 50 && upx > 0.0 {
                            upx -= 0.8;
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

                    self.s.ch.set(0);

                    if self.s.key_pressed() {
                        let (ch, ch2) = self.s.wait_for_key_press();
                        self.s.ch.set(ch.to_ascii_uppercase());
                        self.s.ch2.set(ch2);
                    }

                    if self.s.ch.get() == 0 && self.s.ch2.get() == 68 {
                        self.cupslut = true;
                        self.s.ch.set(27);
                    }
                    if self.s.ch.get() == 27 || self.s.ch.get() == 13 {
                        out = true;
                    }
                    if self.s.ch.get() == b'P' {
                        let (ch, ch2) = self.s.wait_for_key_press();
                        self.s.ch.set(ch);
                        self.s.ch2.set(ch2);
                    }

                    delta_x = self.m.x.get();
                    delta_y = self.m.y.get();

                    x = f32::round(matka + qx) as i32;
                    y = f32::round(kor) as i32;

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
                        lmaara,
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
                        f32::round(ukor) as i32 - self.m.y.get() - 2,
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
                            self.g.e_write_font(
                                temp2 - 1 + random(3) as i32,
                                32 + random(3) as i32,
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

                    self.g.draw_screen();

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
                    str1 = [
                        &txt(self.inj[pel as usize] as i32) as &[u8],
                        b" ",
                        self.l.lstr(76),
                    ]
                    .concat();

                    match self.inj[pel as usize] {
                        1 => str1 = self.l.lstr(77).to_vec(),
                        2 => str1 = self.l.lstr(78).to_vec(),
                        _ => (),
                    }
                }
            }

            if !draw && self.gdetail == 0 {
                self.draw_lumi(
                    delta_x - self.m.x.get(),
                    delta_y - self.m.y.get(),
                    self.tuuli.value.get(),
                    lmaara,
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

            self.g.draw_screen();

            if draw {
                self.s.putsaa();
                self.cupslut = self.s.wait_for_key2();
            } else if self.s.key_pressed() {
                self.s.wait_for_key_press();
                if self.s.ch.get() == 0 && self.s.ch2.get() == 68 {
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

    fn jumpalku(&mut self) {
        self.u.load_info(self.nytmaki, &mut self.act_hill);

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

    //{ cupstyle: 0 - SJ3 WC, 1 - Custom WC, 2 - Just 4Hills }
    fn cup(&mut self) {
        let mut temp = 0;
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

        self.s.ch.set(1);

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
            self.jumpalku();
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

            // TODO: remove!!!
            self.trainrounds = 1;

            if !self.cupslut && self.trainrounds > 0 {
                for temp in 1..=self.trainrounds {
                    self.kierros = -(temp as i32);

                    for index in (1..=NUM_PL).rev() {
                        if self.inj[self.mcluett[index] as usize] == 0 {
                            //{ onko loukkaantunut? }
                            if !self.cupslut {
                                self.hyppy(index as i32, self.mcluett[index] as i32, 0);
                                self.eka = false;
                            }
                        }
                    }
                }
            }
            /*
                 if not (cupstyle=1) then  { v3.13 - customissa skipataan quali }
                 begin

                      kierros:=0;

                      { QUAL ROUND  kierros:=0; }

                      if (not CupSlut) then { NEW! }
                         for index:=NumPl downto 1 do
                             begin
                             skip:=false;
                             if (mcluett[index] > NumPl-pmaara) then
                             begin
                                  skipquali:=Profile[profileorder[Numpl+1-mcluett[index]]].skipquali;
                                  if (skipquali=2) or ((not fourhills) and (skipquali=1)) then skip:=true;
                             end;

                             if (inj[mcluett[index]]=0) and (not cupslut) then
                             if not ((mcluett[index] > NumPl-pmaara) and (skip) and
                                (qual[mcluett[index]] > 0)) then  { skipquali mahis }
                             begin
                               hyppy(index,mcluett[index],0);
                               eka:=false;
                             end;
                    end;

                    jarjestys(2,1,Numpl);

                    if (not cupslut) and (osakilpailu>1) then { ei eka kisa }
                    begin
                         temp2:=1;

                         for temp:=1 to 50-qual[0] do  { yleens� 30 }
                         begin
                             while ((qual[luett[temp2]]>0) or (inj[luett[temp2]]<>0)) and (temp2<NumPl) do inc(temp2);
                             qual[luett[temp2]]:=1;
                         end;

                         if (not dokosystem) then { ko systeemiss� tarvitaan tasan 50. }
                            for temp:=51-qual[0] to NumPl do
                                if (sija[luett[temp]]=sija[luett[temp2]]) and (inj[luett[temp]]=0) then qual[luett[temp]]:=1;

                    end else
                    begin { * eka kerta * }

                          if (dokosystem) then
                          begin
                               for temp:=1 to 50 do
                                   qual[luett[temp]]:=3; { vain tasan 50 tarvitaan KO systeemiin }
                          end else
                              for temp:=1 to NumPl do    { 50 parasta qualifieria, ei v�li� vaikka loukkaant. }
                                  if (sija[temp]<51) then qual[temp]:=1;

                   end;

            {       for a:=50 to NumPl do
                    if (sija[luett[a] }

            {      for a:=1 to NumPl do luett[a]:=0; }

                  temp2:=1;

                  if (dokosystem) then
                   for temp:=1 to 50 do { pit�isi tehd� parit }
                    begin
                     while (qual[luett[temp2]]=0) and (temp2<NumPl) do inc(temp2);
                     qual[luett[temp2]]:=temp; { qualista tulikin l�ht�j�rjestysnumero }
                     inc(temp2);
                    end;

                  if (not cupslut) then { karsinnan tulokset }
                   begin
                    jarjestys(2,1,NumPl);
                    lista(0);
                   end;

                  if (not cupslut) and (dokosystem) then
                   begin
                    for temp:=1 to NumPl do   { tehd��n l�ht�j�rjestyslista }
                     luett[qual[temp]]:=temp;
                    showpairs(0);
                   end;


                 end { cupstyle <> 1 }
                 else
                 begin
                      for temp:=1 to NumPl do qual[temp]:=1;
                 end;

                 for temp:=0 to NumPl do
                  pisteet[temp]:=0;

                 for temp:=1 to NumPl do
                  if (qual[temp]=0) then pisteet[temp]:=-5555;  { pistet��n niille niin huonot pisteet ettei ne kummittele }

                  kierros:=1;

                 if (dokosystem) then                  { *** 1. KIERROS *** }
                  for temp:=25 downto 1 do
                   for temp2:=0 to 1 do
                    begin
                     index:=51-temp;
                     if (temp2=1) then index:=temp;

                     if (inj[luett[index]]=0) and (not cupslut) then
                      begin
                       hyppy(index,luett[index],0);
                       eka:=false;
                      end;

                   end else
                  for index:=NumPl downto 1 do
                   if (qual[mcluett[index]]>0) and (inj[mcluett[index]]=0) and (not cupslut) then
                    begin
                     hyppy(index,mcluett[index],0);
                     eka:=false;
                    end;

                if (not dokosystem) and (not cupslut) then { 1. kierroksen tulokset }
                 begin
                  jarjestys(2,1,NumPl);
                  updatestats(1);
                  lista(1);
                 end;

                 for temp:=0 to NumPl do qual[temp]:=0;

                if (not dokosystem) then { tavallinen }
                 begin
                  for temp:=1 to NumPl do
                  if (sija[temp]<31) then qual[temp]:=1; { n�m� ovat luettelon j�lkeen, ett� siell� tiedet��n koska hypp��j�t loppuu }
                 end else
                  begin                 { ko system }
                   for temp:=25 downto 1 do
                    if (pisteet[luett[temp]] >= pisteet[luett[51-temp]]) then
                     qual[luett[temp]]:=1 else qual[luett[51-temp]]:=1;
                   for temp:=1 to NumPl do
                    mcluett[temp]:=luett[temp]; { laitetaan talteen l�ht�j�rjestys }

                   jarjestys(2,1,NumPl); { tehd��n pisteist� j�rkk� }
                   temp2:=1;
                   for temp:=1 to 5 do     { 5 lucky loseria }
                    begin
                     while (qual[luett[temp2]]<>0) and (temp2<NumPl) do inc(temp2);
                     qual[luett[temp2]]:=2;
                    end;

                   for temp:=1 to NumPl do
                    luett[temp]:=mcluett[temp]; { palautetaan l�ht�j�rjestys }

                   if (not cupslut) then showpairs(1);

                   jarjestys(0,0,Numpl); { ett� mcliivi on oikealla miehell� }
                   jarjestys(2,1,NumPl); { taas pisteist� j�rkk� n�et toinen kierros normaali on }

                  end;

                 kierros:=2;

                 if (not cupslut) then        { *** 2. KIERROS *** }
                  for index:=NumPl downto 1 do
                   if (inj[luett[index]]=0) and (qual[luett[index]]>0) and (not cupslut) then
                    begin
                     hyppy(index,luett[index],0);
                     eka:=false;
                    end;

                  ch:=#1; { ettei hypyn keskeytt�minen sotke liikaa }
            {
                  for temp:=1 to pmaara do
                   if (sija[NumPl+1-temp]>30) then stats[temp,osakilpailu].Reason[2]:=4;}  { Ei selvinnyt tokalle }

                     eka:=true;

                if (cupstyle>0) or (fourhills) then
                 for temp:=1 to NumPl do { custom tai fourhills }
                  if (pisteet[temp]<>-5555) then inc(fourpts[temp],pisteet[temp]);

                if (not cupslut) then { 2.kierros tulokset }
                 begin
                  jarjestys(2,1,NumPl);
                  updatestats(1);
                  lista(2);
                 end;

                if (not cupslut) and (sortby=0) then { MC tulokset, jos niist� kisataan }
                 begin

                  for temp:=1 to NumPl do
                   if (sija[temp]<31) then inc(mcpisteet[temp],WCPoints[sija[temp]]);

                  jarjestys(0,0,NumPl);
                  updatestats(0);
                  lista(4);

                 end;

            {     for temp:=1 to NumPl do
                   if (inj[temp]>0) then dec(inj[temp]); }

                for temp:=0 to NumPl+1 do luett[temp]:=0;

                for temp:=1 to NumPl do if (pisteet[temp]=-5555) then pisteet[temp]:=0;

                if (not cupslut) and ((sortby=1) or (fourhills)) then { ja yhteispiste tulokset }
                 begin
                  jarjestys(1,1,NumPl);
                  updatestats(2);
                  lista(3);
                 end;

                updaterecords(sortby); { enn�tystauluja jos tarvis }

             */
            if (self.osakilpailu == self.cup_hills) || (self.s.ch.get() == 27) || (self.cupslut) {
                break;
            }
        }
        /*

        mcpisteet[15]:=0;  { ????? mik� 15? }

        wcup:=false;

        { testing alleolevia! }

        WriteProfiles;
        WriteRecords;
        MakeSendMe;

           */
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

    fn draw_full_main(&mut self) {
        self.i.draw_main_menu();

        // TODO
        //{$IFDEF REG}
        self.u.new_reg_text(REGNAME, REGNUMBER);
        // {$ELSE}
        //self.u.new_unreg_text();
        // {$ENDIF}
    }

    fn jump_menu(&mut self) {
        let mut index = 1;

        while index != 0 {
            self.u.main_menu_text(1, VERSION_FULL);

            index = self.u.make_menu(11, 97, 108, 12, 6, index, 8, 4, 0);

            match index {
                1 => {
                    self.cup_style = 0;
                    self.cup();
                }
                2 => {
                    self.cup_style = 1;
                    self.cup();
                }
                3 => {
                    self.cup_style = 2;
                    self.cup();
                }
                //{$IFDEF REG}
                4 => {
                    // teamcup;
                }
                //{$ENDIF}
                5 => {
                    // newkingofthehill;
                }
                6 => {
                    // training;
                }
                _ => {}
            }

            if index != 0 {
                self.draw_full_main();
            }
        }
    }

    pub fn main_menu(&mut self) {
        self.g.fill_box(0, 0, 319, 199, 0);

        let mut index = 1;

        // TODO
        // if (languagenumber=255) {
        //     WelcomeScreen(languagenumber);
        //     Replays(true, version, gdetail);
        // }

        while index != 0 {
            self.draw_full_main();
            self.u.main_menu_text(0, VERSION_FULL);
            index = self.u.make_menu(11, 97, 108, 12, 6, index, 8, 4, 2);

            match index {
                1 => {
                    self.jump_menu();
                }
                2 => {
                    // LoadNames(namenumber,jmaara,TeamLineup,false);
                    // profiles;
                    // if (pmaara=8) then jmaara:=2 else jmaara:=1;
                    // jnimet[NumTeams-1]:='Team 2';
                    // LoadNames(namenumber,jmaara,TeamLineup,true);
                }
                3 => {
                    //setupmenu;
                }
                4 => {
                    //showtops(0);
                }
                5 => {
                    // if (NumExtraHills > 0)
                    // {
                    //     showtops(1) }
                    //     else { showtops(2);
                    // }
                }
                6 => {
                    // replays(false, version, gdetail);
                }
                _ => {}
            }

            if index == 0 {
                // index = quitting(0);
            }

            self.g.draw_screen();
        }
    }

    fn check_param(&self) {
        // TODO
    }

    fn read_records(&mut self) {
        let mut l1: i32 = 0;
        let mut l2: i32 = 0;

        let mut f1 = BufReader::new(File::open("HISCORE.SKI").unwrap());

        read_line(&mut f1).unwrap(); //{ se varoitusrivi pois sielt� }

        let mut top = self.i.top.borrow_mut();
        for temp in 1..=41 {
            //{ 20 World Cup, 10 Team Cupia, 5 4hillsia ja 6 Kothia }
            let name = read_line(&mut f1).unwrap();
            l1 += valuestr(&name, temp) as i32;

            let mut str1 = read_line(&mut f1).unwrap();
            l2 += valuestr(&str1, temp) as i32;
            let pos = uncrypt(str1.clone(), temp) as u8;

            str1 = read_line(&mut f1).unwrap();
            l2 += valuestr(&str1, temp) as i32;
            let score = uncrypt(str1, temp);

            top.push(Hiscore {
                name,
                pos,
                score,
                time: Vec::new(),
            });
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

        for temp in 1..=41 {
            top[temp - 1].time = read_line(&mut f1).unwrap();
        }

        for temp in 1..=NUM_WC_HILLS {
            let name = self.u.hrname(temp);
            let len = self.u.hrlen(temp);
            let time = read_line(&mut f1).unwrap();
            self.u.set_hrinfo(temp, name, len, time);
        }

        self.u.vcode.set(parse_line(&mut f1).unwrap_or(1));

        if l1 != l3 || l2 != l4 || valuestr(&str1, 22) as i32 != l5 {
            println!("Error #21A: Something doesn't add up in the HISCORE.SKI file.");
            println!("Maybe you tried to edit it or something.  Please don't do it again.");
            // self.reset_hiscore(1);
            // self.reset_config();
            std::process::exit(1);
        }
    }

    fn read_config(&mut self) {
        /*
        var f2 : text;
            str1 : string;
            temp : integer;
        begin
        */
        let mut temp: i32 = 0;

        let mut f2 = BufReader::new(File::open("CONFIG.SKI").unwrap());

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

        self.namenumber = parse_line(&mut f2).unwrap();
        //{$IFNDEF REG}
        //self.namenumber=0;
        //{$ENDIF}

        self.setfile = b"TEMP".to_vec();
        let mut str1 = read_line(&mut f2).unwrap();
        if Path::new(from_utf8(&str1).unwrap())
            .with_extension(".SJC")
            .exists()
        {
            self.setfile = str1;
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

    pub fn alku(&mut self) {
        println!("Ported to SDL2 by Suomipelit (https://suomipelit.github.io)");
        println!();
        println!("New shortkeys:");
        println!(" * Alt+Enter          : Toggle fullscreen");
        println!(" * Alt+(Keypad) Plus  : Increase window size");
        println!(" * Alt+(Keypad) Minus : Decrease window size");
        println!(" * Alt+R              : Reset window if stretched");
        println!(" * Alt+A              : Toggle 4:3 aspect ratio");
        println!();
        println!("-------");
        println!();
        println!("SJ3 v{} by Ville Könönen 2011", from_utf8(VERSION).unwrap());

        print!("- Loading ANIM.SKI");
        self.g.load_anim("ANIM.SKI");

        self.eka = true;
        self.jcup = false;
        self.wcup = false;
        self.koth = false;
        self.cup_style = 0;

        self.check_param();

        print!(", HISCORE.SKI");
        self.read_records();

        print!(", CONFIG.SKI");
        self.read_config();

        print!(", PLAYERS.SKI");
        self.i.load_profiles();

        print!(", NAMES{}.SKI", self.namenumber);
        self.i
            .load_names(self.namenumber, self.jmaara, &mut self.teamlineup, true);

        print!(", LANGBASE.SKI");
        if self.languagenumber != 255 {
            // This is currently done in main because we want to be able to borrow the strings
            //self.l.load_language(self.languagenumber);
        }

        self.u.check_extra_hills(); //{ lukee ylim. SJH:t ja kirjoittaa MOREHILL.SKI:n }

        print!(", MOREHILL.SKI");
        self.u.load_hill_info(); //{ mnimet[],kri[] ja NumExtraHills kuntoon }

        print!(", Extra Hillrecords");
        if self.u.num_extra_hills > 0 {
            self.u.read_extras();
        }
        println!();

        self.m.alusta();
    }

    // originally in SJ3UNIT.PAS
    pub fn draw_lumi(&mut self, delx: i32, dely: i32, wind: i32, lmaara: u16, draw: bool) {
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
