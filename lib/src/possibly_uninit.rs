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

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct PossiblyUnit<Type> {
    data: Option<Type>,
}

impl<Type> PossiblyUnit<Type> {
    pub fn new() -> Self {
        Self { data: None }
    }

    pub fn from_value(value: Type) -> Self {
        Self { data: Some(value) }
    }

    pub fn to_ref_value<'a>(&'a self) -> Option<&'a Type> {
        self.data.as_ref()
    }

    pub fn to_mut_ref_value<'a>(&'a mut self) -> Option<&'a mut Type> {
        self.data.as_mut()
    }

    pub fn consume_type(self) -> Option<Type> {
        self.data
    }
}
