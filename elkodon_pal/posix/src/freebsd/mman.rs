#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]

use crate::posix::stdlib::*;
use crate::posix::string::*;
use crate::posix::types::*;

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

pub unsafe fn shm_list() -> Vec<[i8; 256]> {
    let mut result = vec![];

    let align = |value, alignment| {
        if value % alignment == 0 {
            value
        } else {
            value + alignment - value % alignment
        }
    };

    let listmib = b"kern.ipc.posix_shm_list\0";
    let mut mib = [0 as int; 3];
    let mut miblen: size_t = 3;
    if unsafe { sysctlnametomib(listmib.as_ptr() as *mut i8, mib.as_mut_ptr(), &mut miblen) } == -1
    {
        return result;
    }

    let mut len: size_t = 0;
    if unsafe {
        sysctl(
            mib.as_mut_ptr(),
            miblen as _,
            0 as *mut void,
            &mut len,
            0 as *mut void,
            0,
        )
    } == -1
    {
        return result;
    }

    len = len * 4 / 3;
    let buffer = unsafe { malloc(len) };

    if buffer == 0 as *mut void {
        return result;
    }

    if unsafe {
        sysctl(
            mib.as_mut_ptr(),
            miblen as _,
            buffer,
            &mut len,
            0 as *mut void,
            0,
        )
    } != 0
    {
        unsafe { free(buffer) };
        return result;
    }

    let mut temp = buffer;
    loop {
        let kif = unsafe { &*(temp as *const kinfo_file) };
        if kif.kf_structsize == 0 || kif.kf_path[0] == 0 {
            break;
        }

        let mut name = [0; 256];
        strncpy(
            name.as_mut_ptr() as *mut char,
            kif.kf_path.as_ptr().offset(1) as *const char,
            256,
        );

        result.push(name);

        temp = unsafe {
            temp.offset(align(
                kif.kf_structsize as usize,
                core::mem::align_of::<kinfo_file>(),
            ) as _)
        };
    }

    unsafe { free(buffer) };
    result
}

pub unsafe fn sysctl(
    name: *mut int,
    namelen: uint,
    oldp: *mut void,
    oldlenp: *mut size_t,
    newp: *mut void,
    newlen: size_t,
) -> int {
    crate::internal::sysctl(name, namelen, oldp, oldlenp, newp, newlen)
}

pub unsafe fn sysctlnametomib(name: *mut char, mibp: *mut int, sizep: *mut size_t) -> int {
    crate::internal::sysctlnametomib(name, mibp, sizep)
}
