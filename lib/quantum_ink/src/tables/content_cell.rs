/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
  Part of the Quantum OS Project

Copyright 2023 Gavin Kellam

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

use std::fmt::{Debug, Display, Formatter};

pub struct ContentCell {
    pub(crate) content: String,
    pub(crate) required_spacing_x: usize,
    pub(crate) required_spacing_y: usize
}

impl ContentCell {
    pub fn new(content_string: String) -> Self {
        let mut longest_part = 0;
        for part in content_string.split('\n') {
            let this_part_len = part.len();

            if this_part_len > longest_part {
                longest_part = this_part_len;
            }
        }

        let amount_of_newline = content_string.matches('\n').count() + 1;

        Self {
            content: content_string,
            required_spacing_x: longest_part,
            required_spacing_y: amount_of_newline
        }
    }

    pub fn from_formattable_display<Type>(value: Type) -> Self
        where Type: Display {
        Self::new(value.to_string())
    }

    pub fn from_formattable_debug<Type>(value: Type) -> Self
        where Type: Debug {
        let content = format!("{:#?}", value);

        Self::new(content)
    }
}

impl Display for ContentCell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}

#[cfg(test)]
mod test {
    use crate::tables::content_cell::ContentCell;

    #[test]
    fn from_debuggable() {
        let debuggable = vec![0, 1, 2, 3, 4];

        let cell = ContentCell::from_formattable_debug(debuggable.clone());

        let cell_formatting = cell.content;

        assert_eq!(cell_formatting, format!("{:#?}", debuggable));
    }

    #[test]
    fn from_displayable() {
        let displayable = 10;

        let cell = ContentCell::from_formattable_display(displayable);

        let cell_formatting = cell.content;

        assert_eq!(cell_formatting, format!("{}", displayable));
    }

    #[test]
    fn content_spacing()  {
        let m_test_content = "Foo\nBar\nBaz Baz".to_string();
        let cell = ContentCell::new(m_test_content.clone());

        let found_lines = cell.required_spacing_y;
        let found_chars = cell.required_spacing_x;

        assert_eq!(found_lines, 3, "There was not 3 lines in the string {:#?}", m_test_content);
        assert_eq!(found_chars, 7, "There was not 7 max horizontal chars in the string {:#?}", m_test_content);
    }
}