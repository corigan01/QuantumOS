/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
  Part of the Quantum OS Kernel

Copyright 2024 Gavin Kellam

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
associated documentation files (the "Software"), to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge, publish, distribute,
sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial
portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT
OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/

/// # Key Point Vector
/// This stores the point to which the key is on the keyboard. Layout for Enlish QWERTY is below
/// for example.
///
/// ## Example Layout
/// ```text
///   (0, 0)
///      V
///     ESC F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 F11 F12 (PRINT SCREEN) (SCROLL LOCK) (PAUSE)
///      `  1  2  3  4  5  6  7  8  9  0  -  =  (BACKSPACE) (INSERT) (HOME) (PAGE UP) (NUM LOCK)  /  *  -
///     TAB  Q  W  E  R  T  Y  U  I  O  P  [  ]  \  (DELETE) (END) (PAGE DOWN)                    7  8  9  +
///     (CAP L) A  S  D  F  G  H  J  K  L  ;  '  (ENTER)                                          4  5  6  
///     SHIFT    Z  X  C  V  B  N  M  ,  .  /  (SHIFT)                  (UP ARROW)                1  2  3 (ENTER)
///     CTRL (LOGO) (ALT) (SPACE BAR) (ALT) (LOGO) (MENU) (CTRL)  (<-) (DOWN ARROW) (->)          0  .
/// ```
pub struct KeyPointVector(u16);

impl KeyPointVector {
    pub const fn new(x: u8, y: u8) -> Self {
        Self((y as u16) << 8 | (x as u16))
    }

    pub const fn into_dim(&self) -> (u8, u8) {
        let x = (self.0 & 0xFF) as u8;
        let y = ((self.0 & 0xFF00) >> 8) as u8;

        (x, y)
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyboardLayout {
    EnglishQ,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyState {
    Pressed,
    Released,
    Repeted,
}

pub struct KeyboardPacket {
    pub layout: KeyboardLayout,
    pub pos: KeyPointVector,
    pub state: KeyState,
}

fn scan_code_converter_1(scan_code: u64, layout: KeyboardLayout) -> KeyboardPacket {
    assert!(
        layout == KeyboardLayout::EnglishQ,
        "Only English QWERTY is supported"
    );

    // FIXME: Maybe we should use recursive array lookup? This is fine for now...
    let (pos, state) = match scan_code {
        0x01 => (KeyPointVector::new(0, 0), KeyState::Pressed), // ESC
        0x1C => (KeyPointVector::new(12, 3), KeyState::Pressed), // ENTER
        0x1D => (KeyPointVector::new(0, 5), KeyState::Pressed), // L-CTRL
        0x2A => (KeyPointVector::new(0, 4), KeyState::Pressed), // L-SHIFT
        0x2B => (KeyPointVector::new(13, 2), KeyState::Pressed), // \
        0x38 => (KeyPointVector::new(2, 5), KeyState::Pressed), // L-ALT
        0x39 => (KeyPointVector::new(3, 5), KeyState::Pressed), // Space
        0x3A => (KeyPointVector::new(0, 3), KeyState::Pressed), // Caps Lock
        0x45 => (KeyPointVector::new(17, 1), KeyState::Pressed), // Number Lock
        0x46 => (KeyPointVector::new(14, 0), KeyState::Pressed), // Scroll Lock
        0x47 => (KeyPointVector::new(17, 2), KeyState::Pressed), // Keypad 7
        0x48 => (KeyPointVector::new(18, 2), KeyState::Pressed), // Keypad 8
        0x49 => (KeyPointVector::new(19, 2), KeyState::Pressed), // Keypad 9
        0x4A => (KeyPointVector::new(20, 1), KeyState::Pressed), // Keypad -
        0x4B => (KeyPointVector::new(13, 3), KeyState::Pressed), // Keypad 4
        0x4C => (KeyPointVector::new(14, 3), KeyState::Pressed), // Keypad 5
        0x4D => (KeyPointVector::new(15, 3), KeyState::Pressed), // Keypad 6
        0x4E => (KeyPointVector::new(20, 2), KeyState::Pressed), // Keypad +
        0x4F => (KeyPointVector::new(13, 4), KeyState::Pressed), // Keypad 1
        0x50 => (KeyPointVector::new(14, 4), KeyState::Pressed), // Keypad 2
        0x51 => (KeyPointVector::new(15, 4), KeyState::Pressed), // Keypad 3
        0x52 => (KeyPointVector::new(11, 5), KeyState::Pressed), // Keypad 0
        0x53 => (KeyPointVector::new(12, 5), KeyState::Pressed), // Keypad .
        0x57 => (KeyPointVector::new(11, 0), KeyState::Pressed), // F11
        0x58 => (KeyPointVector::new(12, 0), KeyState::Pressed), // F12

        n @ 0x02..=0x0E => (KeyPointVector::new(n as u8 - 1, 1), KeyState::Pressed), // 0-9 - = (BACKSPACE)
        n @ 0x0F..=0x1B => (KeyPointVector::new((n - 0x0F) as u8, 2), KeyState::Pressed), // (TAB) Q-P [ ]
        n @ 0x1E..=0x28 => (
            KeyPointVector::new((n - 0x1E) as u8 + 1, 3),
            KeyState::Pressed,
        ), // A-L ; '
        n @ 0x2C..=0x35 => (
            KeyPointVector::new((n - 0x2C) as u8 + 1, 4),
            KeyState::Pressed,
        ), // Z-M , . /
        n @ 0x3B..=0x44 => (
            KeyPointVector::new((n - 0x3B) as u8 + 1, 5),
            KeyState::Pressed,
        ), // F1-F10

        0x81 => (KeyPointVector::new(0, 0), KeyState::Released), // ESC
        0x9C => (KeyPointVector::new(12, 3), KeyState::Released), // ENTER
        0x9D => (KeyPointVector::new(0, 5), KeyState::Released), // L-CTRL
        0xAA => (KeyPointVector::new(0, 4), KeyState::Released), // L-SHIFT
        0xAB => (KeyPointVector::new(13, 2), KeyState::Released), // \
        0xB8 => (KeyPointVector::new(2, 5), KeyState::Released), // L-ALT
        0xB9 => (KeyPointVector::new(3, 5), KeyState::Released), // Space
        0xBA => (KeyPointVector::new(0, 3), KeyState::Released), // Caps Lock
        0xC5 => (KeyPointVector::new(17, 1), KeyState::Released), // Number Lock
        0xC6 => (KeyPointVector::new(14, 0), KeyState::Released), // Scroll Lock
        0xC7 => (KeyPointVector::new(17, 2), KeyState::Released), // Keypad 7
        0xC8 => (KeyPointVector::new(18, 2), KeyState::Released), // Keypad 8
        0xC9 => (KeyPointVector::new(19, 2), KeyState::Released), // Keypad 9
        0xCA => (KeyPointVector::new(20, 1), KeyState::Released), // Keypad -
        0xCB => (KeyPointVector::new(13, 3), KeyState::Released), // Keypad 4
        0xCC => (KeyPointVector::new(14, 3), KeyState::Released), // Keypad 5
        0xCD => (KeyPointVector::new(15, 3), KeyState::Released), // Keypad 6
        0xCE => (KeyPointVector::new(20, 2), KeyState::Released), // Keypad +
        0xCF => (KeyPointVector::new(13, 4), KeyState::Released), // Keypad 1
        0xD0 => (KeyPointVector::new(14, 4), KeyState::Released), // Keypad 2
        0xD1 => (KeyPointVector::new(15, 4), KeyState::Released), // Keypad 3
        0xD2 => (KeyPointVector::new(11, 5), KeyState::Released), // Keypad 0
        0xD3 => (KeyPointVector::new(12, 5), KeyState::Released), // Keypad .
        0xD7 => (KeyPointVector::new(11, 0), KeyState::Released), // F11
        0xD8 => (KeyPointVector::new(12, 0), KeyState::Released), // F12

        n @ 0x82..=0x8E => (KeyPointVector::new(n as u8 - 1, 1), KeyState::Released), // 0-9 - = (BACKSPACE)
        n @ 0x8F..=0x9B => (KeyPointVector::new((n - 0x0F) as u8, 2), KeyState::Released), // (TAB) Q-P [ ]
        n @ 0x9E..=0xA8 => (
            KeyPointVector::new((n - 0x1E) as u8 + 1, 3),
            KeyState::Released,
        ), // A-L ; '
        n @ 0xAC..=0xB5 => (
            KeyPointVector::new((n - 0x2C) as u8 + 1, 4),
            KeyState::Released,
        ), // Z-M , . /
        n @ 0xBB..=0xC4 => (
            KeyPointVector::new((n - 0x3B) as u8 + 1, 5),
            KeyState::Released,
        ), // F1-F10

        // FIXME: Add the remaining codes (https://wiki.osdev.org/PS/2_Keyboard)
        _ => todo!("Unreconized Scan Code 0x{scan_code:X} from scancode converter 1"),
    };

    KeyboardPacket { layout, pos, state }
}
