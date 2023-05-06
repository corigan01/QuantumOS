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

use std::cmp::max;
use std::fmt::{Display, Formatter};
use crate::tables::content_cell::ContentCell;

pub struct Table {
    value_cells: Vec<Vec<ContentCell>>,
}

impl Table {
    pub fn new(values: Vec<Vec<ContentCell>>) -> Self {
        Self {
            value_cells: values
        }
    }

    pub fn spacing_for_x(&self) -> usize {
        let mut spacing = 0;
        for lines in self.value_cells.iter() {
            for cell in lines {
                spacing = max(spacing, cell.required_spacing_x);
            }
        }

        spacing
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for lines in self.value_cells.iter() {
            write!(f, "| ")?;
            for cells in lines {
                write!(f, "{} | ", cells)?;
            }
            writeln!(f, "")?;
        }

        Ok(())
    }
}



#[cfg(test)]
mod test {
    use crate::tables::content_cell::ContentCell;
    use crate::tables::table::Table;

    #[test]
    fn test() {
        let cells = vec![
            vec![ContentCell::new("Value".into()), ContentCell::new("Key".into())],
            vec![ContentCell::from_formattable_display(0), ContentCell::from_formattable_display(10)]
        ];

        let table = Table::new(cells);

        assert_eq!(false, true, "\n\n{}", table);
    }
}