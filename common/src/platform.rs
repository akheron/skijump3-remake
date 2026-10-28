use std::io::{Read, Write};

pub type TPalette = [[u8; 3]; 256];

pub trait Platform {
    fn set_palette(&self, input: &TPalette);
    fn render_phase1(&self, buffer: &[u8]);
    async fn render_phase2(&self);

    async fn key_pressed(&self) -> bool;
    async fn wait_for_key_press(&self) -> (u8, u8);
    async fn wait_for_key(&self) {
        self.putsaa();
        self.wait_for_key_press().await;
    }
    async fn wait_for_key2(&self) -> bool {
        self.putsaa();
        let (ch, ch2) = self.wait_for_key_press().await;
        ch == 0 && ch2 == 68
    }
    fn putsaa(&self);
    fn clearchs(&self);
    fn get_ch(&self) -> u8;
    fn get_ch2(&self) -> u8;
    fn set_ch(&self, ch: u8);
    fn set_ch2(&self, ch: u8);
    fn kword(&self) -> u16 {
        ((self.get_ch() as u16) << 8) + self.get_ch2() as u16
    }

    type WritableFile<'a>: Write
    where
        Self: 'a;
    fn create_file<'a, P: AsRef<str>>(&'a self, path: P) -> Self::WritableFile<'a>;
    /// `create_file` for a `Rewrite` whose `IOResult` the game checks: `None`
    /// when the file cannot be written
    fn try_create_file<'a, P: AsRef<str>>(&'a self, path: P) -> Option<Self::WritableFile<'a>> {
        Some(self.create_file(path))
    }

    type ReadableFile<'a>: Read
    where
        Self: 'a;
    fn open_file<'a, P: AsRef<str>>(&'a self, path: P) -> Self::ReadableFile<'a>;

    fn file_exists<P: AsRef<str>>(&self, path: P) -> bool;
    fn remove_file<P: AsRef<str>>(&self, path: P);
    /// `FindFirst` and `FindNext`: the names of the files that match a
    /// pattern like `*.SJH`
    fn find_files(&self, pattern: &str) -> Vec<String>;

    /// `GetDate` of the DOS unit: year, month, day and day of the week (0 is Sunday)
    fn get_date(&self) -> (u16, u16, u16, u16);
    /// `GetTime` of the DOS unit: hour, minute, second and hundredths of a second
    fn get_time(&self) -> (u16, u16, u16, u16);

    /// `ParamStr`: the command line argument number `index`, from 1, or an
    /// empty string
    fn param_str(&self, index: usize) -> String;
    /// `Wait` of the SDL port
    fn wait(&self, ms: u32);

    /// `DeinitGraphics` of the SDL port: closes the game window
    fn deinit_graphics(&self) {}
    /// True after the player asked to close the game. Then the game runs the
    /// close callback of the SDL port, `CloseAndHalt`.
    fn close_requested(&self) -> bool {
        false
    }
}
