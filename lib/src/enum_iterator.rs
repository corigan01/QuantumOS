/*
  ____                 __               __                __
 / __ \__ _____ ____  / /___ ____ _    / /  ___  ___ ____/ /__ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ _ \/ _ `/ _  / -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/\___/\_,_/\_,_/\__/_/
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

pub trait EnumIntoIterator {
    type IntoIter;

    fn iter() -> Self::IntoIter;
}

/**!
# enum iterator

enum_iterator! is used to add iterators to your enums without having to construct large
arrays that hold each element of the enum. This is accomplished by adding an iterator type
to your enum that contains the auto-generated array structure made at compile time from your
enum. This is then used by the user with the `::iter()` type given by `EnumIntoIterator`.

### Example Usage
```rust
use quantum_os::enum_iterator;
use quantum_os::enum_iterator::EnumIntoIterator;

enum_iterator! {
    // The extra () in `MyEnum` is to define the Iterator for your enum!
    // This is due to Rust's hygiene that makes it such that macros cannot construct new types
    // with concat_ident!(). Note: `concat_ident!()` is a nightly feature and was emitted
    pub enum MyEnum(MyEnumIter) {
        SomeVal,
        SomeOtherVal,
        MoreOptions
    }
}

fn main() {
    println!("All of the possible values in MyEnum are: ");

    for i in MyEnum::iter() {
        println!("\t{}", i);
    }
}
```

 */

/// # enum_iterator! {SampleEnum(SampleEnumIter) {...} }
///
/// We take in two arguments here, as one is the struct for which,
/// we would like to iterate, and the other one is the iterator that
/// stores the index at which we are iterating.
///
/// First thing that this macro does, is that it collects all the info
/// about the enum you provided. Then it breaks it up into the following:
///     * $m -- The meta type (all the #derive and #repr stuff)
///     * $visa -- The visibility of your enum (public or private)
///     * $name -- The name of your enum
///     * $itr -- The iterator type
///     * All the values you provided in your enum:
///         * $val -- the values
///         * $epr -- anything after the equal sign
///
/// The following work has to still be done with these things however:
///     * Make the actual enum you provided
///     * Make the iterator struct (with index)
///     * Make a `impl` for your enum to return the newly created iterator type
///     * Make a huge array containing all values in your enum
///     * make the iterator access this huge array of values
///
///
#[macro_export]
macro_rules! enum_iterator {
    () => {};

    (
        $(#[$m:meta])*
        $visa:vis enum $name:ident($itr:ident) {
            $($val:ident $(= $epr:expr)?),*
        }

    ) => {

        // Make the actual enum here
        $(#[$m])*
        $visa enum $name {
            $($val $(= $epr)?),*
        }

        // Make it implement our custom type `EnumIntoIterator` that will
        // convert any enum into a valid iterator.
        impl EnumIntoIterator for $name {
            type IntoIter = $itr;

            fn iter() -> Self::IntoIter {
                $itr {
                    index: 0
                }
            }
        }

        // Make the iterator struct that stores the current index
        $visa struct $itr {
            index: usize
        }

        // Implement all the needed types for this to work in a `for` loop etc...
        impl Iterator for $itr {
            type Item = $name;

            fn next(&mut self) -> Option<Self::Item>{
                let value = *($name::ITEMS.get(self.index)?);
                self.index += 1;

                Some(value)
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                ($name::ITEMS.len(), Some($name::ITEMS.len()))
            }

            fn nth(&mut self, n: usize) -> Option<Self::Item> {
                Some(*($name::ITEMS.get(n)?))
            }
        }

        // Finally make the big array of all the items in your enum,
        // This will then be indexed into with the Iterator we constructed
        impl $name {
            /// # ITEMS
            /// A big array containing all the items in your enum. This array only contains
            /// the keys in your enum and does not have their values if provided. This is
            /// simply for indexing into your enum as if it was just an array of ints.
            ///
            /// # Safety
            /// This array is a standard rust style array, no no undefined behavior should exist.
            /// The index is a normal usize and each element is in the same order as they where
            /// defined in your macro.
            ///
            /// # How to use
            ///
            /// ```rust
            /// use quantum_os::enum_iterator;
            /// use quantum_os::enum_iterator::EnumIntoIterator;
            ///
            /// enum_iterator! {
            ///     pub enum MyEnum(MyEnumIter) {
            ///         SomeVal,
            ///         SomeOtherVal,
            ///         MoreOptions
            ///     }
            /// }
            ///
            /// fn main() {
            ///     // MyEnum::ITEMS is this array!
            ///     let all_items_in_my_enum = MyEnum::ITEMS;
            ///
            ///     for i in all_items_in_my_enum {
            ///         todo!()
            ///     }
            /// }
            ///
            /// ```
            ///
            const ITEMS: [$name; <[$name]>::len(&[$($name::$val),*])] =
                [$($name::$val),*];

            #[allow(unused_variables)]
            pub fn get_index_of(item: $name) -> usize {
                Self::ITEMS
                    .iter()
                    .position(|value| matches!(value, item))
                    .unwrap()
            }

        }

    }
}
