/*
  ____                 __               __                __
 / __ \__ _____ ____  / /___ ____ _    / /  ___  ___ ____/ /__ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ _ \/ _ `/ _  / -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/\___/\_,_/\_,_/\__/_/
    Part of the Quantum OS Project

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

use core::fmt::{Debug, Display, Result, Write};

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum AsciiForeground {
    None = u8::MAX,
    Black = 30,
    Red = 31,
    Green = 32,
    Yellow = 33,
    Blue = 34,
    Magenta = 35,
    Cyan = 36,
    White = 37,
    Default = 39,
    Reset = 0,
    BrightBlack = 90,
    BrightRed = 91,
    BrightGreen = 92,
    BrightYellow = 93,
    BrightBlue = 94,
    BrightMagenta = 95,
    BrightCyan = 96,
    BrightWhite = 97,
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum AsciiBackground {
    None = u8::MAX,
    OnBlack = 40,
    OnRed = 41,
    OnGreen = 42,
    OnYellow = 43,
    OnBlue = 44,
    OnMagenta = 45,
    OnCyan = 46,
    OnWhite = 47,
    OnDefault = 49,
    OnBrightBlack = 100,
    OnBrightRed = 101,
    OnBrightGreen = 102,
    OnBrightYellow = 103,
    OnBrightBlue = 104,
    OnBrightMagenta = 105,
    OnBrightCyan = 106,
    OnBrightWhite = 107,
}

#[derive(Clone, Copy)]
pub enum AsciiMod {
    None,
    Bold,
    Dim,
    Italic,
    Underline,
    Blinking,
    Inverse,
    Hidden,
    Strikethrough,
}

// pub struct Color<'a, Inner>(&'a Inner);

// impl<'a, Inner: Debug> Color<'a, Inner> {
//     pub fn dbg(inner: &'a Inner) -> Self {
//         Self(inner)
//     }
// }

// impl<'a, Inner: Display> Color<'a, Inner> {
//     pub fn new(inner: &'a Inner) -> Self {
//         Self(inner)
//     }
// }

// pub trait TermColorize<'a>: Display + Sized {
//     fn black(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).foreground(AsciiForeground::Black)
//     }
//     fn red(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).foreground(AsciiForeground::Red)
//     }
//     fn green(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).foreground(AsciiForeground::Green)
//     }
//     fn yellow(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).foreground(AsciiForeground::Yellow)
//     }
//     fn blue(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).foreground(AsciiForeground::Blue)
//     }
//     fn magenta(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).foreground(AsciiForeground::Magenta)
//     }
//     fn cyan(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).foreground(AsciiForeground::Cyan)
//     }
//     fn white(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).foreground(AsciiForeground::White)
//     }
//     fn default(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).foreground(AsciiForeground::Default)
//     }

//     fn bright_black(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).foreground(AsciiForeground::BrightBlack)
//     }
//     fn bright_red(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).foreground(AsciiForeground::BrightRed)
//     }
//     fn bright_green(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).foreground(AsciiForeground::BrightGreen)
//     }
//     fn bright_yellow(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).foreground(AsciiForeground::BrightYellow)
//     }
//     fn bright_blue(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).foreground(AsciiForeground::BrightBlue)
//     }
//     fn bright_magenta(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).foreground(AsciiForeground::BrightMagenta)
//     }
//     fn bright_cyan(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).foreground(AsciiForeground::BrightCyan)
//     }
//     fn bright_white(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).foreground(AsciiForeground::BrightWhite)
//     }

//     fn on_black(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).background(AsciiBackground::OnBlack)
//     }
//     fn on_red(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).background(AsciiBackground::OnRed)
//     }
//     fn on_green(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).background(AsciiBackground::OnGreen)
//     }
//     fn on_yellow(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).background(AsciiBackground::OnYellow)
//     }
//     fn on_blue(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).background(AsciiBackground::OnBlue)
//     }
//     fn on_magenta(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).background(AsciiBackground::OnMagenta)
//     }
//     fn on_cyan(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).background(AsciiBackground::OnCyan)
//     }
//     fn on_white(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).background(AsciiBackground::OnWhite)
//     }
//     fn on_default(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).background(AsciiBackground::OnDefault)
//     }

//     fn on_bright_black(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).background(AsciiBackground::OnBrightBlack)
//     }
//     fn on_bright_red(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).background(AsciiBackground::OnBrightRed)
//     }
//     fn on_bright_green(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).background(AsciiBackground::OnBrightGreen)
//     }
//     fn on_bright_yellow(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).background(AsciiBackground::OnBrightYellow)
//     }
//     fn on_bright_blue(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).background(AsciiBackground::OnBrightBlue)
//     }
//     fn on_bright_magenta(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).background(AsciiBackground::OnBrightMagenta)
//     }
//     fn on_bright_cyan(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).background(AsciiBackground::OnBrightCyan)
//     }
//     fn on_bright_white(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).background(AsciiBackground::OnBrightWhite)
//     }

//     fn reset(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).foreground(AsciiForeground::Reset)
//     }
//     fn bold(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).mode(AsciiMod::Bold)
//     }
//     fn dimmed(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).mode(AsciiMod::Dim)
//     }
//     fn italic(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).mode(AsciiMod::Italic)
//     }
//     fn underline(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).mode(AsciiMod::Underline)
//     }
//     fn blinking(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).mode(AsciiMod::Blinking)
//     }
//     fn inverse(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).mode(AsciiMod::Inverse)
//     }
//     fn hidden(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).mode(AsciiMod::Hidden)
//     }
//     fn strikethrough(&'a self) -> Colorized<'a, Self> {
//         Colorized::new(self).mode(AsciiMod::Strikethrough)
//     }
// }

pub struct Colorized<'a, T> {
    inner: &'a T,

    modify: AsciiMod,
    fore: AsciiForeground,
    back: AsciiBackground,
}

impl<'a, T> Colorized<'a, T> {
    pub fn new(inner: &'a T) -> Self {
        Self {
            inner,
            modify: AsciiMod::None,
            fore: AsciiForeground::None,
            back: AsciiBackground::None,
        }
    }

    pub fn mode(self, modify: AsciiMod) -> Self {
        Self { modify, ..self }
    }

    pub fn foreground(self, fore: AsciiForeground) -> Self {
        Self { fore, ..self }
    }

    pub fn background(self, back: AsciiBackground) -> Self {
        Self { back, ..self }
    }
}

impl<'a, T> Display for Colorized<'a, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match (self.modify, self.fore, self.back) {
            (AsciiMod::None, AsciiForeground::None, AsciiBackground::None) => {
                f.write_fmt(format_args!("{}", self.inner))?;
                return Ok(());
            }
            _ => (),
        }

        // Set graphics mode
        // f.write_char(0x1b as char)?;
        f.write_char('[')?;
        match self.modify {
            AsciiMod::None => (),
            AsciiMod::Bold => f.write_str("1")?,
            AsciiMod::Dim => f.write_str("2")?,
            AsciiMod::Italic => f.write_str("3")?,
            AsciiMod::Underline => f.write_str("4")?,
            AsciiMod::Blinking => f.write_str("5")?,
            AsciiMod::Inverse => f.write_str("7")?,
            AsciiMod::Hidden => f.write_str("8")?,
            AsciiMod::Strikethrough => f.write_str("9")?,
        }

        match self.fore {
            AsciiForeground::None => (),
            m => {
                if !matches!(self.modify, AsciiMod::None) {
                    f.write_char(';')?;
                }

                f.write_fmt(format_args!("{}", m as u8))?;
            }
        }

        match self.back {
            AsciiBackground::None => (),
            m => {
                if !matches!(self.fore, AsciiForeground::None) {
                    f.write_char(';')?;
                }

                f.write_fmt(format_args!("{}", m as u8))?;
            }
        }

        f.write_char('m')?;
        self.inner.fmt(f)?;

        // reset
        // f.write_char(0x1b as char)?;
        f.write_char('[')?;
        match self.modify {
            AsciiMod::None => f.write_str("0m"),
            AsciiMod::Bold => f.write_str("22;0m"),
            AsciiMod::Dim => f.write_str("22;0m"),
            AsciiMod::Italic => f.write_str("23;0m"),
            AsciiMod::Underline => f.write_str("24;0m"),
            AsciiMod::Blinking => f.write_str("25;0m"),
            AsciiMod::Inverse => f.write_str("27;0m"),
            AsciiMod::Hidden => f.write_str("28;0m"),
            AsciiMod::Strikethrough => f.write_str("29;0m"),
        }
    }
}
