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

use core::mem::MaybeUninit;

pub struct PossiblyUninit<Type> {
    data: MaybeUninit<Type>,
    func: MaybeUninit<fn() -> Type>,
    has_func: bool,
    has_been_init: bool
}

impl<Type> PossiblyUninit<Type> {
    pub const fn new() -> Self {
        Self {
            data: MaybeUninit::uninit(),
            func: MaybeUninit::uninit(),
            has_func: false,
            has_been_init: false
        }
    }

    pub const fn new_lazy(fnc: fn() -> Type) -> Self {
        Self {
            data: MaybeUninit::uninit(),
            func: MaybeUninit::new(fnc),
            has_func: true,
            has_been_init: false
        }
    }

    fn auto_init_if_avl(&mut self) {
        if !self.has_func || self.has_been_init {
            return;
        }

        let func = unsafe { self.func.assume_init_ref() };
        self.data = MaybeUninit::new(func());
        self.has_been_init = true;
    }

    pub const fn from(t: Type) -> Self {
        Self {
            data: MaybeUninit::new(t),
            func: MaybeUninit::uninit(),
            has_func: false,
            has_been_init: true
        }
    }

    fn check_conditions(&mut self) -> Option<()> {
        self.auto_init_if_avl();

        if !self.has_been_init {
            None
        } else {
            Some(())
        }
    }

    pub fn get_ref(&mut self) -> Option<&Type> {
        self.check_conditions()?;

        Some(unsafe { self.data.assume_init_ref() })
    }

    pub fn get_mut_ref(&mut self) -> Option<&mut Type> {
        self.check_conditions()?;

        Some(unsafe { self.data.assume_init_mut() })
    }

    pub fn set(&mut self, t: Type) {
        self.data.write(t);
        self.has_been_init = true;
    }
}
