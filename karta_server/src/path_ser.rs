// Utility functions for serializing and deserializing paths.
// These should be expanded to correctly handle paths that are not utf-8.
// The database should be able to be transported between different operating systems.

// NOTE: The handling of non-utf8 paths is not implemented yet. The functions exist and
// are to be used in the code so that their internal implementation can be changed later.

use std::path::PathBuf;

// TODO: Actually handle non-utf8 paths.
pub fn str_to_buf(str: &str) -> PathBuf {
    PathBuf::from(str)
}

pub fn buf_to_str(buf: &PathBuf) -> String {
    buf.to_str().unwrap().into()
}