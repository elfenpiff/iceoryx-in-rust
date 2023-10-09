#![allow(non_camel_case_types, non_snake_case)]
#![allow(clippy::missing_safety_doc)]
#![allow(unused_variables)]

use windows_sys::Win32::{
    Foundation::ERROR_FILE_NOT_FOUND,
    Storage::FileSystem::{GetFileAttributesA, FILE_ATTRIBUTE_DIRECTORY, INVALID_FILE_ATTRIBUTES},
};

use crate::{
    posix::types::*,
    posix::{acquire_mode_from_path, Errno, S_IFDIR, S_IFREG},
    win32call,
    win_internal::c_string_length,
};

pub unsafe fn stat(path: *const char, buf: *mut stat_t) -> int {
    let attr = win32call! { GetFileAttributesA(path as *const u8), ignore ERROR_FILE_NOT_FOUND};
    if attr == INVALID_FILE_ATTRIBUTES {
        return -1;
    }

    if attr & FILE_ATTRIBUTE_DIRECTORY != 0 {
        (*buf).st_mode = S_IFDIR;
    } else {
        (*buf).st_mode = S_IFREG;
    }

    match acquire_mode_from_path(core::slice::from_raw_parts(
        path as *const u8,
        c_string_length(path),
    )) {
        None => {
            Errno::set(Errno::EOVERFLOW);
            -1
        }
        Some(mode) => {
            (*buf).st_mode = (*buf).st_mode | mode;

            0
        }
    }
}

pub unsafe fn umask(mask: mode_t) -> mode_t {
    mode_t::MAX
}
