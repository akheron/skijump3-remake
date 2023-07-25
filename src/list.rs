use crate::graph::{AsBytes, GraphModule};
use crate::help::{txt, txtp};
use crate::lang::LangModule;
use crate::pcx::PcxModule;
use crate::sdlport::SDLPortModule;
use crate::unit::{UnitModule, NUM_PL};

struct Res {
    pos: i32,
    who: i32,
    name: Vec<u8>,
    pts: i32,
    len1: i32,
    len2: i32,
}

pub struct ListModule<'g, 'l, 'm, 'p, 's, 'si, 'u> {
    g: &'g GraphModule<'m, 'p, 's, 'si>,
    l: &'l LangModule,
    p: &'p PcxModule<'m, 's, 'si>,
    s: &'s SDLPortModule<'si>,
    u: &'u UnitModule<'g, 'l, 'm, 'p, 's, 'si>,

    x: i32,
    y: i32,
    last_pos: i32,
    page: i32,
    plus: i32,
    own_players: i32,
    players: i32,
    phase: i32,
    columns: u8,
    setcol1: u8,
    setcol2: u8,
    setcol3: u8,
    status: i32,
    inv_back: bool,
    first_page: bool,
    header_str: Vec<u8>,
}

// 0-indexed in Rust (was 1-indexed in Pascal)
const COLUMN_X: [[i32; 6]; 2] = [[24, 32, 184, 199, 252, 275], [19, 23, 153, 154, 155, 0]];

const START_Y: i32 = 23;

impl<'g, 'l, 'm, 'p, 's, 'si, 'u> ListModule<'g, 'l, 'm, 'p, 's, 'si, 'u> {
    pub async fn new(
        g: &'g GraphModule<'m, 'p, 's, 'si>,
        l: &'l LangModule,
        p: &'p PcxModule<'m, 's, 'si>,
        s: &'s SDLPortModule<'si>,
        u: &'u UnitModule<'g, 'l, 'm, 'p, 's, 'si>,
    ) -> ListModule<'g, 'l, 'm, 'p, 's, 'si, 'u> {
        let mut ls = Self {
            g,
            l,
            p,
            s,
            u,
            x: 0,
            y: 0,
            last_pos: 0,
            page: 0,
            plus: 0,
            own_players: 0,
            players: 0,
            phase: 0,
            columns: 0,
            setcol1: 0,
            setcol2: 0,
            setcol3: 0,
            status: 0,
            inv_back: false,
            first_page: false,
            header_str: vec![0; 100],
        };
        ls.reset_list(1, NUM_PL as i32, -1, b"", false).await;
        ls
    }

    async fn new_page(&mut self) {
        let color: u8;
        match self.phase {
            0 => {
                //{ Quali }
                self.columns = 1;
                self.plus = 7;
                color = 0;
            }
            1 => {
                //{ Scoreboard }
                self.columns = 1;
                self.plus = 8;
                color = 0;
            }
            2 => {
                //{ 4hills }
                self.columns = 2;
                self.plus = 8;
                color = 4;
            }
            3 => {
                //{ WCStandings }
                self.columns = 2;
                self.plus = 8;
                color = 3;
            }
            4 => {
                //{ king of the hill }
                self.columns = 1;
                self.plus = 8;
                color = 2;
                self.setcol3 = 246; //{ ei turkoosia t�nne }
            }
            5 => {
                //{ Team Cup Standings }
                self.columns = 1;
                self.plus = 10;
                color = 1;
            }
            6 => {
                //{ Team Cup WC standings }
                self.columns = 1;
                self.plus = 10;
                color = 3;
            }
            7 => {
                //{ Stats screens (kokeilu) }
                self.columns = 1;
                self.plus = 200; //{ vaikuttaa paljon! }
                color = 5;
            }
            _ if self.phase >= 8 => {
                panic!("Invalid phase {}", self.phase);
            }
            _ => {
                self.last_pos = 0;
                return;
            }
        };

        if self.phase >= 0 {
            if self.inv_back {
                //{ tota pit�� tsiigata viel�... }
                self.g.draw_hill_screen().await;
                if self.first_page {
                    self.p.siirra_standardi_paletti();
                    self.p.tallenna_alkuosa(0);
                    self.p.savyta_paletti(0, 40);
                    self.p.aseta_paletti();
                    self.first_page = false;
                }
                self.g.draw_anim(5, 2, 62); //{ logo kehiin }
            } else {
                self.g.new_screen(1, color).await;
            }
            self.g.font_color(240);
            self.g.write_font(30, 6, &self.header_str);
        }
        self.last_pos = 0;
    }

    pub async fn reset_list(
        &mut self,
        omat: i32,
        kaikki: i32,
        tyyli: i32,
        header: impl AsBytes,
        inv: bool,
    ) {
        self.x = 0;
        self.y = START_Y;
        self.own_players = omat;
        self.players = kaikki;
        self.phase = tyyli;
        self.header_str = header.as_bytes().to_vec();

        self.inv_back = inv;

        self.first_page = true;

        self.page = 1;
        self.plus = 7;

        self.setcol1 = 240;
        self.setcol2 = 246;
        self.setcol3 = 247;

        self.columns = 1; //{ montako saraketta per sivu }
        self.new_page().await;
    }

    //{ 0 - LEAVING, 1 - PAGE_END }
    async fn wait_for_key(&mut self, from: u8) {
        let mut temp: i32;
        let mut good: bool;

        self.g.font_color(241);

        if self.page > 1 {
            self.g
                .e_write_font(319, 5, &[b"(-" as &[u8], &self.l.lstr(246)].concat());
        }

        let mut str1 = self.l.lstr(247);
        if from == 0 {
            str1 = self.l.lstr(248);
        }

        self.g
            .e_write_font(319, 13, &[&str1 as &[u8], b"-)"].concat());

        self.g.draw_screen().await;

        loop {
            temp = self.status;
            if from == 0 {
                temp = -1;
            }

            good = false;

            self.s.wait_for_key_press();

            match self.s.ch.get() {
                13 | 27 => {
                    self.status = -1;
                    good = true;
                }
                b' ' => {
                    self.page += 1;
                    self.status = temp;
                    good = true;
                }
                3 => {
                    self.status = -2;
                    good = true;
                    //{ CTRL-C }
                }
                _ => {}
            }

            if self.s.ch.get() == 0 {
                match self.s.ch2.get() {
                    68 | 45 => {
                        self.status = -2;
                        good = true;
                        //{ F10, ALT-X }
                    }
                    77 | 81 => {
                        self.page += 1;
                        self.status = temp;
                        good = true;
                        //{ PgDn, A_Right }
                    }
                    71 => {
                        //{ HOME }
                        if self.page > 1 {
                            self.page = 1;
                            self.status = 0;
                            good = true;
                        }
                    }
                    73 | 75 => {
                        //{ PgUp, A_Left }
                        if self.page > 1 {
                            self.page -= 1;
                            match self.phase {
                                0 => self.status = (self.page - 1) * 25,
                                1 | 4 => self.status = (self.page - 1) * 22,
                                2 | 3 => self.status = (self.page - 1) * 44,
                                7 => self.status = self.page - 1,
                                _ => {}
                            }
                            good = true;
                        }
                    }
                    _ => {}
                }
            }

            if good {
                break;
            }
        }

        if self.s.ch.get() == 27 && self.phase == 3 {
            if self.u.quitting(1).await == 0 {
                self.status = -2;
            } else {
                self.s.ch.set(1);
            }
        }

        if self.status >= 0 {
            //{ we need a new screen }
            self.y = START_Y;
            self.x = 0;
            self.new_page().await;
        }
    }

    async fn add_y(&mut self, amount: i32) {
        if amount == 0 {
            self.y += self.plus;
        } else {
            self.y += amount;
        }

        //{ s��d� t��!!! 192jees? }
        if self.y > 191 {
            if self.columns == 2 && self.x == 0 {
                //{ nyy kolumni }
                self.y = START_Y;
                self.x = 160;
            } else {
                //{ we want new page }
                self.wait_for_key(1).await;
            }
        }
    }

    pub async fn entry(
        &mut self,
        num: i32,
        pos: i32,
        who: i32,
        name: &[u8],
        pts: i32,
        len1: i32,
        len2: i32,
        extra: &[u8],
    ) -> i32 {
        let mut col1: u8 = 241;
        let mut col2: u8 = self.setcol2 + 5;
        let mut col3: u8 = self.setcol3 + 5;
        let mut str1: Vec<u8>;
        let mut slen1: Vec<u8>;
        let mut slen2: Vec<u8>;

        if who > self.players - self.own_players {
            //{ meid�n j�tki� }
            col1 = self.setcol1;
            col2 = self.setcol2;
            col3 = self.setcol3;
        }

        self.g.font_color(col1);

        self.status = num;

        if pos > 0 {
            self.g.font_color(col2);

            if pos != self.last_pos {
                self.g.e_write_font(
                    self.x + COLUMN_X[(self.columns - 1) as usize][0],
                    self.y,
                    &[&txt(pos) as &[u8], b"."].concat(),
                );
            }
            self.last_pos = pos;

            self.g.font_color(col1);

            let n = if self.columns == 2 {
                self.g.nsh(name, 98)
            } else {
                self.g.nsh(name, 122)
            };

            self.g.write_font(
                self.x + COLUMN_X[(self.columns - 1) as usize][1],
                self.y,
                &n,
            );

            self.g.font_color(col1);

            str1 = match self.phase {
                3 | 6 => txt(pts),
                _ => txtp(pts),
            };

            self.g.e_write_font(
                self.x + COLUMN_X[(self.columns - 1) as usize][2],
                self.y,
                &str1,
            );

            slen1 = txtp(len1);
            while slen1.len() < 5 {
                slen1.insert(0, '$' as u8);
            }
            slen2 = txtp(len2);
            while slen2.len() < 5 {
                slen2.insert(0, '$' as u8);
            }

            if len1 == 0 {
                str1 = Vec::new();
            } else if len2 == 0 {
                str1 = [b"(", &slen1 as &[u8], b"\xAB)"].concat();
            } else {
                str1 = [b"(", &slen1 as &[u8], b"-", &slen2, b"\xAB)"].concat();
            }

            if !str1.is_empty() {
                self.g.font_color(col3);
                self.g.write_font(
                    self.x + COLUMN_X[(self.columns - 1) as usize][3],
                    self.y,
                    &str1,
                );
            }

            if !extra.is_empty() {
                for temp in 0..extra.len() {
                    match extra[temp] {
                        b'Q' => {
                            self.g.font_color(col2);
                            self.g.write_font(
                                self.x + COLUMN_X[(self.columns - 1) as usize][4],
                                self.y,
                                b"Q",
                            );
                        }
                        b'W' => {
                            self.g.font_color(col3);
                            self.g.write_font(
                                self.x + COLUMN_X[(self.columns - 1) as usize][4],
                                self.y,
                                b"Q WC",
                            );
                        }
                        b'I' => {
                            self.g.font_color(249);
                            self.g.write_font(
                                self.x + COLUMN_X[(self.columns - 1) as usize][5],
                                self.y,
                                &[b"INJ-", &extra[temp + 1..]].concat(),
                            );
                        }
                        b'K' => {
                            self.g.font_color(col2);
                            self.g.write_font(
                                self.x + COLUMN_X[(self.columns - 1) as usize][5],
                                self.y,
                                self.l.lstr(143),
                            );
                        }
                        b'R' => {
                            self.g.font_color(col2);
                            self.g.write_font(
                                self.x + COLUMN_X[(self.columns - 1) as usize][5],
                                self.y,
                                b"HR!",
                            );
                        }
                        _ => {}
                    }
                }
            }

            if extra[0] == b'L' {
                self.wait_for_key(0).await;
            } else {
                self.add_y(0).await;
            }
        } else {
            //{ v�lihuomautus (ei nimi) tai muu s�hly }
            if extra[0] == b'L' {
                //{ teko syy p��st� pois }
                self.wait_for_key(0).await;
            } else {
                //{ tavalinen v�lihuomautus }
                if who > 0 {
                    self.add_y(self.plus / 2).await;
                }

                if self.status >= 0 {
                    //{ ei ole pois l�hd�ss� }
                    self.g.font_color(246);
                    self.g.write_font(
                        self.x + COLUMN_X[(self.columns - 1) as usize][1],
                        self.y,
                        name,
                    );

                    self.add_y(0).await;

                    if who > 1 && self.status >= 0 {
                        self.add_y(self.plus / 2).await;
                    }
                }
            }
        }

        if self.status >= 0 {
            self.status += 1;
        } else {
            //{ pois l�hd�ss� listalta }
            if self.inv_back {
                self.p.takaisin_alkuosa(0);
                self.p.aseta_paletti();
            }
            self.first_page = true;
        }

        self.status
    }
}
