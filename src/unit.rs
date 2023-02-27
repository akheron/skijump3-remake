use crate::graph::GraphModule;
use crate::help::{txt, HelpModule};
use crate::lang::LangModule;
use crate::pcx::PcxModule;
use crate::rs_util::{parse_line, read_line};
use crate::sdlport::SDLPortModule;
use std::ffi::OsString;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::unix::prelude::OsStringExt;
use std::path::Path;

pub const NUM_PL: usize = 75; //{ montako pelaajaa on ylip��t��n }
pub const NUM_TEAMS: usize = 15;
pub const MAX_OWN_PL: i32 = 10; //{ montako omaa pelaajaa voi olla }
pub const NUM_WC_HILLS: i32 = 20; //{ montako m�ke� world cupissa }
pub const MAX_EXTRA_HILLS: i32 = 1000; //{ montako extra m�ke� voi olla. check!!! }
pub const MAX_CUSTOMS: i32 = 200; //{ montako custom hill filea .sjc voi olla }

pub const HEX_CH: [u8; 16] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'A', b'B', b'C', b'D', b'E', b'F',
];

pub const WC_POINTS: [u8; 30] = [
    100, 80, 60, 50, 45, 40, 36, 32, 29, 26, 24, 22, 20, 18, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7,
    6, 5, 4, 3, 2, 1,
];
pub const TEAM_POINTS: [i32; 8] = [400, 350, 300, 250, 200, 150, 100, 50];

pub struct Hiscore {
    pub name: String,
    pub pos: u8,
    pub score: i32,
    pub time: String,
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

pub struct UnitModule<'g, 'h, 'l, 'm, 'p, 's, 'si> {
    g: &'g GraphModule<'m, 's, 'si>,
    h: &'h HelpModule,
    l: &'l LangModule,
    p: &'p PcxModule<'m, 's, 'si>,
    s: &'s SDLPortModule<'si>,
}

impl<'g, 'h, 'l, 'm, 'p, 's, 'si> UnitModule<'g, 'h, 'l, 'm, 'p, 's, 'si> {
    pub fn new(
        g: &'g GraphModule<'m, 's, 'si>,
        h: &'h HelpModule,
        l: &'l LangModule,
        p: &'p PcxModule<'m, 's, 'si>,
        s: &'s SDLPortModule<'si>,
    ) -> Self {
        UnitModule { g, h, l, p, s }
    }

    pub fn load_hill(&self, KeulaX: &mut i32, nytmaki: i32, ActHill: &Hill) -> u8 {
        /*
                function LoadHill(var KeulaX:integer;nytmaki:integer;Acthill:hill_type):byte;
        var temp : integer;
            l  : longint;
            str1 : string;
            res : byte;
        begin
        */
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

        let mut str1 = txt(ActHill.kr);

        self.g.font_color(246); //{ 205 }
                                //{          Writefont(160-fontlen(str1) div 2,97,str1); }
        self.g.e_write_font(311, 107, &str1);
        self.g.font_color(240);
        self.g.e_write_font(
            311 - self.g.font_len(&str1),
            107,
            &[&ActHill.name as &[u8], b" K"].concat(),
        );

        self.g.draw_screen();

        /*
                  LataaPCX('FRONT'+Acthill.frindex+'.PCX',1024*512,0,0);

                  TallennaAlkuosa(1);

                  LaskeLinjat(KeulaX,acthill.kr,acthill.pk);

                  l:=0;

                  for temp:=0 to 1023 do
                   inc(l,longint(profiili(temp)*(temp mod 13+profiili(temp) mod 11)) mod 13313);

                   dec(l,1500000);

                  LataaPCX('BACK'+acthill.bkindex+'.PCX',1024*400,Maki.Sivuja,acthill.bkmirror);

                  TakaisinAlkuosa(1);

                  if (l<>acthill.profile) then { profiilichekkaus }
                   begin

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

                   end;

                  SavytaPaletti(1,acthill.bkbright);

                  Loadhill:=res;

        end;

                 */
        // todo
        0
    }

    pub fn make_menu(
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

            self.h.clearchs();
            let oldindex = index;

            let (ch, ch2) = if self.s.key_pressed() {
                self.s.wait_for_key_press()
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
                    index = (ch as i32 - 48);
                }
                b'A'..=b'F' => {
                    if phase != 6 {
                        index = (ch as i32 - 55); /*{ me halutaan ett� E on edit }*/
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
            self.g.draw_screen();

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
            self.g.draw_screen();
            if index > items {
                index = 0;
            } //{ exit }
        }

        self.h.clearchs();
        index
    }

    #[allow(dead_code)]
    pub fn new_unreg_text(&self) {
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
        self.g.draw_screen();
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
                        self.l.lstr(temp as u32 + 26),
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
                        self.l.lstr(temp as u32 + 19),
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

    pub fn new_reg_text(&self, regname: &str, _regnumber: &str) {
        self.g.fill_box(128, 155, 312, 155, 9);
        self.g.font_color(240);
        self.g
            .write_font(132, 163, &[self.l.lstr(35), b" ", self.l.lstr(36)].concat());
        self.g.font_color(246);
        self.g.fill_box(132, 175, 308, 196, 248);
        self.g.write_font(140, 177, regname.as_bytes());
        self.g.draw_screen();
    }

    fn check_file(&self, phase: i32, hill: &mut Hill, str1: &str) {
        let str2: Vec<u8> = if phase == 1 {
            [b"BACK", hill.bk_index.as_slice(), b".PCX"].concat()
        } else {
            [b"FRONT", hill.fr_index.as_slice(), b".PCX"].concat()
        };

        let filename = OsString::from_vec(str2.clone());
        if !Path::new(&filename).exists() {
            println!(
                "Error #345A: File {} does not exist,",
                String::from_utf8_lossy(&str2)
            );
            println!("even though it's mentioned in the {str1} file.");
            println!("Using FRONT1.PCX and BACK1.PCX.");
            println!();
            println!("Press a key...");

            hill.bk_index = b"1".to_vec();
            hill.fr_index = b"1".to_vec();

            self.s.wait_for_key_press();
            /*
               AsetaMoodi($3);
               writeln('Error #345A: File '+filename+' does not exist,');
               writeln('even though it''s mentioned in the ',str1,' file.');
               writeln('Using FRONT1.PCX and BACK1.PCX.');
               writeln;
               writeln('Press a key...');

               hill.bkindex:='1';
               hill.frindex:='1';

               waitforkey;

               AsetaMoodi($13);
              end;
             end;

            */
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
    fn findstart(&self, f1: &mut BufReader<File>, nytmaki: i32) -> u8 {
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

    pub fn load_info(&self, nytmaki: i32, hill: &mut Hill) {
        Self::default_hill(hill);

        let str1 = if nytmaki <= NUM_WC_HILLS {
            "HILLBASE.SKI"
        } else {
            //str1:=hillfile(nytmaki-NumWCHills)+'.SJH';
            unimplemented!("extra hills");
        };
        let temp = if nytmaki > NUM_WC_HILLS { 0 } else { nytmaki };

        let mut f1 = BufReader::new(File::open(str1).unwrap());
        if self.findstart(&mut f1, temp) == 0 {
            hill.name = read_line(&mut f1).unwrap();
            hill.kr = parse_line(&mut f1).unwrap();
            hill.fr_index = read_line(&mut f1).unwrap();
            hill.bk_index = read_line(&mut f1).unwrap();
            hill.bk_bright = parse_line(&mut f1).unwrap();
            hill.bk_mirror = parse_line(&mut f1).unwrap();
            hill.vx_final = parse_line(&mut f1).unwrap();
            let b: u8 = parse_line(&mut f1).unwrap();
            hill.pk = (b as f32) / 100.0;
            let a: i32 = parse_line(&mut f1).unwrap();
            hill.pl_save = (a as f32) / 10000.0;
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
        self.check_file(1, hill, &str1);

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

    pub fn keyname(&self, _chw: u16) -> &'static [u8] {
        unimplemented!();
    }
}

fn valuestr(str0: &[u8], arvo: i32) -> u16 {
    let mut word1: u16 = 0;

    for index in 1..=str0.len() {
        word1 += (str0[index - 1] as i32).wrapping_mul(((index as i32) % 5) + 41) as u16;
    }

    word1.wrapping_mul(((arvo % 7 + 1) + arvo) as u16)
}
