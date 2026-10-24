use crate::graph::GraphModule;
use crate::help::{nsqrt, round, txt, txtp, upstr};
use crate::info::InfoModule;
use crate::lang::LangModule;
use crate::lumi::LumiModule;
use crate::maki::MakiModule;
use crate::pcx::PcxModule;
use crate::platform::Platform;
use crate::rs_util::{dos_file_name, dos_name_bytes, parse_line, read_line};
use crate::trace;
use crate::tuuli::TuuliModule;
use crate::unit::{valuestr, Hill, UnitModule, NUM_WC_HILLS};
use std::io::{BufRead, BufReader, Read};

pub struct ReplModule<'g, 'i, 'l, 'm, 'p, 's, 't, 'u, P: Platform> {
    g: &'g GraphModule<'m, 'p, 's, P>,
    i: &'i InfoModule<'g, 'l, 'm, 'p, 's, 'u, P>,
    l: &'l LangModule<'s, P>,
    m: &'m MakiModule,
    p: &'p PcxModule<'m, 's, P>,
    s: &'s P,
    tuuli: &'t TuuliModule<'g, 'm, 'p, 's, P>,
    u: &'u UnitModule<'g, 'l, 'm, 'p, 's, P>,
}

/// The local variables of `replays`, which its nested procedures share
struct Replay {
    filenames: Vec<Vec<u8>>,
    numreplays: i32,
    intro: bool,

    result: u8,

    acthill: Hill,

    startx: i32,
    starty: i32,
    maxturns: i32,
    nytmaki: i32,
    lmaara: i32,
    realhp: i32,
    flstart: i32,
    flstop: i32,
    hrx: i32,
    hry: i32,
    startgate: i32,
    suitcolor: u8,
    skicolor: u8,
    author: Vec<u8>,
    name: Vec<u8>,
    when: Vec<u8>,
    hillfilename: Vec<u8>,
    laskuri: i32,
    mycheck: i32,
    rd: [[u8; 1001]; 5],
    mcliivi: bool,
}

impl<'g, 'i, 'l, 'm, 'p, 's, 't, 'u, P: Platform> ReplModule<'g, 'i, 'l, 'm, 'p, 's, 't, 'u, P> {
    pub fn new(
        g: &'g GraphModule<'m, 'p, 's, P>,
        i: &'i InfoModule<'g, 'l, 'm, 'p, 's, 'u, P>,
        l: &'l LangModule<'s, P>,
        m: &'m MakiModule,
        p: &'p PcxModule<'m, 's, P>,
        s: &'s P,
        tuuli: &'t TuuliModule<'g, 'm, 'p, 's, P>,
        u: &'u UnitModule<'g, 'l, 'm, 'p, 's, P>,
    ) -> Self {
        ReplModule {
            g,
            i,
            l,
            m,
            p,
            s,
            tuuli,
            u,
        }
    }

    fn txts(&self, phase: u8) -> Vec<u8> {
        let mut str1 = self.l.lstr(343).to_vec();
        match phase {
            1 => str1 = b"50%".to_vec(),
            2 => str1 = b"75%".to_vec(),
            3 => str1 = b"100%".to_vec(),
            4 => str1 = b"150%".to_vec(),
            5 => str1 = b"200%".to_vec(),
            _ => {}
        }

        str1
    }

    async fn introbox(&self, phase: u32) {
        let ix: i32 = 30;
        let mut iy: i32 = 30;

        if let 0..=3 = phase {
            iy = 140;
        }

        //{   fillbox(ix-3,iy-4,ix+245,iy+35,254); }
        self.g.fill_box(
            (ix - 7) as u16,
            (iy - 7) as u16,
            (ix + 261) as u16,
            (iy + 32) as u16,
            248,
        );
        self.g.fill_box(
            (ix - 6) as u16,
            (iy - 6) as u16,
            (ix + 260) as u16,
            (iy + 31) as u16,
            243,
        );
        //{   fillarea(ix-6,iy-6,ix+260,iy+31,63); }

        self.g.font_color(246);
        self.g.write_font(ix, iy, self.l.lstr(360 + phase * 2));
        self.g.write_font(ix, iy + 10, self.l.lstr(361 + phase * 2));

        self.u.wait_for_key3(ix + 246, iy + 21).await;
    }

    async fn playreplay(&self, r: &mut Replay, gdetail: u8, lumi: &mut LumiModule) {
        let mut temp: i32;
        let mut jumper_anim: i32;
        let mut ski_anim: i32;
        let mut hp: i32;
        let mut wind: i32;
        let mut rturns: i32;
        let mut mode: i32;
        let mut oldmode: i32 = 0;
        let mut speed: u8;
        let mut place: u16;
        let mut move_: i32;

        let mut str1: Vec<u8>;

        let mut x: i32;
        let mut y: i32;
        let mut sx: i32;
        let mut sy: i32;
        let mut deltax: i32;
        let mut deltay: i32;
        let keulax: i32;
        let keulay: i32;

        let mut xy = [[0i32; 1001]; 2];

        // IOResult is always 0 here, because the callers have already read
        // the error of the reset of the replay file.
        {
            //{ LoadInfo(nytmaki,ActHill); }

            self.s.set_ch(1);

            x = r.startx;
            y = r.starty;

            for temp in 0..=r.maxturns as usize {
                x = (x + r.rd[0][temp] as i32 - 128) as i16 as i32;
                y = (y + r.rd[1][temp] as i32 - 128) as i16 as i32;
                xy[0][temp] = x;
                xy[1][temp] = y;
            }

            speed = 3; //{ normaali nopeus }
            place = 0;
            rturns = 0;

            mode = 3; /*{ 0 - pause, 1 - play fwd, 2 - play rew,
                      3 - play once then pause
                      4 - thru once (speed change)
                      5 - one step forward }*/

            /*{   LStyle:=Random(2)*Random(2);
            LStyle:=1; }*/

            if gdetail == 1 {
                r.lmaara = 0;
            }

            lumi.vie_lmaara(r.lmaara as u16);

            if r.intro {
                mode = 1;
            }

            let mut keula_x = 0;
            self.u.load_hill(&mut keula_x, r.nytmaki, &r.acthill).await;

            self.m.x.set(0);
            self.m.y.set(0);

            self.g.draw_hill_screen().await;

            self.p.siirra_standardi_paletti();
            if r.lmaara > 1000 {
                self.p.tumma_lumi();
            }

            self.p.load_suit(r.suitcolor, 0);
            self.p.load_skis(r.skicolor, 0);

            if !r.mcliivi {
                self.p.siirra_liivi_pois();
            }

            self.p.muuta_replay(mode as u8);

            self.p.aseta_paletti();

            sx = 0;
            sy = 0;

            keulax = xy[0][r.flstart as usize];
            keulay = xy[1][r.flstart as usize];

            x = r.startx;
            y = r.starty;
            jumper_anim = r.rd[2][rturns as usize] as i32;
            ski_anim = r.rd[3][rturns as usize] as i32;
            wind = r.rd[4][rturns as usize] as i32 - 128;

            loop {
                //{ replayluuppi }

                if mode > 0 {
                    self.s.set_ch(1);
                    self.s.set_ch2(1);

                    /*{      temp:=0;

                    case mode of
                    1 : temp:=1;
                    2 : temp:=-1;
                    end;

                    X:=X+temp*(RD[0,RTurns]-128);
                    Y:=Y+temp*(RD[1,RTurns]-128); }*/

                    if r.intro {
                        match rturns {
                            1 => {
                                self.introbox(0).await;
                                self.introbox(1).await;
                                self.introbox(2).await;
                            }
                            100 => self.introbox(3).await,
                            223 => self.introbox(4).await,
                            257 => self.introbox(5).await,
                            420 => self.introbox(6).await,
                            550 => {
                                self.introbox(7).await;
                                self.introbox(8).await;
                                self.introbox(9).await;
                            }
                            822 => {
                                self.introbox(10).await;
                                self.s.set_ch(27);
                            }
                            _ => {}
                        }
                    }

                    x = xy[0][rturns as usize];
                    y = xy[1][rturns as usize];

                    jumper_anim = r.rd[2][rturns as usize] as i32;
                    ski_anim = r.rd[3][rturns as usize] as i32;
                    wind = r.rd[4][rturns as usize] as i32 - 128;

                    deltax = self.m.x.get();
                    deltay = self.m.y.get();

                    if (160..864).contains(&x) {
                        sx = x - 160;
                    }
                    if (100..412).contains(&y) {
                        sy = y - 100;
                    }

                    self.m.x.set(sx);
                    self.m.y.set(sy);

                    if self.m.x.get() > 704 {
                        self.m.x.set(704);
                    }
                    if self.m.y.get() > 312 {
                        self.m.y.set(312);
                    }
                    if self.m.x.get() < 0 {
                        self.m.x.set(0);
                    }
                    if self.m.y.get() < 0 {
                        self.m.y.set(0);
                    }

                    self.m.tulosta();

                    self.draw_lumi(
                        lumi,
                        deltax - self.m.x.get(),
                        deltay - self.m.y.get(),
                        self.tuuli.value.get(),
                        r.lmaara as u16,
                        true,
                    );

                    if r.hrx > 0 {
                        self.g
                            .draw_anim(r.hrx - self.m.x.get(), r.hry - self.m.y.get(), 68);
                        //{ mäkienkkakeppi }
                    }

                    self.g.draw_anim(
                        x - self.m.x.get(),
                        y - self.m.y.get() - 2,
                        jumper_anim as u8,
                    );
                    self.g
                        .draw_anim(x - self.m.x.get(), y - self.m.y.get() - 1, ski_anim as u8);

                    //{       EWritefont(308,69,'RTURNS '+txt(RTurns)); }

                    if !r.intro {
                        self.g.draw_anim(227, 2, 64);

                        self.g.font_color(246);

                        if
                        /*{ (mode=3) or }*/
                        rturns % 30 > 15 {
                            self.g.write_font(2, 2, b"R");
                        }

                        self.g.font_color(240);

                        self.g.e_write_font(
                            308,
                            9,
                            &[&r.acthill.name as &[u8], b" K", &txt(r.acthill.kr)].concat(),
                        );
                        self.g.e_write_font(308, 19, &r.author);
                        //{        Ewritefont(308,29,when); }

                        //{       EWritefont(308,29,'MTURNS '+txt(MaxTurns)); }

                        self.g.draw_anim(150, 30, 69);
                        //{        DrawAnim(250,62,70); }

                        //{        temp:=mode;  }

                        if mode != oldmode {
                            self.p.muuta_replay(mode as u8);
                        }

                        str1 = b"0.00".to_vec();
                        hp = 0;

                        if rturns > r.flstart {
                            let dx = (x - keulax) as f64;
                            let dy = (y - keulay) as f64;
                            hp = round(
                                (nsqrt((dx * dx + dy * dy) as f32) as f64
                                    * r.acthill.pk as f64
                                    * 0.5) as f32,
                            ) * 5;
                            temp = rturns - r.flstart;
                            if rturns > r.flstop {
                                temp = r.flstop - r.flstart;
                                hp = r.realhp;
                            }
                            str1 = txt(round(((temp * 10) as f64 / 7.0) as f32));

                            while str1.len() < 3 {
                                str1.insert(0, b'0');
                            }
                            str1.insert(str1.len() - 2, b'.');

                            //{    hp:=round(nsqrt((matka*matka)+(kkor*kkor))*acthill.pk*0.5)*5; }
                        }

                        self.g.font_color(247);

                        self.g.e_write_font(
                            309,
                            29,
                            &[&self.l.lstr(340) as &[u8], b" ", &str1].concat(),
                        );
                        self.g.e_write_font(
                            309,
                            39,
                            &[&self.l.lstr(341) as &[u8], b" ", &txtp(hp)].concat(),
                        );
                        self.g.e_write_font(
                            309,
                            49,
                            &[&self.l.lstr(342) as &[u8], b" ", &self.txts(speed)].concat(),
                        );
                        str1 = Vec::new();
                        if let 1..=5 = r.startgate {
                            str1 = self.l.lstr((26 + r.startgate) as u32).to_vec();
                        }

                        if r.startgate > 10 {
                            str1 =
                                [&self.l.lstr(58) as &[u8], b" ", &txt(100 - r.startgate)].concat();
                        }

                        if !str1.is_empty() {
                            self.g.e_write_font(309, 59, &str1);
                        }

                        //{       EWritefont(309,59,txt(mode)); }
                    } else {
                        self.g.font_color(247); //{ tuulta varten vaan }
                    }

                    self.tuuli.value.set(wind);
                    self.tuuli.piirra();
                }

                if mode == 3 {
                    mode = 0;
                }
                //{     if (mode>3) then mode:=3; }

                oldmode = mode;

                if mode > 3 {
                    mode = 3;
                }

                if self.s.key_pressed().await {
                    self.s.wait_for_key_press().await;
                    if
                    /*{(ch2=#80) or (ch2=#72) or}*/
                    self.s.get_ch().to_ascii_uppercase() == b'P' {
                        mode = 3;
                    }
                    if self.s.get_ch2() == 77 {
                        if mode == 1 {
                            mode = 3;
                        } else {
                            mode = 1;
                        }
                    }
                    if self.s.get_ch2() == 75 {
                        if mode == 2 {
                            mode = 3;
                        } else {
                            mode = 2;
                        }
                    }
                    if self.s.get_ch2() == 68 {
                        self.s.set_ch(27);
                    }

                    if self.s.get_ch() == b' ' && mode == 0 {
                        mode = 5;
                    }

                    temp = speed as i32;
                    if (self.s.get_ch() == b'+' || self.s.get_ch2() == 80) && speed < 5 {
                        speed += 1;
                    }
                    if (self.s.get_ch() == b'-' || self.s.get_ch2() == 72) && speed > 0 {
                        speed -= 1;
                    }

                    if temp != speed as i32 && mode == 0 {
                        mode = 4;
                    }

                    if r.intro {
                        mode = oldmode;
                        speed = temp as u8;
                    }
                }

                match mode {
                    /*{     1 : if (oldmode<>2) then inc(Rturns);
                    2 : if (oldmode<>1) then dec(Rturns); }*/
                    1 => {
                        if oldmode != 2 {
                            place += 1;
                        }
                    }
                    2 => {
                        if oldmode != 1 {
                            place += 1;
                        }
                    }
                    _ => {}
                }

                move_ = 0;

                temp = speed as i32;

                if speed == 0 {
                    //{ custom speed }
                    temp = 3;
                    //{       if (Rturns > FLStart) then temp:=3; }
                    if (rturns - r.flstart).abs() < 40 || (rturns - r.flstop).abs() < 40 {
                        temp = 2;
                    }
                    if (rturns - r.flstart).abs() < 20
                    /*{ or
                    (abs(Rturns-FlStop) < 15) }*/
                    {
                        temp = 1;
                    }
                }

                match temp {
                    1 => move_ = (place % 4 == 0) as i32,
                    2 => move_ = (place % 2) as i32,
                    3 => move_ = 1,
                    4 => move_ = 1 + (place % 2) as i32,
                    5 => move_ = 2,
                    _ => {}
                }

                if place > 999 {
                    place = 0;
                }

                match mode {
                    1 => rturns += move_,
                    2 => rturns -= move_,
                    5 => rturns += 1,
                    _ => {}
                }

                if mode == 5 {
                    mode = 3; //{ one step }
                }

                if rturns < 0 {
                    mode = 3;
                    rturns = 0;
                }
                if rturns > r.maxturns {
                    mode = 3;
                    rturns = r.maxturns;
                }

                self.p.aseta_paletti();

                self.g.draw_screen().await;

                if self.s.get_ch() == 27 {
                    break;
                }
            }
        }

        self.s.clearchs();
    }

    fn findreplays(&self, r: &mut Replay) {
        r.numreplays = 0;

        for temp in 0..=100 {
            r.filenames[temp] = Vec::new();
        }

        for name in self.s.find_files("*.SJR") {
            if r.numreplays < 100 {
                r.numreplays += 1;

                let filename = &mut r.filenames[r.numreplays as usize];
                for c in dos_name_bytes(&name).into_iter().take_while(|&c| c != b'.') {
                    // string[12]
                    if filename.len() < 12 {
                        filename.push(c);
                    }
                }
            }
        }
    }

    fn readreplayinfo(&self, r: &mut Replay, filename: &[u8]) {
        let mut f1 = BufReader::new(self.s.open_file(dos_file_name(filename)));

        r.startx = parse_line::<i16>(&mut f1).unwrap() as i32;
        r.starty = parse_line::<i16>(&mut f1).unwrap() as i32;

        r.maxturns = parse_line::<i16>(&mut f1).unwrap() as i32;
        r.nytmaki = parse_line::<i16>(&mut f1).unwrap() as i32;
        r.hillfilename = read_line(&mut f1).unwrap();
        r.hillfilename.truncate(12);
        r.laskuri = parse_line(&mut f1).unwrap(); //{ profile }
        r.lmaara = parse_line::<i16>(&mut f1).unwrap() as i32;
        r.realhp = parse_line::<i16>(&mut f1).unwrap() as i32;
        r.flstart = parse_line::<i16>(&mut f1).unwrap() as i32;
        r.flstop = parse_line::<i16>(&mut f1).unwrap() as i32;
        r.hrx = parse_line::<i16>(&mut f1).unwrap() as i32;
        r.hry = parse_line::<i16>(&mut f1).unwrap() as i32;
        r.suitcolor = parse_line(&mut f1).unwrap();
        r.skicolor = parse_line(&mut f1).unwrap();

        // NameStr = string[30], TimeStr = string[22]
        r.author = read_line(&mut f1).unwrap();
        r.author.truncate(30);
        r.name = read_line(&mut f1).unwrap();
        r.name.truncate(30);
        r.when = read_line(&mut f1).unwrap();
        r.when.truncate(22);

        let temp = parse_line::<i16>(&mut f1).unwrap();
        r.mcliivi = temp != 0;

        r.startgate = parse_line::<i16>(&mut f1).unwrap() as i32;
        r.mycheck = parse_line(&mut f1).unwrap();

        let mut tempch: u8 = 1;

        while tempch != b'*' {
            tempch = read_char(&mut f1);
        }

        for temp in 0..=1000 {
            for temp2 in 0..=4 {
                tempch = read_char(&mut f1);
                r.rd[temp2][temp] = tempch;
            }
        }

        drop(f1);
    }

    fn countcheck(&self, r: &Replay) -> i32 {
        let mut temp: i32 = 0;
        temp = temp.wrapping_add(r.startx * 2 + r.starty);
        temp = temp.wrapping_add(r.maxturns * 3);
        if r.nytmaki <= NUM_WC_HILLS {
            temp = temp.wrapping_add((r.nytmaki * 131) as i16 as i32);
        }
        temp = temp.wrapping_add((valuestr(&r.hillfilename, 3) as i32 * 3) as u16 as i32);
        temp = temp.wrapping_add(r.acthill.profile);
        temp = temp.wrapping_add(((r.realhp + 2) * 69) as i16 as i32);
        temp = temp.wrapping_add(r.flstart + r.flstop);
        temp = temp.wrapping_add(((r.hrx + r.hry) * 2) as i16 as i32);
        temp = temp.wrapping_add(valuestr(&r.author, 2) as i32);
        temp = temp.wrapping_add((r.startgate * 1412) as i16 as i32);

        temp ^ 3675433
    }

    fn trace_replay_load(&self, r: &Replay, filename: &[u8]) {
        let mut fields: Vec<(&str, trace::Value)> =
            vec![("name", filename.into()), ("result", r.result.into())];
        if r.result != 1 {
            fields.extend([
                ("startx", r.startx.into()),
                ("starty", r.starty.into()),
                ("maxturns", r.maxturns.into()),
                ("nytmaki", r.nytmaki.into()),
                ("hillfilename", (&r.hillfilename as &[u8]).into()),
                ("profile", r.laskuri.into()),
                ("lmaara", r.lmaara.into()),
                ("realhp", r.realhp.into()),
                ("flstart", r.flstart.into()),
                ("flstop", r.flstop.into()),
                ("hrx", r.hrx.into()),
                ("hry", r.hry.into()),
                ("suitcolor", r.suitcolor.into()),
                ("skicolor", r.skicolor.into()),
                ("author", (&r.author as &[u8]).into()),
                ("replayname", (&r.name as &[u8]).into()),
                ("when", (&r.when as &[u8]).into()),
                ("mcliivi", r.mcliivi.into()),
                ("startgate", r.startgate.into()),
                ("check", r.mycheck.into()),
                (
                    "rdcrc",
                    (trace::crc32(&r.rd.each_ref().map(|row| &row[..])) as i32).into(),
                ),
            ]);
        }
        trace::record("ReplayLoad", &fields);
    }

    fn clearscreen(&self) {
        self.g.fill_box(145, 50, 319, 199, 243);
        self.g.fill_box(128, 70, 145, 199, 243);

        self.g.fill_area(128, 70, 145, 199, 64);
        self.g.fill_area(145, 50, 319, 199, 64);

        self.g.font_color(241);
        self.g
            .write_font(170, 51, &[&self.l.lstr(25) as &[u8], b":"].concat());
        self.g.write_font(150, 185, self.l.lstr(146));
    }

    async fn noreplays(&self) {
        self.clearscreen();
        self.g.font_color(246);
        self.g.write_font(170, 80, self.l.lstr(290));
        self.u
            .getch(177 + self.g.font_len(self.l.lstr(290)), 80, 243)
            .await;
        self.g.draw_screen().await;
    }

    /*{
     procedure savedscreen;
     begin

      move(mem[TempVideo:0],mem[Video:0],64000);
      TakaisinAlkuosa(0);
      SiirraStandardiPaletti;
      SpecialMainPaletti;
       AsetaPaletti;

     end;
    }*/

    pub async fn replays(&self, start: bool, version: &[u8], gdetail: u8, lumi: &mut LumiModule) {
        let mut r = Replay {
            filenames: vec![Vec::new(); 501],
            numreplays: 0,
            intro: false,
            result: 0,
            acthill: Hill::default(),
            startx: 0,
            starty: 0,
            maxturns: 0,
            nytmaki: 0,
            lmaara: 0,
            realhp: 0,
            flstart: 0,
            flstop: 0,
            hrx: 0,
            hry: 0,
            startgate: 0,
            suitcolor: 0,
            skicolor: 0,
            author: Vec::new(),
            name: Vec::new(),
            when: Vec::new(),
            hillfilename: Vec::new(),
            laskuri: 0,
            mycheck: 0,
            rd: [[0; 1001]; 5],
            mcliivi: false,
        };
        let mut oldindex: i32;
        let mut temp: i32;
        let mut index: i32;

        if start {
            r.result = 0;

            r.intro = true;

            if !self.s.file_exists("INTRO.SJR") {
                r.result = 1;
            } else {
                self.readreplayinfo(&mut r, b"INTRO.SJR");
            }

            self.trace_replay_load(&r, b"INTRO.SJR");
            if r.result == 0 {
                self.u.load_info(r.nytmaki, &mut r.acthill).await;
                self.playreplay(&mut r, gdetail, lumi).await;
            }
        } else {
            self.findreplays(&mut r);

            oldindex = 0;

            /*{
            Alloclarge(TempVideo,64000);
            move(mem[Video:0],mem[TempVideo:0],64000);
            TallennaAlkuosa(0);
            }*/

            if r.numreplays > 0 {
                index = 1;

                loop {
                    if oldindex != index {
                        r.result = 0;
                        r.intro = false;

                        let filename = [&r.filenames[index as usize] as &[u8], b".SJR"].concat();

                        if upstr(&r.filenames[index as usize]) == b"INTRO" {
                            r.intro = true;
                        }

                        if !self.s.file_exists(dos_file_name(&filename)) {
                            r.result = 1;
                        } else {
                            self.readreplayinfo(&mut r, &filename);
                        }

                        if r.nytmaki > NUM_WC_HILLS && r.result == 0 {
                            r.result = 2;
                            temp = 0;

                            let num_extra_hills = self.u.num_extra_hills.get() as i32;
                            if num_extra_hills > 0 {
                                while temp < num_extra_hills {
                                    temp += 1;
                                    if self.u.hillfile(temp) == r.hillfilename {
                                        self.u.load_info(temp + NUM_WC_HILLS, &mut r.acthill).await;
                                        if r.acthill.profile == r.laskuri {
                                            temp = num_extra_hills + 1;
                                            r.result = 0;
                                        }
                                    }
                                }
                            }
                        } else if r.result == 0 {
                            self.u.load_info(r.nytmaki, &mut r.acthill).await;
                        }

                        if self.countcheck(&r) != r.mycheck {
                            r.result = 3;
                        }
                        self.trace_replay_load(&r, &filename);

                        if r.result != 0 {
                            r.author = b"Unknown".to_vec();
                            r.name = b"Not a valid replay.".to_vec();
                            r.when = b"-".to_vec();
                            r.acthill.name = b"?".to_vec();
                            match r.result {
                                1 => r.acthill.name = b"File didn't open.".to_vec(),
                                2 => r.acthill.name = b"Extra Hill Not Found.".to_vec(),
                                3 => r.acthill.name = b"Checksum mismatch.".to_vec(),
                                _ => {}
                            }

                            r.acthill.kr = -1;
                        }

                        self.clearscreen();

                        self.g.write_font(
                            272,
                            85,
                            &[&txt(index) as &[u8], b"/", &txt(r.numreplays)].concat(),
                        );

                        self.g.write_font(150, 106, self.l.lstr(291));
                        self.g.write_font(150, 126, self.l.lstr(292));
                        self.g.write_font(150, 146, self.l.lstr(294));
                        self.g.write_font(150, 71, self.l.lstr(293));
                        self.g.write_font(170, 163, &r.when);

                        if r.acthill.kr > 0 {
                            self.g.font_color(246);
                        } else {
                            self.g.font_color(241);
                        }

                        self.g.fill_box(163, 78, 257, 98, 248);
                        self.g.fill_box(164, 79, 256, 97, 243);
                        self.g.write_font(170, 85, &r.filenames[index as usize]);
                        self.g.font_color(240);
                        self.g.write_font(170, 115, &r.author);
                        self.g.write_font(170, 135, &r.name);
                        if r.acthill.kr > 0 {
                            self.g.write_font(
                                170,
                                155,
                                &[&r.acthill.name as &[u8], b" K", &txt(r.acthill.kr)].concat(),
                            );
                        } else {
                            self.g.write_font(170, 155, &r.acthill.name);
                        }
                        /*{        writefont(170,168,txt(countcheck));
                        writefont(170,173,txt(mycheck)); }*/

                        self.g.draw_screen().await;
                    }

                    self.s.set_ch(1);
                    self.s.set_ch2(1);
                    oldindex = index;

                    self.s.wait_for_key_press().await;

                    let (ch, ch2) = (self.s.get_ch(), self.s.get_ch2());
                    if ch2 == 77 || ch2 == 80 || ch == b' ' || ch == b'+' {
                        index += 1;
                    }
                    if ch2 == 75 || ch2 == 72 || ch == b'-' {
                        index -= 1;
                    }
                    if ch2 == 68 {
                        self.s.set_ch(27);
                    }

                    if self.s.get_ch() == 13 && r.result == 0 {
                        self.playreplay(&mut r, gdetail, lumi).await;
                        oldindex = 0; //{ piirtää ruudun uudestaan sitten }
                        self.i.draw_main_menu().await;
                        self.u.main_menu_text(0, version);
                        //{ savedscreen; }
                    }

                    if self.s.get_ch2() == 83 {
                        //{ delete! }
                        if self
                            .u
                            .might_delete(
                                &[&r.filenames[index as usize] as &[u8], b".SJR"].concat(),
                            )
                            .await
                            == 0
                        {
                            self.findreplays(&mut r);
                        }
                        self.s.set_ch(1);
                        oldindex = 0;
                        self.i.draw_main_menu().await;
                        self.u.main_menu_text(0, version);

                        if r.numreplays == 0 {
                            self.noreplays().await;
                            self.s.set_ch(27);
                        }
                    }

                    if index < 1 {
                        index = r.numreplays;
                    }
                    if index > r.numreplays {
                        index = 1;
                    }

                    if self.s.get_ch() == 27 {
                        break;
                    }
                }
            } else {
                self.noreplays().await;
            }

            //{ freemem(TempVideo,64000); }
        }
    }

    // originally in SJ3UNIT.PAS
    fn draw_lumi(
        &self,
        lumi: &mut LumiModule,
        delx: i32,
        dely: i32,
        wind: i32,
        lmaara: u16,
        draw: bool,
    ) {
        if lmaara > 0 {
            lumi.update(
                self.m.video.borrow_mut(),
                delx * 2,
                dely * 2,
                wind * 8,
                draw,
            );
        }
    }
}

/// `Read` of a char from a text file in Free Pascal, which gives #26 at the
/// end of the file
fn read_char(f: &mut BufReader<impl Read>) -> u8 {
    match f.fill_buf().unwrap().first() {
        Some(&c) => {
            f.consume(1);
            c
        }
        None => 26,
    }
}
