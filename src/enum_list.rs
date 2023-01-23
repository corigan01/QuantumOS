/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
  Part of the Quantum OS Kernel

Copyright 2022 Gavin Kellam

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

#[macro_export]
macro_rules! enum_list {
    () => {};

    (
        $(#[$m:meta])*
        $visa:vis enum $name:ident {
            $($val:ident $(= $epr:expr)?),*
        }

    ) => {

        $(#[$m])*
        $visa enum $name {
            $($val $(= $epr)?),*
        }

        impl Iterator for $name {
            type Item = $name

            fn next(&mut self) -> Option<Self::Item>{
                for i in 0..($name::ITEMS.len() - 1) {
                    let val = $name::ITEMS[i];

                    if *self == val {
                        return Some($name::ITEMS[i+1]);
                    }
                }

                None
            }
        }

        impl $name {
            const ITEMS: [$name; <[$name]>::len(&[$($name::$val),*])] =
                [$($name::$val),*];
        }

    }
}