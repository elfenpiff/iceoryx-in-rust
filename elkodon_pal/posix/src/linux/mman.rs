#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]

use crate::posix::{closedir, opendir, readdir, types::*};

pub unsafe fn mlock(addr: *const void, len: size_t) -> int {
    crate::internal::mlock(addr, len)
}

pub unsafe fn munlock(addr: *const void, len: size_t) -> int {
    crate::internal::munlock(addr, len)
}

pub unsafe fn mlockall(flags: int) -> int {
    crate::internal::mlockall(flags)
}

pub unsafe fn munlockall() -> int {
    crate::internal::munlockall()
}

pub unsafe fn shm_open(name: *const char, oflag: int, mode: mode_t) -> int {
    crate::internal::shm_open(name, oflag, mode)
}

pub unsafe fn shm_unlink(name: *const char) -> int {
    crate::internal::shm_unlink(name)
}

pub unsafe fn shm_list() -> Vec<[i8; 256]> {
    let mut result = vec![];
    let dir = opendir(b"/dev/shm/\0".as_ptr().cast());
    if dir == 0 as *mut DIR {
        return result;
    }

    loop {
        let entry = readdir(dir);
        if entry == 0 as *const dirent {
            break;
        }

        result.push((*entry).d_name);
    }
    closedir(dir);

    result
}

pub unsafe fn mmap(
    addr: *mut void,
    len: size_t,
    prot: int,
    flags: int,
    fd: int,
    off: off_t,
) -> *mut void {
    crate::internal::mmap(addr, len, prot, flags, fd, off)
}

pub unsafe fn munmap(addr: *mut void, len: size_t) -> int {
    crate::internal::munmap(addr, len)
}

pub unsafe fn mprotect(addr: *mut void, len: size_t, prot: int) -> int {
    crate::internal::mprotect(addr, len, prot)
}
