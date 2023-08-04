/*
  ____                 __
 / __ \__ _____ ____  / /___ ____ _
/ /_/ / // / _ `/ _ \/ __/ // /  ' \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/
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

use std::{env, fs};
use std::io::{Error, ErrorKind};
use std::path::Path;
use crate::CompileOptions;

pub fn ensure_artifact_dir_exists(path: &Path) -> std::io::Result<()> {
    if path.is_file() {
        fs::remove_file(path)?;
    }

    if path.exists() {
        return Ok(());
    }

    fs::create_dir(path)
}

/// # Get Cargo Path
/// Finds the true cargo path from the $PATH env variable. A simple 'whereis' in rust.
///
/// ### Function Steps
///   * Gets Path String
///   * Splits path string into path's
///       - Goes from "/bar/foo:/bar/bazz:/foo/bar" to ["/bar/foo", "/bar/bazz" ...]
///   * Takes path and walks the directory to find each file
///       - checks if the filename is 'cargo'
///   * Returns the first full path with filename 'cargo'
pub fn get_cargo_path() -> std::io::Result<String> {
    env::var("PATH")
        .map_err(|_| Error::new(ErrorKind::NotFound, "Could not get PATH"))?
        .split(":")
        .find_map(|path| {
            let path = Path::new(path);
            path.read_dir().ok()?.find_map(|entry| {
                let entry = entry.ok()?;

                if entry.file_name() == "cargo" {
                    Some(String::from(entry.path().to_string_lossy()))
                } else {
                    None
                }
            })
        })
        .ok_or(Error::new(ErrorKind::NotFound, "Could not find cargo in PATH"))
}

pub fn does_directory_contain_file(dir_path: &str, filename: &str) -> std::io::Result<bool> {

    let dir = Path::new(dir_path);
    dir.read_dir()?
        .find(|file| {
            if let Ok(file) = file {
                file.file_name() == filename
            } else { false }
        })
        .ok_or(Error::new(ErrorKind::NotFound, "Could not find file in path"))?
        .map(|_| true)
}

pub fn get_project_root() -> std::io::Result<String> {
    let current_dir = env::current_dir()
        .map_err(|_| Error::new(ErrorKind::NotFound, "Could not find current directory"))?;

    let attempted_root =
        current_dir
            .to_string_lossy()
            .split("Meta")
            .next()
            .map(|str| String::from(str))
            .ok_or(Error::new(ErrorKind::NotFound, "Could not determine path of project root"))?;

    if !does_directory_contain_file(attempted_root.as_str(), "Meta")? ||
        !does_directory_contain_file(attempted_root.as_str(), "kernel")? {
        return Err(Error::new(ErrorKind::Unsupported, "Attempted project root does not contain './Meta/' or './Kernel/', which should not be possible"))
    }

    Ok(String::from(attempted_root))
}

pub fn build_kernel(artifact_dir: &Path, options: &CompileOptions) -> std::io::Result<()> {
    let cargo_path = get_cargo_path()?;




    Ok(())
}

#[cfg(test)]
mod test {
    use crate::artifacts::{get_cargo_path, get_project_root};

    #[test]
    fn does_find_cargo_path() {
        let cargo_path = get_cargo_path();
        assert!(cargo_path.is_ok(), "{:?}", cargo_path);
    }

    #[test]
    fn test_find_project_root() {
        let project_root = get_project_root();
        assert!(project_root.is_ok(), "{:?}", project_root);

    }
}