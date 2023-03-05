use crate::graph::GraphModule;
use crate::help::{nsqrt, txt, txtp, HelpModule};
use crate::info::InfoModule;
use crate::lang::LangModule;
use crate::lumi::LumiModule;
use crate::maki::MakiModule;
use crate::pcx::{PcxModule, NUM_SKIS, NUM_SUITS};
use crate::regfree::{REGNAME, REGNUMBER};
use crate::rs_util::random;
use crate::sdlport::SDLPortModule;
use crate::table::{parru_anim, suksi_laskussa};
use crate::tuuli::TuuliModule;
use crate::unit::{loadgoal, Hill, Stat, Time, UnitModule, NUM_PL, NUM_TEAMS, NUM_WC_HILLS};
use std::fs::File;

pub struct SJ3Module<'g, 'h, 'i, 'l, 'm, 'p, 's, 'si, 't, 'u> {
    g: &'g GraphModule<'m, 's, 'si>,
    h: &'h HelpModule,
    i: &'i InfoModule<'g, 'l, 'm, 'p, 's, 'si>,
    l: &'l LangModule,
    lumi: LumiModule,
    m: &'m MakiModule,
    p: &'p PcxModule<'m, 's, 'si>,
    s: &'s SDLPortModule<'si>,
    tuuli: &'t TuuliModule<'g, 'h, 'm, 's, 'si>,
    u: &'u UnitModule<'g, 'h, 'l, 'm, 'p, 's, 'si>,

    version_full: &'static [u8],
    act_hill: Hill,
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
    set_file: Vec<u8>, //{ pakko olla global, muuten se unohtuu }
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

impl<'g, 'h, 'i, 'l, 'm, 'p, 's, 'si, 't, 'u> SJ3Module<'g, 'h, 'i, 'l, 'm, 'p, 's, 'si, 't, 'u> {
    pub fn new(
        g: &'g GraphModule<'m, 's, 'si>,
        h: &'h HelpModule,
        i: &'i InfoModule<'g, 'l, 'm, 'p, 's, 'si>,
        l: &'l LangModule,
        lumi: LumiModule,
        m: &'m MakiModule,
        p: &'p PcxModule<'m, 's, 'si>,
        s: &'s SDLPortModule<'si>,
        tuuli: &'t TuuliModule<'g, 'h, 'm, 's, 'si>,
        u: &'u UnitModule<'g, 'h, 'l, 'm, 'p, 's, 'si>,
    ) -> Self {
        SJ3Module {
            g,
            h,
            i,
            l,
            lumi,
            m,
            p,
            s,
            tuuli,
            u,
            version_full: b"3.13-remake0",
            act_hill: Hill::default(),
            jmaara: 0,
            nytmaki: (0),
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
            set_file: Vec::new(),
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
                        &self.i.jnimet[self.mcluett[temp] as usize] as &[u8],
                        b"$",
                        &str1,
                    ]
                    .concat();
                } else {
                    str1 = [
                        &self.i.nimet[self.mcluett[temp] as usize] as &[u8],
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
        self.g.e_write_font(308, 29, self.u.hrname(self.nytmaki));
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
        let mut tempb: u8;
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

        let mut deltah: [i32; 6] = [0; 6];

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

        let mut cjumper: bool;
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
        let mut rd: [[u8; 1001]; 5];

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
        self.h.ch.set(0);
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
                self.h.ch.set(27);
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
        if (self.eka) {
            self.p.aseta_paletti();
        }
        cjumper = true;
        draw = false;

        if pel > NUM_PL as i32 - self.i.pmaara as i32 {
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
            actprofile = self.i.profileorder[statsvictim as usize];
            self.p
                .load_suit(self.i.profile[actprofile as usize].suitcolor, 0);
            self.p
                .load_skis(self.i.profile[actprofile as usize].skicolor, 0);
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
            deltah[temp] = 0;
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

        namestr = self.i.nimet[pel as usize].clone();
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

        if draw && self.h.ch.get() != 27 {
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
                        &self.i.jnimet[team as usize],
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
                                        &txt(1 + self.kothmaara as i32 + self.i.pmaara as i32
                                            - self.mcpisteet[0]),
                                        b" ",
                                        self.l.lstr(8),
                                        b" ",
                                        &txt(self.kothmaara as i32 + self.i.pmaara as i32),
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
                                                self.i.nimet[top5[1] as usize].as_slice(),
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
                                        if (self.pisteet[top5[temp] as usize] > 0) {
                                            let mut str1 = txtp(self.pisteet[top5[temp] as usize]);

                                            if self.diff && temp > 1 {
                                                str1 = txtp(
                                                    self.pisteet[top5[temp] as usize]
                                                        - self.pisteet[top5[1] as usize],
                                                );
                                            }
                                            if self.jcup {
                                                str1 = [
                                                    &self.i.jnimet[top5[temp] as usize] as &[u8],
                                                    b"$",
                                                    &str1,
                                                ]
                                                .concat();
                                            } else {
                                                str1 = [
                                                    &self.i.nimet[top5[temp] as usize] as &[u8],
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

                self.h.ch.set(1);

                if self.s.key_pressed() {
                    let (ch, ch2) = self.s.wait_for_key_press();
                    self.h.ch.set(ch);
                    self.h.ch2.set(ch2);

                    if self.h.ch.get() == 0 && self.h.ch2.get() == 68 {
                        self.cupslut = true;
                        self.h.ch.set(27);
                    }

                    /*
                    if (upcase(ch)=lch(60,1)) then
                     begin
                      SetupMenu;
                      ch:=#1;
                      LoadSuit(Profile[actprofile].suitcolor,0);
                      LoadSkis(Profile[actprofile].skicolor,0);
                      mcliivi:=true;
                      if (mcluett[1]<>pel) or (treeni) or (koth) then mcliivi:=false;
                      if (not mcliivi) then SiirraLiiviPois;
                      Tuuli.AsetaPaikka(windplace);
                     end;

                    if (treeni) then
                     begin
                      if (ch='+') then begin inc(startgate); ch:=#1; end;
                      if (ch='-') then begin dec(startgate); ch:=#1; end;
                      if (startgate<1) then startgate:=1;
                      if (startgate>30) then startgate:=30;
                     end;
                    */
                }

                if self.h.ch.get() != 1 {
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

        if self.h.ch.get() != 27 && draw {
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
                                self.u.keyname(self.k[temp as usize]),
                            ]
                            .concat()),
                        );
                    }
                    self.g.font_color(247);
                }

                self.g.draw_screen();

                self.h.ch.set(0);
                self.h.ch2.set(0);

                if self.s.key_pressed() {
                    let (ch, ch2) = self.s.wait_for_key_press();
                    self.h.ch.set(ch);
                    self.h.ch2.set(ch2);
                    self.h.ch.set(self.h.ch.get().to_ascii_uppercase());
                    if self.h.ch.get() == 0 && self.h.ch2.get() == 68 {
                        self.cupslut = true;
                        self.h.ch.set(27);
                    }
                    if !cjumper && (self.h.kword() == self.k[2] || self.h.ch.get() == 13) {
                        out = true; //{ liikkeelle }
                    }
                    if self.h.ch.get() == 0
                        && self.h.ch2.get() == 63
                        && self.wcup
                        && self.cup_style == 1
                        && index == NUM_PL as i32
                        && self.kierros == 1
                        && self.osakilpailu == 1
                    {
                        self.tuuli.alusta(self.windplace);
                    }
                    if self.h.ch.get() == 27 {
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
                    if self.h.kword() == self.k[3] {
                        kulmalaskuri = 3000;
                    }
                    if self.h.kword() == self.k[4] {
                        kulmalaskuri = 4000;
                    }
                    if self.h.kword() == self.k[1] {
                        kulmalaskuri = 1000;
                    }
                    sx = 0;
                    sy = 0;
                }

                jumper_anim = parru_anim(&mut kulmalaskuri);

                if self.h.ch.get() == 27 && cjumper {
                    jumper_anim = 164; //{ emm� haluu tsiigaa }
                }

                if out && jumper_anim == 164 || laskuri > 700 {
                    break;
                }
            }
        }
        /*
          sx:=0;
          sy:=0;

          if (laskuri>700) then   { ei sitten kaveri l�htenytk��n }
            begin
             if (beeppi) then beep(2);
             muutalogo(4); { vihre� pois }
             asetapaletti;
             DrawAnim(x-Maki.X+60,y-Maki.Y-10,67); { liikennevalo }

             DrawAnim(3,150,65);

             fontcolor(240);
              writefont(12,160,namestr+' '+lstr(79));

             drawscreen;

             waitforkey;
             ch:=#27;
            end;

          Out:=False;

        {  temp:=67; }

          laskuri:=0;
          px:=0;

         if (ch=#27) and (draw) and (cjumper) then  { emm� haluukkaan tsiigata }
          begin
           draw:=false;
           ch:=#0;
          end;

        {  pl:=0.10; }

         if (cjumper) and (not draw) then { nopeutetaan v�h�n :) }
          begin
           px:=acthill.vxfinal;
           matka:=-45;
           for temp:=1 to 100 do Tuuli.Siirra; { veivataan my�s tuulta v�h�n }
          end;

          kulmalaskuri:=200;
        {  reflexlaskuri:=0; }
        {
         writefont(100,100,'ch: '+txt(byte(ch)));
         writefont(100,110,'m: '+txt(round(matka)));
         drawscreen;
         readkey;
        }

         if (ch<>#27) then { hyppy - main }
          repeat

           Tuuli.Hae;

        {   tuuli:=-10; }

           matka:=matka+(px*0.01);

            fx:=x;  { former x & y }
            fy:=y;

           x:=round(matka+qx);

           if (Ch<>#27) then Ch:=#0;  { en tied� muisteleeko se vanhoja }
           ch2:=#0;

           if (SDLPort.KeyPressed) then
              begin
               SDLPort.WaitForKeyPress(ch,ch2);
               ch:=upcase(ch);

        {       if (cjumper) then ch:=#1; }

               if (ch=#0) and (ch2=#68) then
                begin
                 cupslut:=true;
                 ch:=#27;
                 Out:=True;
                end;

               if (ch=#27) then
                begin

                 if (cjumper) then
                  begin
                   Draw:=False;
                   if (px<37) then px:=37;
                  end
                   else Out:=True;

                end;

                if (cjumper) then begin ch:=#0; ch2:=#0; end;

                if (ch='P') then SDLPort.WaitForKeyPress(ch,ch2);

              end;


        {       if (ch='q') then kulma1:=49;
                if (ch='w') then kulma1:=51; }

        {     if (matka=0) then beep(5); }

            if (matka>=0) then  { matka => 0 eli kun matka=0 ollaan jo lennossa }
             begin

              inc(laskuri);

              if (kulma1<kulmalaskuri) then kulmalaskuri:=kulma1;
        {      if (LentoAnim(kulma1) > 107) and (landing=0) then inc(reflexlaskuri); }

              if (cjumper) then
               begin
                 { automatic jumper angle differentiating }
                if (LentoAnim(kulma1) > 107) and (laskuri mod reflex = 0) then begin ch:=chr(hi(K[2])); ch2:=chr(lo(K[2])); end;
                if (LentoAnim(kulma1) < 107) and (laskuri mod reflex = 0) then begin ch:=chr(hi(K[3])); ch2:=chr(lo(K[3])); end;

                temp:=FindLanding(Makikulma(x));

                if (clanding=2) then temp:=round(temp*0.6); { ex (temp-temp div 4) }

        {          WriteFont(280,50,'T'+txt(temp));
                  WriteFont(280,60,'H'+txt(Height));
                  WriteFont(280,70,'D'+txt(DeltaH[0] + DeltaH[1] + DeltaH[2])); }

        {          WriteFont(280,70,'L'+txtp(hp));
                  WriteFont(280,80,'T-H'+txt(temp-height));
                  Writefont(250,90,'C'+txt(clanding)+' L'+txt(landing)); }

        {           DrawScreen;
                   ch:=readkey; }

                 { kumpualastulovarmistus! }
                if (landing=0) and (matka>3) and (height<4) then begin { beep(2); } landing:=2; end;

                if (Landing=0) and (DeltaH[0] + DeltaH[1] + DeltaH[2] > 1) and (Height < temp) then
                 begin

                  if (clanding=2) then landing:=2;

                  if (clanding=0) then { eka kertaa }
                   if (random(250) < temp*(temp div 10)) then clanding:=2
                                                         else landing:=1;
                 end;

               end; { if (cjumper) }

        {       if (upcase(ch)='T') then Landing:=1;
               if (upcase(ch)='R') then Landing:=2; }
               if (kword(ch,ch2)=K[4]) then Landing:=1;
               if (kword(ch,ch2)=K[5]) then Landing:=2;

               if (kword(ch,ch2)=K[3]) and (kulma1<=600) then kulma1:=kulma1+round(kulma1/4);
               if (kword(ch,ch2)=K[2]) and (Landing=0) and (kulma1>0) then kulma1:=kulma1-round(kulma1/5);

        {       if (ch='x') then pl:=0.1; }

                { ylirotaatio }
               if (kulma1<50) then begin pl:=pl+0.0001-((kulma1-50)/18000); {beep(1);} end;


        { orig. if (tuuli>0) then pl:=pl-((1-(kulma1/900))/1500)+(sqrt(sqrt(2*tuuli)))/52400
                 else pl:=pl-((1-(kulma1/900))/1500)-(sqrt(sqrt(-2*tuuli)))/52400; }

              if (tuuli.value>0) then pl:=pl-((1-(kulma1/900))/1875)+(sqrt(sqrt(2*tuuli.value)))/65500
                           else pl:=pl-((1-(kulma1/900))/1875)-(sqrt(sqrt(-2*tuuli.value)))/65500;

        {      px:=px-((kulma1/900)/19)+((sqrt(4*tuuli.value+245)-16)/360); } { "alkup." }
        {      px:=px-((kulma1/900)/24)+((sqrt(4*tuuli.value+245)-16)/455); }
              px:=px-((kulma1/900)/20)+((nsqrt(4*tuuli.value+245)-16)/400);

              t:=t+0.01;

               { Satunnaispuuska }
               if (random(30000)<tuuli.windy+10+tuuli.voim) then
                begin

                 laskuri:=0; { reflexi� varten }

                 if (random(2)=1) then tyylip[1]:=tyylip[1]-5;

                 temp:=random(15);  { �ij�n kulman muutos... S��D� T�T�! }
                 dec(temp,6);

                 Kulma1:=kulma1+temp;

                 if (temp>0) then
                  begin { puuska tuo sukset yl�s }
                   ssuunta:=3;
                   pl:=pl-random(tuuli.voim+50)/15000;
                  end;

                 if (temp<0) then
                  begin { puuska vie suksia alas }
                   ssuunta:=6;
        {           pl:=pl-random(tvoim+20)/6000+(tvoim/12000); } { vanha }
                   pl:=pl+random(tuuli.voim+50)/15000;
                  end;

                end;

              if (pl<0.105) then pl:=0.105; { leijuntaesto :) }

              kor:=kor+(t*t*pl)-((py-8)/100);

        {       delay(600); }

             if (ssuunta>0) then  { SUKSIEN HEILUNTA - ponnistusfibat ja puuskat }
              begin

               case ssuunta of

               2 : begin { ponn my�h�ss�, k menossa alas }
                    if (kulmas<0) then inc(kulmas,2); { 4, 3 }
                    if (kulmas>0) then kulmas:=0;
                   end;

               1 : begin { ponn my�h�ss�, k tulossa yl�s }

                    if (kulmas=0) then
                     begin { first strike }
                      kulmas:=-51-(16-ponnistus)*6;
                      if (kulmas<-105) then kulmas:=-105;

                     end else dec(kulmas,4);

                    if (kulmas < (ponnistus-16)*14) then ssuunta:=2;

                   end;

               3 : begin { puuska, k saa iskua (ja tulee alas) }
                    kulmas:=kulmas-random(50)-30;
                    ssuunta:=2;
                   end;


               5 : begin { ponn ajoissa, k tulossa yl�s }
                    if (kulmas>0) then dec(kulmas,1);
                    if (kulmas<0) then kulmas:=0;
                   end;

               4 : begin { ponn ajoissa, k menee alas }

                    if (kulmas=0) then
                     begin { first strike }
                      kulmas:=70+(ponnistus-16)*6;
                      if (kulmas>130) then kulmas:=130; { ei nyt ihan �lyt�n }

                     end else inc(kulmas,3);

                    if (kulmas > (ponnistus-16)*14) then ssuunta:=5; { riitt�� }

                   end;

               6 : begin { puuska, k saa iskua (ja tulee yl�s) }
                    kulmas:=kulmas+random(50)+30;
                    ssuunta:=5;
                   end;

               end; { case }

               if (kulmas=0) then ssuunta:=0;
              end;


              if (Landing>0) then  { alastulolis�ykset }
               if (kulma1<600) then
                begin
                 kulma1:=kulma1+9+(landing-1)*5;

                 if (pl<1) then pl:=pl+0.003;
                end;

              if (OK) then  { alkukulma... RUN ONCE (OK) }
               begin
                kulma1:=158; OK:=False;
                {  kulma1:=52; }

                if (ponnistus<16) then ssuunta:=1; { ponn liian my�h��n - k�rjet yl�s}
                if (ponnistus>16) then ssuunta:=4; { ponn liian aikaisin - k�rjet alas }
                if (ponnistus=0) then ssuunta:=0; { ei sitten ponnistanut ollenkaan }
               end;

              JumperAnim:=LentoAnim(kulma1);

              if (Ponnphase<25) then JumperAnim:=PonnAnim(ponnphase);

              temp:=Height; { vanha korkeus }
              Height:=Profiili(x)-round(kor);

              if (Height<0) then Height:=0;

              DeltaH[laskuri mod 3]:=temp-Height;

              if (Height<6) { and (kulmas>-10) } and (matka>20) then
               begin  { j�nnitt�v� m�en ennakointi! }
        (*        ssuunta:=2; { suunta varmasti alas - lopettaa nollaan } *)
                if (kulmas=0) then ssuunta:=0; { meik� m��r�� varmasti asennon }
                SkiAnim:=SuksiLaskussa(MakiKulma(x) div (Height+1));
               end
                else SkiAnim:=SuksiLennossa(kulmas);

              if (Height=0) then Out:=true; { the eagle has landed }

             End
            else

             begin  { matka < 0 }

               inc(laskuri);

                { mies ponnistaa jo... }
               If (Ponnistus>0) then inc(ponnistus);

                { start ponnistus! }
               If (kword(ch,ch2)=K[1]) and (matka>-40) and (ponnistus=0) then
                begin
                 inc(Ponnistus);
                 { SoundOn[1]:=StartSound(Sound[1],1,False);}
                end;

               { Automaattiponnistus }
                if (cjumper) and (matka>-(skill*px*0.01)) and (ponnistus=0) then inc(ponnistus);

              SkiAnim:=SuksiLaskussa(MakiKulma(x));

              JumperAnim:=LaskuAsento(SkiAnim);

              if (draw) and (laskuri<28) then
               begin
                JumperAnim:=165+Laskuri div 7;
                px:=37;
                if (laskuri<14) then px:=0;
               end;

              if (ponnistus>0) then JumperAnim:=PonnAnim(ponnphase);

                 { Pudotetaan �ij� }

              kor:=Profiili(x);

              px:=px*pxk;  { Hanaa... }

              if (px>maxspeed) then begin px:=maxspeed; {beep(2);} end;

              if (ponnistus>0) then
               begin
                 { ponnistuslis�ys }
                 px:=px+0.21;
                 py:=py+1.21;
                 { pl:=pl-(5*0.01); }
                 kulma1:=kulma1+12;

                if (ponnistus>16) then { Ponnistus liian aikaisin }
                 begin

                  if (ponnistus=17) then
                   begin
                    pl:=pl+0.023;
                    if (not cjumper) and (beeppi) then beep(1);
                   end;

                  pl:=pl+0.013;
                  py:=py-1;


                  if (kierros>0) then stats[statsvictim,osakilpailu].Reason[kierros]:=1;

                  kulma1:=158;  { onko tarpeellinen?? }

                 end;
               end;


             end;  { end matkariippuvaiset }

           x:=round(matka+qx);
           y:=round(kor);

         if (draw) then
          begin

           DeltaX:=Maki.X;
           DeltaY:=Maki.Y;

           if (x>=160) and (x<864) then inc(sx,x-fx);
           if (y>=100) and (y<412) then inc(sy,y-fy);

           Maki.X:=sx;
           Maki.Y:=sy;

           if (Maki.X>704) then Maki.X:=704;
           if (Maki.Y>312) then Maki.Y:=312;

           Maki.Tulosta;

           DrawLumi(DeltaX-Maki.X,DeltaY-Maki.Y,Tuuli.Value,LMaara,true);

           if (goals) and (goalx>0) then DrawAnim(goalx-Maki.X,goaly-Maki.Y,66); { tavoitekeppi }
           if (wrx>0) then DrawAnim(wrx-Maki.X,wry-Maki.Y,68); { m�kienkkakeppi }

           DrawAnim(x-Maki.X,y-Maki.Y-2,JumperAnim);
           DrawAnim(x-Maki.X,y-Maki.Y-1,SkiAnim);

           if (cjumper) then writefont(x-Maki.X,y-Maki.Y-20,'C');

        {   PutPixel(KeulaX-Maki.X,Profiili(KeulaX)-Maki.Y,15); }

        {   tempb:=SuksiAnim(x); }

        {   Lumi.Update(Video,128,(DeltaX-Maki.X)*2,(DeltaY-Maki.Y)*2); }

        {   Writefont(x-Maki.X+20,y-Maki.Y-20,txt(round(px))); }
        {   Writefont(x-Maki.X,y-Maki.Y-25,txt(round(pl*100))); }
        {   Writefont(x-Maki.X-20,y-Maki.Y-20,txt(round(py))); }

        {    Writefont(x-maki.x,y-maki.y-25,txt(deltah[0]));
            Writefont(x-maki.x,y-maki.y-19,txt(deltah[1]));
            Writefont(x-maki.x,y-maki.y-13,txt(deltah[2])); }

        {   Writefont(x-Maki.X-20,y-Maki.Y-20,txt(round(kulma1))); }

        {   Writefont(x-Maki.X+20,y-Maki.Y-20,txt(round(kulmas)));
            Writefont(x-Maki.X,y-Maki.Y-20,txt(ssuunta)); }
        {   Writefont(x-Maki.X,y-Maki.Y-20,txt(makikulma(x))); }
        {   Writefont(x-Maki.X,y-Maki.Y-20,txtp(hp)); }

        {   if (ponnistus>0) then
             begin
              Writefont(x-Maki.X,y-Maki.Y-40,txt(ponnistus));
              Writefont(x-Maki.X+30,y-Maki.Y-40,txt(kulmas));
             end;
        }

            if (windplace>10) then Tuuli.Tuo(x-Maki.X,y-Maki.Y);

            Tuuli.Piirra;

        {    if (ex) then balk(0); }

             RD[0,RTurns]:=byte(128+x-fx);
             RD[1,RTurns]:=byte(128+y-fy);
             RD[2,RTurns]:=JumperAnim;
             RD[3,RTurns]:=SkiAnim;
             RD[4,RTurns]:=byte(Tuuli.value+128);
             if (matka<0) then RFlstart:=Rturns else RFlstop:=Rturns;

             inc(RTurns);

        {    if (ex) then balk(1); }

        {   if (matka>0) then writefont(10,10,1,'!'); }

            DrawScreen;

        {    if (matka<0) then delay(500); }

        {  if (ponnistus>0) then ch:=readkey;

           if (ch=#13) then out:=true; }

        {  if (matka>-5) then ch:=readkey; }

        {  if (height<8) and (matka>=0) then ch:=readkey; }

          end; { if draw }

        (*
           if (x>1015) then begin draw:=false; out:=true; end; { lent�� ulos ruudusta u nou }
        *)

          until (Out);   { ***  LENTO LOPPUU  *** }

           kkor:=kor-KeulaY;
           hp:=round(nsqrt((matka*matka)+(kkor*kkor))*acthill.pk*0.5)*5;

           if (ch=#27) then { painoi ESCi� }
            begin
             hp:=0;
             score:=0;
             landing:=1;
             if (kierros >= 0) then
              begin
               CStats[kierros,pel]:=hp;
               stats[statsvictim,osakilpailu].RoundPts[kierros]:=score;
               stats[statsvictim,osakilpailu].RoundLen[kierros]:=hp;
              end
            end else
             begin
              inc(Profile[actprofile].totaljumps);

              grade:=0;
              if (kr<>0) then grade:=round(hp/kr)*10;

              temp:=MakiKulma(x);
              height:=round((temp*1.34) + (kulma1/10));

              riski:=jumprisk(temp);
               if (hp<20/3*kr) then riski:=1; { lyhyiden hyppyjen riski? }

              if (height<63) then riski:=round(riski*(1+((63-height)*0.075)));

        {      laskuri:=round(2.5*sqr(sqr(sqr(sqr(sqr(((hp/10)/kr))))))+((hp/10)/kr*2)-1); }

        {     riski:=10 * riski; }

              if (landing=0) or (height < 56) then
               begin
                if (kierros>0) then stats[statsvictim,osakilpailu].Reason[kierros]:=2;
                kupat:=2;
                if (landing=0) then kupat:=1;
               end;

              if (landing=1) then { telemark-juttuja }
               begin
                inc(riski,2*riski);
        {        inc(laskuri,3*laskuri); }

                if (height<60) then dec(tyylip[1],5);
                if (height<64) then dec(tyylip[1],5);
               end;


        {      writefont(270,10,txt(round(makikulma(x)))); }

        {     writefont(270,20,txt(round(kulma1/10)));
              writefont(270,30,txt(round((makikulma(x)*1.34) + (kulma1/10)))); }

        {      writefont(260,40,'K'+txt(round(temp)));
              writefont(260,20,'OLD-R'+txtp(laskuri)); }
        {       writefont(260,30,'NEW-R'+txtp(riski)); }

        {      writefont(270,30,'L'+txt(kulmalaskuri)); }
        {      writefont(260,40,'H'+txt(height)); }

        {      drawscreen;
               readkey; }

              if (random(1000)<riski) then { liian longa hyppy tai vaan kehno sk�g� }
               begin
                if (kierros>0) then stats[statsvictim,osakilpailu].Reason[kierros]:=3;
                kupat:=3;
               end;

              for temp:=1 to round((kr+(kr/20)-(hp/10))/6) do
               tyylip[1]:=tyylip[1]-5;

              if (kupat>0) then tyylip[1]:=tyylip[1]-100
               else
                if (landing=2) then dec(tyylip[1],15+random(2)*5); { tasajalka tuomarirokotus }

              for temp:=2 to 5 do
               begin
                temp2:=random(4);
                tyylip[temp]:=tyylip[1];
                tyylip[temp]:=tyylip[temp]-(temp2-1)*5; { ennen temp2-2 }
               end;

              for temp:=1 to 5 do
               begin
                if (tyylip[temp]>200) then tyylip[temp]:=200;
                if (tyylip[temp]<0) then tyylip[temp]:=0;
                if (tyylip[temp]>tyylip[7]) then tyylip[7]:=tyylip[temp];
                if (tyylip[temp]<tyylip[6]) then tyylip[6]:=tyylip[temp];
               end;

              if (kupat>0) then inj[pel]:=injured; { LOUKKAANTUMINEN!!! }

        {    if (kupat=1) then inj[pel]:=1;
            if (kupat=2) then inj[pel]:=3;
             if (kupat=3) then inj[pel]:=6; }

              score:=0;

        {   splitscreen(60);
           waitraster; }

        {if (ch<>#27) then}

              for temp:=1 to 5 do inc(score,tyylip[temp]);

              dec(score,tyylip[6]);  { pienin ja suurin pois }
              dec(score,tyylip[7]);

              if (kr<>0) then { vanha tyyli }
               inc(score,round(((hp/10)-(kr*2/3))*(180/kr)*10));  { pituuspisteet }

        (*
              if (kr<>0) then { uusi fis:in mukainen }
               inc(score,round((hp-(kr*10))*(lengthpoint(kr)/10)+600));  { pituuspisteet }

              if (kr>=160) then inc(score,600); { lentom�ist� saa isot pisteet }
        *)

              if (score>paras) then { harjoituskamaa }
               begin
                paras:=score;
               end;

              if (kierros=-10) then pisteet[NumPl+1]:=score;

              if (not jcup) and (kierros>=0) then
               begin
                inc(pisteet[pel],score); { Oma pelaajan pisteet }
                CStats[kierros,pel]:=hp;

        (*        if (ex) then Cstats[kierros,pel]:=height*10+landing; *)

        (*        Cstats[kierros,pel]:=ponnistus*10; { yl�svaan } *)
               end;

              if (jcup) then inc(pisteet[team],score);

              if (kierros>=0) then
               begin
                stats[statsvictim,osakilpailu].RoundPts[kierros]:=score;
                stats[statsvictim,osakilpailu].RoundLen[kierros]:=hp;
               end;

              for temp:=1 to 5 do inc(tyylip[temp],10000);

              hillrecord:=false;

              if (wcup) and (cupstyle=0) and (hp > Profile[actprofile].bestwcjump) then
               begin      { vain real world cup }
                Profile[actprofile].bestwcjump := hp;
                Profile[actprofile].bestwchill := nytmaki;
               end;

              if (hp > Profile[actprofile].bestjump) then
               begin
                Profile[actprofile].bestjump := hp;
                Profile[actprofile].besthill := nytmaki;
                Profile[actprofile].besthillfile:='HILLBASE';
                if (nytmaki>NumWCHills) then
                 Profile[actprofile].besthillfile:=hillfile(nytmaki-NumWCHills);
               end;

              if (not koth) and (kupat=0) and (not treeni) and (hp > HRLen(nytmaki)) then
               hillrecord:=true;

              if (cjumper) and (not comphrs) then hillrecord:=false; { ei saa }

              if (hillrecord) then
               begin
                {
                dwritefont(200,24,'NEW HILLRECORD!!!');
                dwritefont(200,32,'NEW: '+txtp(hp)+'�');
                writefont(270,32,7,'('+txtp(MEPituus[nytmaki])+'�)'); }

                if (cjumper) then namestr:=namestr+'�';

                SetHRinfo(nytmaki,namestr,hp,dayandtime(Today,Now));

        {        HR[nytmaki].len:=hp; }
        {        HR[nytmaki].name:=namestr; }
        {        HR[nytmaki].time:=dayandtime(Today,Now); }

                if (kierros>0) then stats[statsvictim,osakilpailu].Reason[kierros]:=5;

                ThisIsAHillRecord:=(kierros*1000)+pel;

               end;
        {
              fillbox(100,100,300,180,1);
              writefont(110,110,1,'PONN '+txt(ponnistus));
              writefont(110,120,1,'HP '+txtp(hp));
              Maki.Paivitaruutu;
              putsaa;
              ch:=readkey;       }

              laskuri:=0;
              startanim:=100; { start "nousepa yl�s"-kuvio }

              OK:=boolean(random(2)); { if OK then sukset j�� kiinni }

              if (landing=2) then dec(startanim,50);

              umatka:=matka; { ukon koordinaatit, jos sukset l�htee alta... }
              ukor:=kor;
              upx:=px;

            {  if (gdetail=0) or (not cjumper) then} FontColor(247);

              Out:=False;

        {     kupat:=3; }

              if (kupat>0) then grade:=kupat;

             if (draw) then
              repeat                 { ***  LASKU  *** }

               Tuuli.Hae;

               inc(laskuri);

               matka:=matka+(px*0.008);
               umatka:=umatka+(upx*0.008);

               if (OK) then matka:=umatka;

               fx:=x;
               fy:=y;

               x:=round(matka+qx);
               ux:=round(umatka+qx);

               kor:=Profiili(x);
               ukor:=Profiili(ux);

              { if (x<1010) then}

               SkiAnim:=SuksiLaskussa(MakiKulma(x));

               JumperAnim:=LaskuAnim(SkiAnim, landing); { t�t� voi my�s sitten muuttaa jos haluaa }

               if (laskuri<7) and (landing>0) then JumperAnim:=113+landing; { esilaskeutuminen }

               if (kupat>0) then
                begin

                 if (laskuri>50) and (upx>0) then upx:=upx-0.8;
                 if (upx<0) then
                  begin
                   upx:=0;
                   if (OK) then Out:=True;
                  end;

                 case kupat of
                 1,2 : begin { ei alastuloa, �ij� suoraan turvalleen }

                        temp:=2-(kulma1 div 80);
                        if (temp<0) then temp:=0;

                        tempb:=142+laskuri div 10+temp;

                        if (kupat=2) then tempb:=142+(laskuri-6) div 10+temp;

                        if (tempb>145) then
                         begin
                          tempb:=144;
                          case (SuksiLaskussa(MakiKulma(ux))-71) of
                            4   : tempb:=145;
                            5   : tempb:=146;
                            6   : tempb:=147;
                          7..12 : tempb:=148;
                          end; { case }
                         end;

                        if not ((kupat=2) and (laskuri<6)) then JumperAnim:=tempb;

                       end;

                 3 : if (laskuri>14) then { liian pitk� - esilaskeut. & v�h�n norm. laskua }
                      begin

                       tempb:=151+(laskuri-14) div 10;

                       if (tempb>155) then
                        begin
                         tempb:=163;
                         case (SuksiLaskussa(Makikulma(ux))-71) of
                          3..4  : tempb:=162;
                          5..6  : tempb:=161;
                          7..12 : tempb:=160;
                         end; { case }
                        end else if (landing=2) then inc(tempb,5);

                        JumperAnim:=tempb;
                       end;

                 end; { case }

                end else
                 begin { ei kaatuna }

                  if (laskuri>startanim) then
                   begin
                    temp:=(laskuri-startanim) div 12;
                    if (temp>6) then temp:=6;

                    case temp of
                    0 : tempb:=122+(landing*6);
                    1 : tempb:=123+(landing*6);
                    2 : tempb:=136;
                    3..6 : begin
                            tempb:=136; { peruslasku }
                            case grade of
                            0..75  : begin       { h�pe� v�h�n }
                                      tempb:=137;
                                      { if (temp>4) and (grade<50) then tempb:=138; }
                                     end;

                            105..200 : begin       { yeah! }
                                        tempb:=139;
                                        if (temp>3) then
                                         if (grade>114) then tempb:=141 else tempb:=140;

        {                                if (temp>4) and (grade>114) then tempb:=141; }
                                       end;

                           end; { case grade }

                          end;
                    end; { case }

                    JumperAnim:=tempb;

                   end;

                 end;

                Ch:=#0;

                if (SDLPort.KeyPressed) then
                 begin
                  SDLPort.WaitForKeyPress(ch,ch2);
                  ch:=upcase(ch);
                 end;
                if (ch=#0) and (Ch2=#68) then begin cupslut:=true; ch:=#27; end;
                if (Ch=#27) or (Ch=#13) then Out:=True;
                if (Ch='P') then SDLPort.WaitForKeyPress(ch,ch2);

                DeltaX:=Maki.X;
                DeltaY:=Maki.Y;

                x:=round(matka+qx);
                y:=round(kor);

                if (x>=160) and (x<864) then inc(sx,x-fx);
                if (y>=100) and (y<412) then inc(sy,y-fy);

                Maki.X:=sx;
                Maki.Y:=sy;

                if (Maki.X>704) then Maki.X:=704;
                if (Maki.Y>312) then Maki.Y:=312;

                Maki.Tulosta;

        {       PutPixel(KeulaX-Maki.X,Profiili(KeulaX)-Maki.Y,15); }
        {       tempb:=SuksiAnim(x); }
        {       Writefont(x-maki.x,y-maki.y-15,txt(SkiAnim-71)); }
        {       Writefont(x-Maki.X,y-Maki.Y-20,txt(makikulma(x))); }
        {       Writefont(x-Maki.X+15,y-Maki.Y-15,txt(kulma1)); }
        {       DrawAnim(x-Maki.X,y-Maki.Y-2,JumperAnim); }

        {   Writefont(10,192,'LJA: '+txt(JumperAnim));
           Writefont(60,192,'LSA: '+txt(SkiAnim)); }

        {   if (JumperAnim>NumofAnims) or (SkiAnim>NumofAnims) or
              (JumperAnim<1) or (SkiAnim<1) then
            begin
             beep(1);
             writefont(100,80,'LASKU!');
             writefont(100,100,'JAnim: '+txt(JumperAnim));
             writefont(100,120,'SAnim: '+txt(SkiAnim));
             maki.paivitaruutu;
             ch2:=readkey;
            end;
        }
                DrawLumi(DeltaX-Maki.X,DeltaY-Maki.Y,Tuuli.Value,LMaara,true);

                if (goals) and (goalx>0) then DrawAnim(goalx-Maki.X,goaly-Maki.Y,66); { tavoitekeppi }
                if (wrx>0) then DrawAnim(wrx-Maki.X,wry-Maki.Y,68); { m�kienkkakeppi }

                DrawAnim(ux-Maki.X,round(ukor)-Maki.Y-2,JumperAnim);
                DrawAnim(x-Maki.X,y-Maki.Y-1,SkiAnim);

        {       Lumi.Update(Video,128,(DeltaX-Maki.X)*2,(DeltaY-Maki.Y)*2); }

        (*      drawanim(297,5,59);  { sapluuna } *)
        {       drawanim(300,60,59); }

        {       drawanim(298,9,59); }

                if (cjumper) then writefont(x-Maki.X,y-Maki.Y-20,'C');

                if (x>1050) then Out:=True;

                DrawAnim(227,2,64); { hillrec sapluun }

                ewritefont(308,9,namestr);

        {       temp2:=311; }
                temp2:=308;
        {       if (kierros=2) then temp2:=280; }

                ewritefont(temp2,33,txtp(hp)+'�');

                if (hillrecord) and (not Out) then
                 begin
                  if (laskuri mod 30 < 15) then writefont(260,33,'HR!');
        {         writefont(279+random(3),71+random(3),txtp(hp)+'�'); }

                  if (random(2)=0) then ewritefont(temp2-1+random(3),32+random(3),txtp(hp)+'�');

        {         writefont(280,72,txtp(hp)+'�'); }
                 end;

                { tyylipisteet ruutuun }

                temp:=random(5)+1;
                 if (random(20)=1) and (tyylip[temp]>9999) then dec(tyylip[temp],10000);

                for temp:=1 to 5 do
                 if (tyylip[temp]<9999) then
                  ewritefont(308-(temp-1)*24,21,txtp(tyylip[temp]));
          {       ewritefont(304,25+(temp-1)*11,txtp(tyylip[temp])); }

        {       ewritefont(304,8,namestr); }

        {        if (ex) then balk(0); }

                 RD[0,RTurns]:=byte(128+x-fx);
                 RD[1,RTurns]:=byte(128+y-fy);
                 RD[2,RTurns]:=JumperAnim;
                 RD[3,RTurns]:=SkiAnim;
                 RD[4,RTurns]:=byte(Tuuli.value+128);

                 inc(RTurns);

        {        if (ex) then balk(1); }

                DrawScreen;

        {       delay(500); }

               until (Out);  { LASKU LOPPUU }

        {  StopSound(5);
          StopSound(4); }

               fx:=0; { k�ytet��n effektiivisten tyylipisteiden l�yt�miseen :) }
               fy:=0;

               for temp:=1 to 5 do     { tyylipisteet n�ytt��n ja poisj��v�t tummaksi }
                begin

                 if (tyylip[temp]>9999) then dec(tyylip[temp],10000);

                 if (draw) and ((gdetail=0) or (not cjumper)) then fontcolor(247);

                 if (tyylip[temp]=tyylip[6]) and (fx=0) then
                  begin if (draw) and ((gdetail=0) or (not cjumper)) then fontcolor(252); inc(fx); end;

                 if (tyylip[temp]=tyylip[7]) and (fy=0) then
                  begin if (draw) and ((gdetail=0) or (not cjumper)) then fontcolor(252); inc(fy); end;

        {        ewritefont(304,25+(temp-1)*11,txtp(tyylip[temp])); }
                 if (draw) then ewritefont(308-(temp-1)*24,21,txtp(tyylip[temp]));

                end;

               if (jcup) then
                begin
                 if (draw) and ((gdetail=0) or (not cjumper)) then FontColor(241);

                 if (draw) then EWritefont(302,14,jnimet[team]);
        {        EWritefont(302-FontLen(namestr),9,jnimet[16-pel]); }
                end;

               if (draw) and ((gdetail=0) or (not cjumper)) then FontColor(240);

               if (draw) then EWriteFont(308,9,namestr)
                         else
                          begin
                           EWriteFont(296,9,lstr(57));
                           EWriteFont(308,9,txt(index)); { computers jumping... }
                          end;

        {
               WriteFont(160,9,'P'+txt(ponnistus));
               WriteFont(160,21,'R'+txtp(riski));
               Writefont(160,45,'RL'+txt(reflexlaskuri div 5)); }

               if (draw) and ((gdetail=0) or (not cjumper)) then FontColor(247);

        {       drawscreen;
               readkey; }

               fx:=0; { seuraavassa: osallistujat }
               fy:=0; { - " - : sijoitus }

               temp2:=0;

               if (not koth) and (kierros>=0) then  { komea "monesko olen nyt?"-laskuri! }
                begin

                 fx:=NumPl;
                 if (jcup) then fx:=NumTeams;

                 for temp:=1 to fx do
                  begin
                   if (jcup) then temp2:=pisteet[team] { omat pisteet }
                             else temp2:=pisteet[pel];

                   if (temp2 >= pisteet[temp]) then inc(fy);
                  end;

                 if (draw) then ewritefont(255,45,'($'+txt(fx-fy+1)+'.)');

                end;

               fx:=308;

               if (draw) and (hillrecord) then
                begin
                 writefont(260,33,'HR!');
                 fontcolor(246);
                end;

               if (draw) then ewritefont(fx,33,txtp(hp)+'�');

               if (draw) and ((gdetail=0) or (not cjumper)) then fontcolor(246);

               temp2:=score;

               if (jcup) then temp2:=pisteet[team];
               if (wcup) or (koth) then temp2:=pisteet[pel];

               if (wcup) and (kierros<0) then temp2:=score;

               if (draw) then
                begin
                 ewritefont(308,45,txtp(temp2)); { score }
                 if (gdetail=0) or (not cjumper) then fontcolor(241);

                 if (draw) and (kierros=2) and (wcup) then
                  begin
                   ewritefont(255,33,txtp(CStats[1,pel])+'�');
                   ewritefont(311,55,'('+txtp(score)+')');
                  end;

                 if (jcup) and (index*kierros>1) then
                  begin
                   ewritefont(311,55,'('+txtp(score)+')');
                  end;

                 if (inj[pel]>0) and (wcup) then
                  begin
                   fontcolor(239);
        {        str1:='LEGS.';
                 if (inj[pel]=1) then str1:='LEG.';
                 writefont(12,176,'WILL MISS NEXT '+txt(inj[pel])+str1); }

                   str1:=txt(inj[pel])+' '+lstr(76);

                   case inj[pel] of
                   1 : str1:=lstr(77);
                   2 : str1:=lstr(78);
                   end;

                 { ewritefont(212,33,'INJ-'+txt(inj[pel]-1)); }
                   ewritefont(311,64,lstr(75)+' '+str1);

                  end;
                 end;

               if (not draw) and (gdetail=0) then
                begin
        {         tuuli.value:=tuulisafe; }
                 DrawLumi(DeltaX-Maki.X,DeltaY-Maki.Y,tuuli.value,LMaara,false); { pakko piirt�� }
                end;

               if (draw) and (not cjumper) and (grade>0) and
                  (Profile[actprofile].cstyle>0) then DoCoachCorner(height,kulmalaskuri,grade,ponnistus,Profile[actprofile].cstyle);

        {      if (landing=2) then writefont(160,45,'TASA');
               WriteFont(160,33,'K'+txt(height)); }

               dec(RTurns); { niit� on yksi liikaa }

        {       Writefont(100,100,'T'+txt(RTurns)); }
        {$IFDEF REG}
               if (not cjumper) and (draw) then EWritefont(308,73,lstr(298));
        {$ENDIF}

               DrawScreen;

        {       delay(100); }

              if (draw) then
               begin
                Putsaa;
                cupslut:=WaitForKey2;

               end
                else
                 if (SDLPort.KeyPressed) then
                  begin
                   SDLPort.WaitForKeyPress(ch,ch2);
                   if (ch=#0) and (ch2=#68) then cupslut:=true;
                  end;
              end;

        {  if (ch=#68) then begin cupslut:=true; ch:=#27; end; }

          Maki.Lopeta;

        {  AsetaMoodi($3); }

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

        { if (ex) then
          begin

           fontcolor(240);
           writefont(60,10,'Normaali lopetus');

           writefont(60,20,'KeulaX: '+txt(keulaX));
           writefont(60,30,'KeulaY: '+txt(profiili(keulaX)));
           writefont(60,40,'Ponn: '+txt(ponnistus));

           drawscreen;
           readkey;

          end;
        }

        end;

                 */
    }

    fn jarjestys(&mut self, fromarray: u8, toarray: u8, num: u8) {
        // { from: 0 - MCpist, 1 - fourpts, 2 - pisteet }
        // { to: 0 - mcluett, 1 - luett }
        let mut score1: i32;
        let mut score2: i32;
        let mut templuett: [u8; NUM_PL + 2] = [0; NUM_PL + 2];

        for t1 in 0..=NUM_PL {
            //{ kaikki nolliin }
            templuett[t1] = 0;
            self.sija[t1] = 0;
        }
        for t1 in 0..num {
            //{ Jokainen hypp��j� k�yd��n l�pi }
            let mut t2 = 0;

            while t2 < num as i32 {
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
                    for t3 in (t2..num as i32).rev() {
                        templuett[t3 as usize] = templuett[t3 as usize - 1];
                    }

                    templuett[t2 as usize] = t1;
                    t2 = 100;
                } else if t2 == num as i32 {
                    templuett[t1 as usize] = t1;
                }

                t2 += 1;
            }
        }

        for t1 in 0..num {
            //{ k��nteistaulukko eli sija[pelaaja]? }
            self.sija[templuett[t1 as usize] as usize] = t1;
        }

        for t1 in 1..num {
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

            let mut sija = self.sija;
            if score1 == score2 {
                sija[templuett[t1 as usize] as usize] = sija[templuett[t1 as usize - 1] as usize];
            }
        }

        match toarray {
            0 => {
                for t1 in 0..num {
                    self.mcluett[t1 as usize] = templuett[t1 as usize];
                }
            }
            1 => {
                for t1 in 0..num {
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
                let mut hill_order = self.hill_order;
                for temp in 0..=40 {
                    hill_order[temp as usize] = temp;
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
                let mut hill_order = self.hill_order;
                for temp in 0..=4 {
                    hill_order[temp as usize] = temp + 8;
                }
                self.cup_hills = 4;
                sortby = 1;
            }
            _ => {}
        }

        if self.cup_hills == 0 {
            self.cupslut = true;
        }

        self.h.ch.set(1);

        while (self.osakilpailu != self.cup_hills) && (self.h.ch.get() != 27) && (!self.cupslut) {
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
            self.u.main_menu_text(1, self.version_full);

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
            self.u.main_menu_text(0, self.version_full);
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
