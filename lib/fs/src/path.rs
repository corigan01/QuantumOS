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
        let mut rebuilding_string = String::new();
        let mut state = 0;
        let iterator = self
            .children()
            .filter(|member| member != &".")
            .rev()
            .map(|member| {
                if member == ".." {
                    state += 1;
                    return "";
                }

                if state > 0 {
                    state -= 1;
                    return "";
                }

                member
            })
            .filter(|member| member.len() != 0);

        let self_0 = self.0.trim();

        let collection: Vec<&str> = iterator.collect();
        if !self_0.starts_with("/") {
            for _ in 0..state {
                rebuilding_string.push_str("../");
            }
        }

        for child in collection.iter().rev() {
            rebuilding_string.push_str("/");
            rebuilding_string.push_str(child);
        }

        if !self_0.ends_with("/") {
            rebuilding_string = String::from(rebuilding_string.trim_start_matches("/"));
        }

        if (self_0.ends_with("/") || self_0.ends_with("/..")) && !rebuilding_string.ends_with("/") {
            rebuilding_string.push_str("/");
        }

        if self_0.starts_with(".") && !rebuilding_string.starts_with(".") {
            rebuilding_string.prepend(".");
        }

        if self_0.starts_with("/") && !rebuilding_string.starts_with("/") {
            rebuilding_string.prepend("/");
        }

        if !self_0.starts_with("/") && rebuilding_string.starts_with("/") {
            rebuilding_string = String::from(&rebuilding_string.as_str()[1..]);
        }

        if rebuilding_string.len() == 0 {
            rebuilding_string.push_str(".");
        }

        Path::from(rebuilding_string)
    }

    pub fn children<'a>(&'a self) -> impl DoubleEndedIterator<Item = &'a str> {
        self.0
            .as_str()
            .split('/')
            .filter(|member| member.len() != 0 && member != &"/")
    }

    pub fn is_absolute(&self) -> bool {
        self.0.starts_with("/")
    }

    pub fn is_relative(&self) -> bool {
        !self.is_absolute()
    }

    /// # Snip Off
    /// Takes the current path and trims it to the part past the path provided.
    pub fn remove_parent(&self, path: &Path) -> Option<Path> {
        if !path.is_child_of(&self) {
            return None;
        }

        Some(Path::from(
            &self.0.as_str()[path.0.trim_end_matches("/").len()..],
        ))
    }

    pub fn as_str<'a>(&'a self) -> &'a str {
        self.0.as_str()
    }

    pub fn is_child_of(&self, path: &Path) -> bool {
        if self.children().count() > path.children().count() {
            return false;
        }

        for (idx, child) in self.children().enumerate() {
            if child != path.children().nth(idx).unwrap_or("") {
                return false;
            }
        }

        true
    }

    pub fn parent_path(&self) -> Path {
        let mut new_path_string = String::from(self.0.as_str());
        new_path_string.push_str("/..");

        if self.0.ends_with("/") {
            new_path_string.push_str("/");
        }

        new_path_string = Path::from(new_path_string).truncate_path().0;

        if new_path_string.ends_with("/") && !self.0.ends_with("/") {
            new_path_string.pop();
        }

        if !new_path_string.starts_with("/") && self.0.starts_with("/") {
            new_path_string.prepend("/");
        }

        Path::from(new_path_string)
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
            children.collect::<Vec<&str>>().as_slice(),
            &["1", "2", "3", "4", "5", "6", "7", "8"]
        );

        let path = Path::from("///1/2/3/4/5/../6");
        let children = path.children();
        assert_eq!(
            children.collect::<Vec<&str>>().as_slice(),
            &["1", "2", "3", "4", "5", "..", "6"]
        );
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
