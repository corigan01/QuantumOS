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

#[warn(unstable_features)]
#[macro_export]
macro_rules! enum_flags {
    () => {};

    (
        $(#[$m:meta])*
        $visa:vis enum $name:ident($itr:ident) {
            $($val:ident $(= $epr:expr)?),*
        }

    ) => {
        $(#[$m])*
        $visa enum $name {
            $($val $(= $epr)?),*
        }

        pub struct $itr {
            flags: u64
        }

        #[allow(dead_code)]
        impl $itr {
            $visa fn new() -> Self {
                Self {
                    flags: 0
                }
            }

            fn get_this_enum_id(flag: $name) -> usize {
                let mut id: Option<usize> = None;
                for (key, value) in $name::ITEMS.iter().enumerate() {
                    if *value == flag {
                        id = Some(key);
                    }
                }

                id.expect("Internal enum flags error!")
            }

            /// # Safety
            /// Will not check your value and ensure its valid, it only
            /// puts the value into its own internal register!
            $visa unsafe fn unsafe_from_value(value: usize) -> $itr {
                Self {
                    flags: value as u64
                }
            }

            $visa fn set_flag(&mut self, flag: $name) {
                let id = Self::get_this_enum_id(flag);

                self.flags.set_bit(id as u8, true);
            }

            $visa fn remove_flag(&mut self, flag: $name) {
                let id = Self::get_this_enum_id(flag);

                self.flags.set_bit(id as u8, false);
            }

            $visa fn is_flag_active(&self, flag: $name) -> bool {
                let id = Self::get_this_enum_id(flag);

                self.flags.get_bit(id as u8)
            }
        }

        impl $name {
            const ITEMS: [$name; <[$name]>::len(&[$($name::$val),*])] =
                [$($name::$val),*];


        }

    }
}
