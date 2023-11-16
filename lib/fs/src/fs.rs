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
*
*/

use qk_alloc::string::String;
use qk_alloc::vec::Vec;

pub struct Path(String);

impl From<&str> for Path {
    fn from(value: &str) -> Self {
        Path(String::from(value))
    }
}

impl From<String> for Path {
    fn from(value: String) -> Self {
        Path(value)
    }
}

impl Path {
    /// # Truncate Path
    /// Removes all the unneeded path items in the path. For example if you have a path that goes
    /// back two directories, and then goes up two directories. You don't need to go down, and back
    /// up.
    ///
    /// ### Example
    /// 1.  Path = "/./././././"
    ///     desolved = "/."
    ///
    /// 2.  Path = "./////././///.."
    ///     desolved = ".."
    ///
    /// 3.  Path = "/Node/../Node/../."
    ///     desolved = "/."
    ///
    pub fn truncate_path(self) -> Path {
        let mut final_string: String = self
            // Get the children of the path
            // Example path = "/home/user/someone"
            //  1. 'home'
            //  2. 'user'
            //  3. 'someone'
            .children()
            .into_iter()
            // Remove all '.'
            .filter(|child| !child.contains('.'))
            .collect::<Vec<&str>>()
            // Remove all "/path/../path/"
            .windows(3)
            .filter(|children| {
                let mut child_iter = children.iter();

                let fist = child_iter.next();
                let secd = child_iter.next();
                let last = child_iter.next();

                !fist
                    .and_then(|first| {
                        Some(
                            secd.and_then(|second| Some(second == &".."))?
                                && last.and_then(|last| Some(first == last))?,
                        )
                    })
                    .unwrap_or(false)
            })
            // Since its windows, we only care about the first element
            .take(1)
            .flatten()
            // Add the '/' back for each of the children
            .fold(
                self.0
                    .starts_with('/')
                    .then(|| String::from("/"))
                    .unwrap_or(String::new()),
                |mut acc, val| {
                    acc.push_str(val);
                    acc.push_str("/");

                    acc
                },
            );

        if self.0.starts_with("/") {
            // FIXME: string should have a insert method!
            final_string = String::from("/") + final_string;
        }

        if self.0.ends_with("/") && !self.0.ends_with("//") {
            final_string.push('/');
        }

        if self.0.ends_with(".") {
            final_string.push_str(".")
        }

        Path(final_string)
    }

    pub fn children<'a>(&'a self) -> Vec<&'a str> {
        self.0
            .as_str()
            .split('/')
            .filter(|child| !child.contains('/'))
            .collect()
    }
}

pub struct Vfs {}

impl Vfs {}

#[cfg(test)]
mod test {}
