use std::future::Future;
use std::io::{Read, Write};

pub type TPalette = [[u8; 3]; 256];

/// The keys that the game reads. The web port shows touch controls for them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Controls {
    /// Any key of a keyboard
    All,
    /// Enter, Esc, the arrow keys in `arrows`, and Delete if `delete`
    Select {
        arrows: Arrows,
        delete: bool,
    },
    /// The pages of a list: Left, Right, Enter and Esc
    List,
    /// Any key continues
    AnyKey,
    /// The key of the letter `yes` means yes, and any other key means no
    YesNo {
        yes: u8,
    },
    Text,
    /// `keys` are the jump keys `k` of the game: take-off, rotate forward, rotate backward,
    /// telemark landing and two-footed landing at the indexes 1 to 5
    Jump {
        phase: JumpPhase,
        keys: [u16; 6],
        computer: bool,
    },
    /// The replay playback. In the intro replay, only Esc works.
    Replay {
        intro: bool,
    },
    /// The touch controls stay as they are, as in a jump that the game does not show
    Unchanged,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arrows {
    UpDown,
    LeftRight,
    All,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JumpPhase {
    /// The info screen before the jump. The key `setup` opens the setup menu. In `practice`,
    /// + and - move the start place.
    Info {
        setup: u8,
        practice: bool,
    },
    Start,
    /// The inrun and the flight
    Air,
    Landing,
    /// `save_replay` is the key that saves the replay, if the replay can be saved
    Result {
        save_replay: Option<u8>,
    },
}

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

    /// The touch controls are `controls` until the matching `pop_controls`
    fn push_controls(&self, _controls: Controls) {}
    fn pop_controls(&self) {}

    fn controls(&self, controls: Controls) -> ControlsGuard<'_, Self> {
        self.push_controls(controls);
        ControlsGuard { platform: self }
    }

    async fn with_controls<F: Future>(&self, controls: Controls, future: F) -> F::Output {
        let _controls = self.controls(controls);
        future.await
    }
}

/// Keeps the touch controls of `Platform::controls` until it is dropped
#[must_use]
pub struct ControlsGuard<'a, P: Platform + ?Sized> {
    platform: &'a P,
}

impl<P: Platform + ?Sized> ControlsGuard<'_, P> {
    /// Changes the controls of this guard. No guard made after this one can still be alive.
    pub fn set(&self, controls: Controls) {
        self.platform.pop_controls();
        self.platform.push_controls(controls);
    }
}

impl<P: Platform + ?Sized> Drop for ControlsGuard<'_, P> {
    fn drop(&mut self) {
        self.platform.pop_controls();
    }
}
