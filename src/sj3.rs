use crate::graph::GraphModule;
use crate::info::InfoModule;
use crate::regfree::{REGNAME, REGNUMBER};
use crate::unit::UnitModule;

pub struct SJ3Module<'g, 'h, 'i, 'l, 'm, 'p, 's, 'si, 'u> {
    g: &'g GraphModule<'m, 's, 'si>,
    i: &'i InfoModule<'g, 'l, 'm, 'p, 's, 'si>,
    u: &'u UnitModule<'g, 'h, 'l, 'm, 's, 'si>,
    version_full: &'static [u8],
}

impl<'g, 'h, 'i, 'l, 'm, 'p, 's, 'si, 'u> SJ3Module<'g, 'h, 'i, 'l, 'm, 'p, 's, 'si, 'u> {
    pub fn new(
        g: &'g GraphModule<'m, 's, 'si>,
        i: &'i InfoModule<'g, 'l, 'm, 'p, 's, 'si>,
        u: &'u UnitModule<'g, 'h, 'l, 'm, 's, 'si>,
    ) -> Self {
        SJ3Module {
            g,
            i,
            u,
            version_full: b"3.13-remake0",
        }
    }

    fn draw_full_main(&self) {
        self.i.draw_main_menu();

        // TODO
        //{$IFDEF REG}
        self.u.new_reg_text(REGNAME, REGNUMBER);
        // {$ELSE}
        //self.u.new_unreg_text();
        // {$ENDIF}
    }

    pub fn main_menu(&self) {
        self.g.fill_box(0, 0, 319, 199, 0);

        let mut index = 1;

        // TODO
        // if (languagenumber=255) {
        //     WelcomeScreen(languagenumber);
        //     Replays(true, version, gdetail);
        // }

        // loop {
        self.draw_full_main();
        self.u.main_menu_text(0, self.version_full);
        index = self.u.make_menu(11, 97, 108, 12, 6, index, 8, 4, 2);

        self.g.draw_screen();
        // }
    }
    /*
    {  index:=MakeMenu(30,69,110,10,8,index,243,0,6); }

      index:=MakeMenu(11,97,108,12,6,index,8,4,2);

      case index of
       1 : jumpmenu;
       2 : { signupplayers; }
           begin
            LoadNames(namenumber,jmaara,TeamLineup,false);
            profiles;
            if (pmaara=8) then jmaara:=2 else jmaara:=1;
             jnimet[NumTeams-1]:='Team 2';
            LoadNames(namenumber,jmaara,TeamLineup,true);
           end;
       3 : setupmenu;
       4 : showtops(0);
       5 : if (NumExtraHills>0) then showtops(1)
                                else showtops(2);
       6 : replays(false,version,gdetail);

    {    testanims; }

    {   1 : begin justfourhills:=false; cup; end;
       2 : begin justfourhills:=true; cup; end;
       3 : teamcup;
       4 : newkingofthehill;
       5 : training;
       6 : signupplayers;
       7 : setupmenu;
       8 : showtops; }
      end; { case }

      if (index=0) then index:=quitting(0);

     until (index=0);

    { TestAnims; }

     AsetaMoodi($3);

    { Textmode(CO80); }

    {  for a:=1 to 50 do write(index,' '); }

    {  readln; }

    end;

         */
}
