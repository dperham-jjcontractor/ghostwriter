use anyhow::Result;
use log::debug;

use std::collections::HashMap;
use std::{thread, time};

use evdev::{uinput::VirtualDevice, AttributeSet, EventType as EvdevEventType, InputEvent, KeyCode as EvdevKey};

pub struct Keyboard {
    device: Option<VirtualDevice>,
    key_map: HashMap<char, (EvdevKey, bool)>,
    progress_count: u32,
    no_draw_progress: bool,
}

impl Keyboard {
    pub fn new(no_draw: bool, no_draw_progress: bool) -> Self {
        let device = if no_draw { None } else { Some(Self::create_virtual_device()) };

        Self {
            device,
            key_map: Self::create_key_map(),
            progress_count: 0,
            no_draw_progress,
        }
    }

    fn create_virtual_device() -> VirtualDevice {
        debug!("Creating virtual keyboard");
        let mut keys = AttributeSet::new();

        keys.insert(EvdevKey::KEY_A);
        keys.insert(EvdevKey::KEY_B);
        keys.insert(EvdevKey::KEY_C);
        keys.insert(EvdevKey::KEY_D);
        keys.insert(EvdevKey::KEY_E);
        keys.insert(EvdevKey::KEY_F);
        keys.insert(EvdevKey::KEY_G);
        keys.insert(EvdevKey::KEY_H);
        keys.insert(EvdevKey::KEY_I);
        keys.insert(EvdevKey::KEY_J);
        keys.insert(EvdevKey::KEY_K);
        keys.insert(EvdevKey::KEY_L);
        keys.insert(EvdevKey::KEY_M);
        keys.insert(EvdevKey::KEY_N);
        keys.insert(EvdevKey::KEY_O);
        keys.insert(EvdevKey::KEY_P);
        keys.insert(EvdevKey::KEY_Q);
        keys.insert(EvdevKey::KEY_R);
        keys.insert(EvdevKey::KEY_S);
        keys.insert(EvdevKey::KEY_T);
        keys.insert(EvdevKey::KEY_U);
        keys.insert(EvdevKey::KEY_V);
        keys.insert(EvdevKey::KEY_W);
        keys.insert(EvdevKey::KEY_X);
        keys.insert(EvdevKey::KEY_Y);
        keys.insert(EvdevKey::KEY_Z);

        keys.insert(EvdevKey::KEY_1);
        keys.insert(EvdevKey::KEY_2);
        keys.insert(EvdevKey::KEY_3);
        keys.insert(EvdevKey::KEY_4);
        keys.insert(EvdevKey::KEY_5);
        keys.insert(EvdevKey::KEY_6);
        keys.insert(EvdevKey::KEY_7);
        keys.insert(EvdevKey::KEY_8);
        keys.insert(EvdevKey::KEY_9);
        keys.insert(EvdevKey::KEY_0);

        // Add punctuation and special keys
        keys.insert(EvdevKey::KEY_SPACE);
        keys.insert(EvdevKey::KEY_ENTER);
        keys.insert(EvdevKey::KEY_TAB);
        keys.insert(EvdevKey::KEY_LEFTSHIFT);
        keys.insert(EvdevKey::KEY_MINUS);
        keys.insert(EvdevKey::KEY_EQUAL);
        keys.insert(EvdevKey::KEY_LEFTBRACE);
        keys.insert(EvdevKey::KEY_RIGHTBRACE);
        keys.insert(EvdevKey::KEY_BACKSLASH);
        keys.insert(EvdevKey::KEY_SEMICOLON);
        keys.insert(EvdevKey::KEY_APOSTROPHE);
        keys.insert(EvdevKey::KEY_GRAVE);
        keys.insert(EvdevKey::KEY_COMMA);
        keys.insert(EvdevKey::KEY_DOT);
        keys.insert(EvdevKey::KEY_SLASH);

        keys.insert(EvdevKey::KEY_BACKSPACE);
        keys.insert(EvdevKey::KEY_ESC);

        keys.insert(EvdevKey::KEY_LEFTCTRL);
        keys.insert(EvdevKey::KEY_LEFTALT);

        VirtualDevice::builder()
            .unwrap()
            .name("Virtual Keyboard")
            .with_keys(&keys)
            .unwrap()
            .build()
            .unwrap()
    }

    fn create_key_map() -> HashMap<char, (EvdevKey, bool)> {
        let mut key_map = HashMap::new();

        // Lowercase letters
        key_map.insert('a', (EvdevKey::KEY_A, false));
        key_map.insert('b', (EvdevKey::KEY_B, false));
        key_map.insert('c', (EvdevKey::KEY_C, false));
        key_map.insert('d', (EvdevKey::KEY_D, false));
        key_map.insert('e', (EvdevKey::KEY_E, false));
        key_map.insert('f', (EvdevKey::KEY_F, false));
        key_map.insert('g', (EvdevKey::KEY_G, false));
        key_map.insert('h', (EvdevKey::KEY_H, false));
        key_map.insert('i', (EvdevKey::KEY_I, false));
        key_map.insert('j', (EvdevKey::KEY_J, false));
        key_map.insert('k', (EvdevKey::KEY_K, false));
        key_map.insert('l', (EvdevKey::KEY_L, false));
        key_map.insert('m', (EvdevKey::KEY_M, false));
        key_map.insert('n', (EvdevKey::KEY_N, false));
        key_map.insert('o', (EvdevKey::KEY_O, false));
        key_map.insert('p', (EvdevKey::KEY_P, false));
        key_map.insert('q', (EvdevKey::KEY_Q, false));
        key_map.insert('r', (EvdevKey::KEY_R, false));
        key_map.insert('s', (EvdevKey::KEY_S, false));
        key_map.insert('t', (EvdevKey::KEY_T, false));
        key_map.insert('u', (EvdevKey::KEY_U, false));
        key_map.insert('v', (EvdevKey::KEY_V, false));
        key_map.insert('w', (EvdevKey::KEY_W, false));
        key_map.insert('x', (EvdevKey::KEY_X, false));
        key_map.insert('y', (EvdevKey::KEY_Y, false));
        key_map.insert('z', (EvdevKey::KEY_Z, false));

        // Uppercase letters
        key_map.insert('A', (EvdevKey::KEY_A, true));
        key_map.insert('B', (EvdevKey::KEY_B, true));
        key_map.insert('C', (EvdevKey::KEY_C, true));
        key_map.insert('D', (EvdevKey::KEY_D, true));
        key_map.insert('E', (EvdevKey::KEY_E, true));
        key_map.insert('F', (EvdevKey::KEY_F, true));
        key_map.insert('G', (EvdevKey::KEY_G, true));
        key_map.insert('H', (EvdevKey::KEY_H, true));
        key_map.insert('I', (EvdevKey::KEY_I, true));
        key_map.insert('J', (EvdevKey::KEY_J, true));
        key_map.insert('K', (EvdevKey::KEY_K, true));
        key_map.insert('L', (EvdevKey::KEY_L, true));
        key_map.insert('M', (EvdevKey::KEY_M, true));
        key_map.insert('N', (EvdevKey::KEY_N, true));
        key_map.insert('O', (EvdevKey::KEY_O, true));
        key_map.insert('P', (EvdevKey::KEY_P, true));
        key_map.insert('Q', (EvdevKey::KEY_Q, true));
        key_map.insert('R', (EvdevKey::KEY_R, true));
        key_map.insert('S', (EvdevKey::KEY_S, true));
        key_map.insert('T', (EvdevKey::KEY_T, true));
        key_map.insert('U', (EvdevKey::KEY_U, true));
        key_map.insert('V', (EvdevKey::KEY_V, true));
        key_map.insert('W', (EvdevKey::KEY_W, true));
        key_map.insert('X', (EvdevKey::KEY_X, true));
        key_map.insert('Y', (EvdevKey::KEY_Y, true));
        key_map.insert('Z', (EvdevKey::KEY_Z, true));

        // Numbers
        key_map.insert('0', (EvdevKey::KEY_0, false));
        key_map.insert('1', (EvdevKey::KEY_1, false));
        key_map.insert('2', (EvdevKey::KEY_2, false));
        key_map.insert('3', (EvdevKey::KEY_3, false));
        key_map.insert('4', (EvdevKey::KEY_4, false));
        key_map.insert('5', (EvdevKey::KEY_5, false));
        key_map.insert('6', (EvdevKey::KEY_6, false));
        key_map.insert('7', (EvdevKey::KEY_7, false));
        key_map.insert('8', (EvdevKey::KEY_8, false));
        key_map.insert('9', (EvdevKey::KEY_9, false));

        // Special characters
        key_map.insert('!', (EvdevKey::KEY_1, true));
        key_map.insert('@', (EvdevKey::KEY_2, true));
        key_map.insert('#', (EvdevKey::KEY_3, true));
        key_map.insert('$', (EvdevKey::KEY_4, true));
        key_map.insert('%', (EvdevKey::KEY_5, true));
        key_map.insert('^', (EvdevKey::KEY_6, true));
        key_map.insert('&', (EvdevKey::KEY_7, true));
        key_map.insert('*', (EvdevKey::KEY_8, true));
        key_map.insert('(', (EvdevKey::KEY_9, true));
        key_map.insert(')', (EvdevKey::KEY_0, true));
        key_map.insert('_', (EvdevKey::KEY_MINUS, true));
        key_map.insert('+', (EvdevKey::KEY_EQUAL, true));
        key_map.insert('{', (EvdevKey::KEY_LEFTBRACE, true));
        key_map.insert('}', (EvdevKey::KEY_RIGHTBRACE, true));
        key_map.insert('|', (EvdevKey::KEY_BACKSLASH, true));
        key_map.insert(':', (EvdevKey::KEY_SEMICOLON, true));
        key_map.insert('"', (EvdevKey::KEY_APOSTROPHE, true));
        key_map.insert('<', (EvdevKey::KEY_COMMA, true));
        key_map.insert('>', (EvdevKey::KEY_DOT, true));
        key_map.insert('?', (EvdevKey::KEY_SLASH, true));
        key_map.insert('~', (EvdevKey::KEY_GRAVE, true));

        // Common punctuation
        key_map.insert('-', (EvdevKey::KEY_MINUS, false));
        key_map.insert('=', (EvdevKey::KEY_EQUAL, false));
        key_map.insert('[', (EvdevKey::KEY_LEFTBRACE, false));
        key_map.insert(']', (EvdevKey::KEY_RIGHTBRACE, false));
        key_map.insert('\\', (EvdevKey::KEY_BACKSLASH, false));
        key_map.insert(';', (EvdevKey::KEY_SEMICOLON, false));
        key_map.insert('\'', (EvdevKey::KEY_APOSTROPHE, false));
        key_map.insert(',', (EvdevKey::KEY_COMMA, false));
        key_map.insert('.', (EvdevKey::KEY_DOT, false));
        key_map.insert('/', (EvdevKey::KEY_SLASH, false));
        key_map.insert('`', (EvdevKey::KEY_GRAVE, false));

        // Whitespace
        key_map.insert(' ', (EvdevKey::KEY_SPACE, false));
        key_map.insert('\t', (EvdevKey::KEY_TAB, false));
        key_map.insert('\n', (EvdevKey::KEY_ENTER, false));

        // Action keys, such as backspace, escape, ctrl, alt
        key_map.insert('\x08', (EvdevKey::KEY_BACKSPACE, false));
        key_map.insert('\x1b', (EvdevKey::KEY_ESC, false));

        key_map
    }

    /// Replace the characters models like to emit (curly quotes, long dashes,
    /// bullets, arrows, accented letters) with plain ASCII the virtual keyboard
    /// can type. Anything else that is not ASCII is dropped.
    pub fn fold_to_ascii(input: &str) -> String {
        let mut out = String::with_capacity(input.len());
        for c in input.chars() {
            match c {
                '\u{2010}' | '\u{2011}' | '\u{2012}' | '\u{2013}' | '\u{2014}' | '\u{2015}' | '\u{2212}' => out.push('-'),
                '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{2032}' | '\u{02BC}' => out.push('\''),
                '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{2033}' => out.push('"'),
                '\u{2022}' | '\u{25CF}' | '\u{25E6}' | '\u{00B7}' | '\u{2023}' => out.push('-'),
                '\u{2192}' | '\u{27A1}' => out.push_str("->"),
                '\u{2190}' => out.push_str("<-"),
                '\u{2026}' => out.push_str("..."),
                '\u{00D7}' => out.push('x'),
                '\u{00A0}' | '\u{2009}' | '\u{200A}' | '\u{202F}' => out.push(' '),
                '\u{00B0}' => out.push_str(" deg"),
                '\u{00E0}'..='\u{00E5}' => out.push('a'),
                '\u{00E8}'..='\u{00EB}' => out.push('e'),
                '\u{00EC}'..='\u{00EF}' => out.push('i'),
                '\u{00F2}'..='\u{00F6}' => out.push('o'),
                '\u{00F9}'..='\u{00FC}' => out.push('u'),
                '\u{00C0}'..='\u{00C5}' => out.push('A'),
                '\u{00C8}'..='\u{00CB}' => out.push('E'),
                '\u{00CC}'..='\u{00CF}' => out.push('I'),
                '\u{00D2}'..='\u{00D6}' => out.push('O'),
                '\u{00D9}'..='\u{00DC}' => out.push('U'),
                '\u{00E7}' => out.push('c'),
                '\u{00C7}' => out.push('C'),
                '\u{00F1}' => out.push('n'),
                '\u{00D1}' => out.push('N'),
                '\u{00DF}' => out.push_str("ss"),
                '\r' => {}
                c if c.is_ascii() => out.push(c),
                other => debug!("Dropping character with no ASCII equivalent: {:?}", other),
            }
        }
        out
    }

    /// Fold a model reply to ASCII, trim it, and cap its length so it types
    /// cleanly and quickly. At about 100 characters per second, 900 characters
    /// keeps a reply under ten seconds of typing.
    pub fn prepare_reply(text: &str, max_chars: usize) -> String {
        let folded = Self::fold_to_ascii(text);
        let trimmed = folded.trim();
        let capped: String = if trimmed.chars().count() > max_chars {
            let mut cut: String = trimmed.chars().take(max_chars.saturating_sub(3)).collect();
            cut.push_str("...");
            cut
        } else {
            trimmed.to_string()
        };
        if capped != text.trim() {
            log::warn!("Reply folded or capped: {} chars in, {} chars typed", text.chars().count(), capped.chars().count());
        }
        capped
    }

    pub fn key_down(&mut self, key: EvdevKey) -> Result<()> {
        if let Some(device) = &mut self.device {
            device.emit(&[(InputEvent::new(EvdevEventType::KEY.0, key.code(), 1))])?;
            device.emit(&[InputEvent::new(EvdevEventType::SYNCHRONIZATION.0, 0, 0)])?;
            thread::sleep(time::Duration::from_millis(1));
        }
        Ok(())
    }

    pub fn key_up(&mut self, key: EvdevKey) -> Result<()> {
        if let Some(device) = &mut self.device {
            device.emit(&[(InputEvent::new(EvdevEventType::KEY.0, key.code(), 0))])?;
            device.emit(&[InputEvent::new(EvdevEventType::SYNCHRONIZATION.0, 0, 0)])?;
            thread::sleep(time::Duration::from_millis(1));
        }
        Ok(())
    }

    pub fn string_to_keypresses(&mut self, input: &str) -> Result<()> {
        let input = Self::fold_to_ascii(input);
        if let Some(device) = &mut self.device {
            // make sure we are synced before we start; this might be paranoia
            device.emit(&[InputEvent::new(EvdevEventType::SYNCHRONIZATION.0, 0, 0)])?;
            thread::sleep(time::Duration::from_millis(10));

            for c in input.chars() {
                if let Some(&(key, shift)) = self.key_map.get(&c) {
                    if shift {
                        // Press Shift
                        device.emit(&[InputEvent::new(EvdevEventType::KEY.0, EvdevKey::KEY_LEFTSHIFT.code(), 1)])?;
                    }

                    // Press key
                    device.emit(&[InputEvent::new(EvdevEventType::KEY.0, key.code(), 1)])?;

                    // Release key
                    device.emit(&[InputEvent::new(EvdevEventType::KEY.0, key.code(), 0)])?;

                    if shift {
                        // Release Shift
                        device.emit(&[InputEvent::new(EvdevEventType::KEY.0, EvdevKey::KEY_LEFTSHIFT.code(), 0)])?;
                    }

                    // Sync event
                    device.emit(&[InputEvent::new(EvdevEventType::SYNCHRONIZATION.0, 0, 0)])?;
                    thread::sleep(time::Duration::from_millis(10));
                } else {
                    debug!("Skipping character with no key mapping: {:?}", c);
                }
            }
        }
        Ok(())
    }

    fn key_cmd(&mut self, button: &str, shift: bool) -> Result<()> {
        self.key_down(EvdevKey::KEY_LEFTCTRL)?;
        if shift {
            self.key_down(EvdevKey::KEY_LEFTSHIFT)?;
        }
        self.string_to_keypresses(button)?;
        if shift {
            self.key_up(EvdevKey::KEY_LEFTSHIFT)?;
        }
        self.key_up(EvdevKey::KEY_LEFTCTRL)?;
        Ok(())
    }

    pub fn key_cmd_title(&mut self) -> Result<()> {
        self.key_cmd("1", false)?;
        Ok(())
    }

    pub fn key_cmd_subheading(&mut self) -> Result<()> {
        self.key_cmd("2", false)?;
        Ok(())
    }

    pub fn key_cmd_body(&mut self) -> Result<()> {
        self.key_cmd("3", false)?;
        Ok(())
    }

    pub fn key_cmd_bullet(&mut self) -> Result<()> {
        self.key_cmd("4", false)?;
        Ok(())
    }

    pub fn progress(&mut self, note: &str) -> Result<()> {
        if self.no_draw_progress {
            return Ok(());
        }
        self.string_to_keypresses(note)?;
        // Count what was actually typed (after the ASCII fold) so progress_end
        // erases exactly that many characters.
        self.progress_count += Self::fold_to_ascii(note).chars().count() as u32;
        Ok(())
    }

    pub fn progress_end(&mut self) -> Result<()> {
        if self.no_draw_progress {
            return Ok(());
        }
        // Send a backspace for each progress
        for _ in 0..self.progress_count {
            self.string_to_keypresses("\x08")?;
        }
        self.progress_count = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Keyboard;

    #[test]
    fn folds_common_model_output_to_ascii() {
        assert_eq!(Keyboard::fold_to_ascii("Key \u{2014} \u{201C}fit\u{201D} \u{2192} ask \u{2026}"), "Key - \"fit\" -> ask ...");
        assert_eq!(Keyboard::fold_to_ascii("\u{2022} caf\u{00E9} \u{2018}ok\u{2019}"), "- cafe 'ok'");
        assert_eq!(Keyboard::fold_to_ascii("plain ASCII stays\n"), "plain ASCII stays\n");
        assert_eq!(Keyboard::fold_to_ascii("emoji \u{1F600} gone"), "emoji  gone");
        assert_eq!(Keyboard::fold_to_ascii("crlf\r\nline"), "crlf\nline");
    }

    #[test]
    fn prepare_reply_trims_and_caps() {
        assert_eq!(Keyboard::prepare_reply("  hello \u{2014} there  ", 900), "hello - there");
        let long = "x".repeat(1000);
        let capped = Keyboard::prepare_reply(&long, 900);
        assert_eq!(capped.chars().count(), 900);
        assert!(capped.ends_with("..."));
    }
}
