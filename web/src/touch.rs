use crate::webport::{character, named_key};
use common::{Arrows, Controls, JumpPhase};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    Up,
    Down,
    Left,
    Right,
    Esc,
    Corner,
    Main,
    Extra1,
    Extra2,
    /// A tap on the game screen
    Screen,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Key(u8, u8),
    Keyboard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Button {
    pub slot: Slot,
    pub label: &'static str,
    pub action: Action,
    pub repeat: bool,
}

const UP: &str = "\u{25b2}";
const DOWN: &str = "\u{25bc}";
// Without the text variation selector, iOS draws these two as emoji
const LEFT: &str = "\u{25c0}\u{fe0e}";
const RIGHT: &str = "\u{25b6}\u{fe0e}";

pub fn buttons(controls: Controls) -> Vec<Button> {
    match controls {
        Controls::All | Controls::Unchanged => {
            let mut buttons = arrows(Arrows::All);
            buttons.extend([
                esc(),
                button(Slot::Corner, "\u{2328}", Action::Keyboard),
                enter(),
                button(Slot::Extra1, "Space", letter(b' ')),
                button(Slot::Extra2, "Del", key("Delete")),
            ]);
            buttons
        }
        Controls::Select { arrows: a, delete } => {
            let mut buttons = arrows(a);
            buttons.extend([esc(), enter()]);
            if delete {
                buttons.push(button(Slot::Extra2, "Del", key("Delete")));
            }
            buttons
        }
        // Right leaves the list on its last page, so a held arrow must not repeat
        Controls::List => vec![
            button(Slot::Left, LEFT, key("ArrowLeft")),
            button(Slot::Right, RIGHT, key("ArrowRight")),
            esc(),
            enter(),
        ],
        Controls::AnyKey => vec![enter(), button(Slot::Screen, "", key("Enter"))],
        Controls::YesNo { yes } => vec![
            button(Slot::Esc, "\u{2717}", key("Escape")),
            button(Slot::Main, "\u{2713}", letter(yes)),
        ],
        Controls::Text => vec![
            esc(),
            button(Slot::Corner, "\u{2328}", Action::Keyboard),
            enter(),
            button(Slot::Screen, "", Action::Keyboard),
        ],
        Controls::Jump {
            phase,
            keys,
            computer,
        } => jump(phase, keys, computer),
        Controls::Replay { intro: false } => vec![
            button(Slot::Left, LEFT, key("ArrowLeft")),
            button(Slot::Right, RIGHT, key("ArrowRight")),
            esc(),
            button(Slot::Main, "Space", letter(b' ')),
            repeating(Slot::Extra1, "\u{2212}", letter(b'-')),
            repeating(Slot::Extra2, "+", letter(b'+')),
        ],
        Controls::Replay { intro: true } => vec![esc()],
    }
}

fn jump(phase: JumpPhase, keys: [u16; 6], computer: bool) -> Vec<Button> {
    match phase {
        JumpPhase::Info { setup, practice } => {
            let mut buttons = vec![
                esc(),
                button(Slot::Corner, "\u{2699}", letter(setup)),
                enter(),
                button(Slot::Screen, "", key("Enter")),
            ];
            if practice {
                buttons.extend([
                    repeating(Slot::Extra1, "\u{2212}", letter(b'-')),
                    repeating(Slot::Extra2, "+", letter(b'+')),
                ]);
            }
            buttons
        }
        JumpPhase::Start | JumpPhase::Air if computer => vec![esc()],
        // The take-off button starts the inrun, so that the slots and labels stay the same when
        // the jump starts
        JumpPhase::Start => flight(key("Enter"), keys),
        JumpPhase::Air => flight(jump_key(keys[1]), keys),
        JumpPhase::Landing => vec![esc(), enter()],
        JumpPhase::Result { save_replay } => {
            let mut buttons = vec![esc(), enter(), button(Slot::Screen, "", key("Enter"))];
            if let Some(save_replay) = save_replay {
                buttons.push(button(Slot::Corner, "\u{1f4be}", letter(save_replay)));
            }
            buttons
        }
    }
}

fn flight(take_off: Action, keys: [u16; 6]) -> Vec<Button> {
    vec![
        repeating(Slot::Left, LEFT, jump_key(keys[3])),
        repeating(Slot::Right, RIGHT, jump_key(keys[2])),
        esc(),
        button(Slot::Main, UP, take_off),
        button(Slot::Extra1, "T", jump_key(keys[4])),
        button(Slot::Extra2, "R", jump_key(keys[5])),
    ]
}

fn arrows(arrows: Arrows) -> Vec<Button> {
    let up = repeating(Slot::Up, UP, key("ArrowUp"));
    let down = repeating(Slot::Down, DOWN, key("ArrowDown"));
    let left = repeating(Slot::Left, LEFT, key("ArrowLeft"));
    let right = repeating(Slot::Right, RIGHT, key("ArrowRight"));
    match arrows {
        Arrows::UpDown => vec![up, down],
        Arrows::LeftRight => vec![left, right],
        Arrows::All => vec![up, down, left, right],
    }
}

fn esc() -> Button {
    button(Slot::Esc, "Esc", key("Escape"))
}

fn enter() -> Button {
    button(Slot::Main, "\u{21b5}", key("Enter"))
}

fn button(slot: Slot, label: &'static str, action: Action) -> Button {
    Button {
        slot,
        label,
        action,
        repeat: false,
    }
}

fn repeating(slot: Slot, label: &'static str, action: Action) -> Button {
    Button {
        repeat: true,
        ..button(slot, label, action)
    }
}

/// A key as the keyboard gives it
fn key(name: &str) -> Action {
    let (ch, ch2) = named_key(name).unwrap();
    Action::Key(ch, ch2)
}

/// The key of a character of the game as the keyboard gives it without Shift
fn letter(c: u8) -> Action {
    let key = if c.is_ascii() {
        character(c.to_ascii_lowercase() as char)
    } else {
        None
    };
    let (ch, ch2) = key.unwrap_or((c, 0));
    Action::Key(ch, ch2)
}

/// A jump key of the game as `kword` gives it
fn jump_key(kword: u16) -> Action {
    Action::Key((kword >> 8) as u8, kword as u8)
}

pub fn to_json(buttons: &[Button]) -> String {
    let items: Vec<String> = buttons
        .iter()
        .map(|button| {
            let action = match button.action {
                Action::Key(ch, ch2) => format!("\"key\":[{ch},{ch2}]"),
                Action::Keyboard => "\"keyboard\":true".to_string(),
            };
            format!(
                "{{\"slot\":\"{}\",\"label\":\"{}\",{action},\"repeat\":{}}}",
                slot_name(button.slot),
                button.label,
                button.repeat
            )
        })
        .collect();
    format!("[{}]", items.join(","))
}

/// The names of the slots must stay in sync with the grid areas in index.html
fn slot_name(slot: Slot) -> &'static str {
    match slot {
        Slot::Up => "up",
        Slot::Down => "down",
        Slot::Left => "left",
        Slot::Right => "right",
        Slot::Esc => "esc",
        Slot::Corner => "corner",
        Slot::Main => "main",
        Slot::Extra1 => "extra1",
        Slot::Extra2 => "extra2",
        Slot::Screen => "screen",
    }
}

#[cfg(test)]
mod tests {
    use super::{buttons, to_json, Action, Button, Slot};
    use crate::storage::LocalStorageStore;
    use crate::webport::{Modifiers, WebPlatform};
    use crate::Files;
    use common::{Arrows, Controls, JumpPhase, Platform};
    use futures::executor::block_on;

    // The jump keys of a player who changed them all
    const KEYS: [u16; 6] = [0, 0x2011, 0x1e1e, 0x2020, 0x1f1f, 0x1414];

    fn jump(phase: JumpPhase, computer: bool) -> Vec<Button> {
        buttons(Controls::Jump {
            phase,
            keys: KEYS,
            computer,
        })
    }

    fn action(buttons: &[Button], slot: Slot) -> Option<Action> {
        buttons
            .iter()
            .find(|button| button.slot == slot)
            .map(|button| button.action)
    }

    fn slots(buttons: &[Button]) -> Vec<Slot> {
        buttons.iter().map(|button| button.slot).collect()
    }

    #[test]
    fn keys_are_the_keys_of_a_keyboard() {
        let platform = WebPlatform::new(LocalStorageStore::in_memory(), Files::new(&[0]));
        let none = Modifiers {
            shift: false,
            ctrl: false,
            alt: false,
            alt_graph: false,
            meta: false,
            caps_lock: false,
        };
        let select = buttons(Controls::Select {
            arrows: Arrows::All,
            delete: true,
        });
        let replay = buttons(Controls::Replay { intro: false });
        let table = [
            (&select, Slot::Main, "Enter", "Enter"),
            (&select, Slot::Esc, "Escape", "Escape"),
            (&select, Slot::Up, "ArrowUp", "ArrowUp"),
            (&select, Slot::Down, "ArrowDown", "ArrowDown"),
            (&select, Slot::Left, "ArrowLeft", "ArrowLeft"),
            (&select, Slot::Right, "ArrowRight", "ArrowRight"),
            (&select, Slot::Extra2, "Delete", "Delete"),
            (&replay, Slot::Main, " ", "Space"),
            (&replay, Slot::Extra1, "-", "Minus"),
            (&replay, Slot::Extra2, "+", "NumpadAdd"),
        ];
        for (buttons, slot, key, code) in table {
            platform.keydown(key, code, none);
            let (ch, ch2) = block_on(platform.wait_for_key_press());
            assert_eq!(action(buttons, slot), Some(Action::Key(ch, ch2)), "{key}");
        }
    }

    #[test]
    fn yes_sends_the_letter_without_shift() {
        let buttons = buttons(Controls::YesNo { yes: b'K' });
        assert_eq!(action(&buttons, Slot::Main), Some(Action::Key(b'k', 37)));
        assert_eq!(action(&buttons, Slot::Esc), Some(Action::Key(27, 0)));
    }

    #[test]
    fn letter_of_a_short_language_string() {
        // `lch` gives 0 for a character that the language string does not have
        let buttons = buttons(Controls::YesNo { yes: 0 });
        assert_eq!(action(&buttons, Slot::Main), Some(Action::Key(0, 0)));
    }

    #[test]
    fn list_arrows_do_not_repeat() {
        let buttons = buttons(Controls::List);
        assert_eq!(
            slots(&buttons),
            [Slot::Left, Slot::Right, Slot::Esc, Slot::Main]
        );
        assert!(buttons.iter().all(|button| !button.repeat));
    }

    #[test]
    fn delete_only_when_the_screen_uses_it() {
        let without = buttons(Controls::Select {
            arrows: Arrows::UpDown,
            delete: false,
        });
        assert_eq!(
            slots(&without),
            [Slot::Up, Slot::Down, Slot::Esc, Slot::Main]
        );
        let with = buttons(Controls::Select {
            arrows: Arrows::LeftRight,
            delete: true,
        });
        assert_eq!(
            slots(&with),
            [Slot::Left, Slot::Right, Slot::Esc, Slot::Main, Slot::Extra2]
        );
    }

    #[test]
    fn jump_buttons_send_the_jump_keys() {
        let air = jump(JumpPhase::Air, false);
        assert_eq!(action(&air, Slot::Main), Some(Action::Key(0x20, 0x11)));
        assert_eq!(action(&air, Slot::Right), Some(Action::Key(0x1e, 0x1e)));
        assert_eq!(action(&air, Slot::Left), Some(Action::Key(0x20, 0x20)));
        assert_eq!(action(&air, Slot::Extra1), Some(Action::Key(0x1f, 0x1f)));
        assert_eq!(action(&air, Slot::Extra2), Some(Action::Key(0x14, 0x14)));
    }

    #[test]
    fn take_off_button_starts_the_inrun() {
        let start = jump(JumpPhase::Start, false);
        let air = jump(JumpPhase::Air, false);
        assert_eq!(action(&start, Slot::Main), Some(Action::Key(13, 0)));
        // A second tap after the start must not land on a different button
        let layout = |buttons: &[Button]| -> Vec<(Slot, &str)> {
            buttons.iter().map(|b| (b.slot, b.label)).collect()
        };
        assert_eq!(layout(&start), layout(&air));
    }

    #[test]
    fn computer_jumper_can_only_be_skipped_in_the_air() {
        assert_eq!(slots(&jump(JumpPhase::Start, true)), [Slot::Esc]);
        assert_eq!(slots(&jump(JumpPhase::Air, true)), [Slot::Esc]);
    }

    #[test]
    fn practice_info_moves_the_start_place() {
        let info = |practice| {
            jump(
                JumpPhase::Info {
                    setup: b'A',
                    practice,
                },
                false,
            )
        };
        assert_eq!(action(&info(false), Slot::Extra1), None);
        assert_eq!(
            action(&info(true), Slot::Extra1),
            Some(Action::Key(b'-', 0))
        );
        assert_eq!(
            action(&info(true), Slot::Extra2),
            Some(Action::Key(b'+', 0))
        );
        assert_eq!(
            action(&info(true), Slot::Corner),
            Some(Action::Key(b'a', 30))
        );
    }

    #[test]
    fn result_saves_the_replay_only_when_it_can_be_saved() {
        let result = |save_replay| jump(JumpPhase::Result { save_replay }, false);
        assert_eq!(action(&result(None), Slot::Corner), None);
        assert_eq!(
            action(&result(Some(b'T')), Slot::Corner),
            Some(Action::Key(b't', 20))
        );
    }

    #[test]
    fn to_json_gives_key_or_keyboard() {
        let json = to_json(&buttons(Controls::Text)[1..3]);
        assert_eq!(
            json,
            "[{\"slot\":\"corner\",\"label\":\"\u{2328}\",\"keyboard\":true,\"repeat\":false},\
             {\"slot\":\"main\",\"label\":\"\u{21b5}\",\"key\":[13,0],\"repeat\":false}]"
        );
    }
}
