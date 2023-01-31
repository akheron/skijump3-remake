use crate::graph::GraphModule;
use crate::help::HelpModule;
use crate::lang::LangModule;
use crate::sdlport::SDLPortModule;

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

pub struct UnitModule<'g, 'h, 'l, 'm, 's, 'si> {
    g: &'g GraphModule<'m, 's, 'si>,
    h: &'h HelpModule,
    l: &'l LangModule,
    s: &'s SDLPortModule<'si>,
}

impl<'g, 'h, 'l, 'm, 's, 'si> UnitModule<'g, 'h, 'l, 'm, 's, 'si> {
    pub fn new(
        g: &'g GraphModule<'m, 's, 'si>,
        h: &'h HelpModule,
        l: &'l LangModule,
        s: &'s SDLPortModule<'si>,
    ) -> Self {
        UnitModule { g, h, l, s }
    }

    /*
    function MakeMenu(x,y,length,height,items,index:integer;bgcolor,phase,tab:byte):integer;
    var{ tempch1,tempch2 : char; }
    {    index : integer; }
        xx,yy : integer;
        out,fill,putkeen : boolean;
        boxcol : byte;
        del : boolean;
        oldindex : integer;
        thing : integer;
    */
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

        let index = index.abs();
        let xx = (x - 6) as u16;
        let yy = (y - 3 + ((index - 1) * height)) as u16;

        let boxcol = 240;

        loop {
            self.g
                .box_(xx, yy, xx + length as u16, yy + height as u16, bgcolor);
            if fill {
                self.g
                    .fill_area(xx, yy, xx + length as u16, yy + height as u16, thing);
            }

            self.h.clearchs();
            let oldindex = index;

            if self.s.key_pressed() {
                self.s.wait_for_key_press();
            }

            self.g.draw_screen();

            // just break
            // if true != false {
            //     break;
            // }
        }

        self.g.draw_screen();
        0 // todo
    }
    /*
     repeat

       clearchs; oldindex:=index;

      if (SDLPort.KeyPressed) then SDLPort.WaitForKeyPress(ch,ch2);

       case (ch2) of
       #72,#75 :{  if (cols=2) and (tempch2=#75) and (index>tab) then dec(index,tab)
                 else }
                  begin
                   if (index>1) then dec(index)
                                else index:=items+2;
                   if (index=items+1) then dec(index); { skip v�li }
                  { boxcol:=32; }
                  end;
       #77,#80 :{ if (cols=2) and (tempch2=#77) then inc(index,tab)
                  else }
                 begin
                  if (index<items+1) then inc(index)
                                     else index:=1;
                  if (index=items+1) then inc(index); { skip v�li }
                 { boxcol:=32; }
                 end;
       #59..#68 : begin
                   index:=ord(ch2)-58;
                   out:=true;
                  end;
       end; { case }

       case (ch) of
       '0'..'9' : index:=ord(ch)-48;
       'A'..'F' : if (phase<>6) then index:=ord(ch)-55; { me halutaan ett� E on edit }
       'a'..'f' : index:=ord(ch)-87;
       end;

       if (index<=0) or (index>items) then
        begin
         index:=items+2;
         if (putkeen) then index:=items+1;
        end;

       if (ch2=#71) then index:=1;
       if (ch=#27) or (ch2=#79) then
        begin
         index:=items+2;
         if (putkeen) then index:=items+1;
        end;

       if (ch2=#68) then { F10 }
        begin
         out:=true;
         index:=items+2;
         if (putkeen) then index:=items+1;
         if (phase=6) then index:=254;
        end;

       if (ch=#9) and (tab>0) then
        begin { tab }
         out:=true;
         if (tab<254) then index:=tab;
         if (tab=255) then index:=items+2;
        end;

       if (ch=#13) or (ch=' ') then out:=true;

       xx:=x-6;
       yy:=y-3+((index-1)*height);

       if (phase=6) and ((ch=#13) or (upcase(ch)='E') or (ch=#9)) then
        begin
         index:=tab;
         out:=true;
        end;

    (*
       if (index>tab) and (cols=2) then
        begin
         xx:=x+160-6;
         yy:=y-3+((index-1-tab)*height);
        end;
    *)

    {  inc(boxcol); if (boxcol>47) then boxcol:=16; }

      box(xx,yy,xx+length,yy+height,boxcol);

      DrawScreen;

       if (ch2=#83) and (del) then begin out:=true; index:=-index; end; { delete! }

       if (phase=6) and (index<>oldindex) then out:=true;

     until (out);

      box(xx,yy,xx+length,yy+height,bgcolor);
      if (fill) then fillarea(xx,yy,xx+length,yy+height,thing);

     if (phase<>6) then
      begin
       DrawScreen;
       if (index>items) then index:=0;  { exit }
      end;

     clearchs;

     MakeMenu:=index;

    end;

     */
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
}
