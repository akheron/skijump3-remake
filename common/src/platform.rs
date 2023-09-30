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

    type ReadableFile<'a>: Read
    where
        Self: 'a;
    fn open_file<'a, P: AsRef<str>>(&'a self, path: P) -> Self::ReadableFile<'a>;

    fn file_exists<P: AsRef<str>>(&self, path: P) -> bool;
    fn remove_file<P: AsRef<str>>(&self, path: P);
}
