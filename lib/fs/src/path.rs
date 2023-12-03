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

#[derive(Clone, Debug)]
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

impl PartialEq<&str> for Path {
    fn eq(&self, other: &&str) -> bool {
        self.0.as_str() == *other
    }
}

impl PartialEq<Path> for Path {
    fn eq(&self, other: &Path) -> bool {
        self.0.as_str() == other.0.as_str()
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
    ///     desolved = "/"
    ///
    /// 2.  Path = "./////././///.."
    ///     desolved = ".."
    ///
    /// 3.  Path = "/Node/../Node/../."
    ///     desolved = "/"
    ///
    pub fn truncate_path(&self) -> Path {
        let mut starting_path = self.0.clone();

        // TODO: Make this better in the future
        // I would like to do this without memory allocation in the future. I think memory
        // allocation is just slowing this down.

        let final_string: String = loop {
            let before_buffer = [""].iter().map(|entry| *entry);
            let mut final_string: String = before_buffer
                .chain(
                    Path::from(starting_path.clone())
                        // Get the children of the path
                        // Example path = "/home/user/someone"
                        //  1. 'home'
                        //  2. 'user'
                        //  3. 'someone'
                        .children()
                        .into_iter()
                        // Remove all '.'
                        .filter(|child| child != &".")
                        .chain(["", "", ""]),
                )
                .collect::<Vec<&str>>()
                // Remove all "/path/../path/"
                .windows(2)
                .scan(0_usize, |val, init| {
                    if *val > 0 {
                        *val = val.checked_sub(1).unwrap_or(0);
                        return Some("");
                    }
                    let first = &init[0];
                    let second = &init[1];

                    if second == &".." && first != &".." {
                        *val += 1;
                        Some("")
                    } else {
                        *val = val.checked_sub(1).unwrap_or(0);
                        Some(first)
                    }
                })
                .filter(|val| val.len() != 0)
                // Add the '/' back for each of the children
                .fold(
                    starting_path
                        .starts_with('/')
                        .then(|| String::from("/"))
                        .unwrap_or(String::new()),
                    |mut acc, val| {
                        acc.push_str(val);
                        acc.push_str("/");

                        acc
                    },
                );

            // Replace some remaining truncated chars
            if starting_path.starts_with(".") && !final_string.starts_with(".") {
                final_string = String::from(".") + final_string;
            }

            if starting_path.ends_with("/") && !final_string.ends_with("/") {
                final_string.push_str("/");
            }

            if !(starting_path.ends_with("/") || starting_path.ends_with("."))
                && final_string.ends_with("/")
            {
                final_string.pop();
            }

            if starting_path.starts_with("/") && final_string.len() == 0 {
                final_string.push_str("/");
            }

            if final_string.len() == 0 {
                final_string.push_str(".");
            }

            if !final_string.contains("..")
                || final_string.starts_with("..")
                || final_string.starts_with("/..")
            {
                break final_string;
            }

            starting_path = final_string;
        };

        Path(final_string)
    }

    pub fn children<'a>(&'a self) -> Vec<&'a str> {
        self.0
            .as_str()
            .split('/')
            .filter(|child| !(*child == "/") && !(*child == ""))
            .collect()
    }

    pub fn is_absolute(&self) -> bool {
        self.0.starts_with("/")
    }

    pub fn is_relative(&self) -> bool {
        !self.is_absolute()
    }

    pub fn snip_off(self, path: Path) -> Option<Path> {
        let mut path = path.truncate_path();

        if path.0.ends_with("/") {
            path = Path(path.0.as_str()[..path.0.len() - 1].into())
        }

        if !self.0.starts_with(path.as_str()) {
            return None;
        }

        Some(Path::from(String::from(
            &self.0.as_str()[path.as_str().len()..],
        )))
    }

    pub fn as_str<'a>(&'a self) -> &'a str {
        self.0.as_str()
    }

    pub fn is_child_of(&self, path: &Path) -> bool {
        if self.0.starts_with(path.0.as_str()) {
            return true;
        }

        let truncated_self = self.clone().truncate_path();
        let truncated_other = path.clone().truncate_path();

        truncated_self.0.starts_with(truncated_other.0.as_str())
    }

    pub fn parent_path(&self) -> Path {
        let parent_truncated = self.clone().truncate_path();
        let mut children = parent_truncated.children();
        children.pop();

        if self.0.ends_with("/") && self.0.len() > 1 {
            children.pop();
        }

        let mut rebuilt = String::new();
        for child in children {
            rebuilt.push_str(child);
            rebuilt.push('/');
        }

        if !self.0.ends_with("/") {
            rebuilt.pop();
        }

        if rebuilt.is_empty() {
            rebuilt.push_str("/");
        }

        Path::from(rebuilt)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_path_from() {
        crate::set_example_allocator();
        assert_eq!(
            Path::from("/home/cringe/some_file"),
            "/home/cringe/some_file"
        );

        assert_eq!(Path::from("/test/test/test/test"), "/test/test/test/test");

        assert_eq!(Path::from("/something/"), "/something/");
    }

    #[test]
    fn test_truncate() {
        crate::set_example_allocator();
        assert_eq!(Path::from("./").truncate_path(), "./");

        assert_eq!(Path::from("./../../../.").truncate_path(), "../../../");
        assert_eq!(
            Path::from("someone/../someone/").truncate_path(),
            "someone/"
        );
        assert_eq!(Path::from(".//.///././///././.").truncate_path(), ".");
        assert_eq!(Path::from("/.//././//././//").truncate_path(), "/");
        assert_eq!(
            Path::from("/.//././//././//testdir").truncate_path(),
            "/testdir"
        );

        assert_eq!(
            Path::from("/.//././//././//test/").truncate_path(),
            "/test/"
        );
    }

    #[test]
    fn test_truncation_2() {
        crate::set_example_allocator();
        assert_eq!(Path::from("somepath/test").truncate_path(), "somepath/test");
        assert_eq!(Path::from("somepath/test/..").truncate_path(), "somepath/");
    }

    #[test]
    fn test_root() {
        crate::set_example_allocator();
        assert_eq!(Path::from("/"), "/");
        assert_eq!(Path::from("/").truncate_path(), "/");
    }

    const TEST_CASE_LOTS: [(&str, &str); 10] = [
        ("/home/test/../test", "/home/test"),
        ("/wow/wow/wow/wow/wow/wow/", "/wow/wow/wow/wow/wow/wow/"),
        (".", "."),
        ("this_is_a_super_long_path_name_test/..", "."),
        ("/bin/bash", "/bin/bash"),
        (
            "some_path/wow/other/../nothing/etc/../../test",
            "some_path/wow/test",
        ),
        ("//..", "/"),
        ("/../../../../..", "/"),
        ("/////////////", "/"),
        ("/path/buf/buf/test/", "/path/buf/buf/test/"),
    ];

    #[test]
    fn test_lots_paths() {
        crate::set_example_allocator();

        for (requires_truncate, test) in TEST_CASE_LOTS {
            assert_eq!(
                Path::from(requires_truncate).truncate_path(),
                test,
                "\n\tPath: '{}', Expected: '{}'",
                requires_truncate,
                test
            );
        }
    }

    #[test]
    fn test_multiple_truncate() {
        crate::set_example_allocator();

        for (requires_truncate, test) in TEST_CASE_LOTS {
            assert_eq!(
                Path::from(requires_truncate)
                    .truncate_path()
                    .truncate_path()
                    .truncate_path()
                    .truncate_path(),
                Path::from(test)
                    .truncate_path()
                    .truncate_path()
                    .truncate_path()
                    .truncate_path()
            );
        }
    }

    #[test]
    fn test_child_path() {
        crate::set_example_allocator();

        let path = Path::from("/1/2/3/4/5/6/7/8");
        let children = path.children();
        assert_eq!(
            children.as_slice(),
            &["1", "2", "3", "4", "5", "6", "7", "8"]
        )
    }

    #[test]
    fn test_parent_path() {
        crate::set_example_allocator();

        assert_eq!(
            Path::from("/something/neat.txt").parent_path(),
            Path::from("/something")
        );

        assert_eq!(
            Path::from("/something/neat/").parent_path(),
            Path::from("/something/")
        );

        assert_eq!(
            Path::from("/something/otherthing/super").parent_path(),
            Path::from("/something/otherthing")
        );

        assert_eq!(Path::from("/test").parent_path(), Path::from("/"));
        assert_eq!(Path::from("/").parent_path(), Path::from("/"));
    }

    #[test]
    fn test_parent_path_second() {
        crate::set_example_allocator();

        assert_eq!(
            Path::from("/this/this/").parent_path(),
            Path::from("/this/")
        );

        assert_eq!(Path::from("//////////").parent_path(), Path::from("/"));
        assert_eq!(
            Path::from("this is a test/ path or something//nicepath/or something").parent_path(),
            Path::from("this is a test/ path or something/nicepath")
        );
    }
}
