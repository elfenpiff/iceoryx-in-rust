#![allow(non_camel_case_types, non_snake_case)]
#![allow(clippy::missing_safety_doc)]

use crate::posix::types::*;

pub unsafe fn select(
    nfds: int,
    readfds: *mut fd_set,
    writefds: *mut fd_set,
    errorfds: *mut fd_set,
    timeout: *mut timeval,
) -> int {
    crate::internal::select(nfds, readfds, writefds, errorfds, timeout)
}

pub const fn CMSG_ALIGN(len: usize) -> usize {
    (len + std::mem::size_of::<usize>() - 1) & !(std::mem::size_of::<usize>() - 1)
}

pub const unsafe fn CMSG_SPACE(length: uint) -> uint {
    (CMSG_ALIGN(length as usize) + CMSG_ALIGN(std::mem::size_of::<cmsghdr>())) as uint
}

pub fn CMSG_SPACE_NON_CONST(length: uint) -> uint {
    (CMSG_ALIGN(length as usize) + CMSG_ALIGN(std::mem::size_of::<cmsghdr>())) as uint
}

pub unsafe fn CMSG_FIRSTHDR(mhdr: *const msghdr) -> *mut cmsghdr {
    match ((*mhdr).msg_controllen as usize) < std::mem::size_of::<cmsghdr>() {
        true => std::ptr::null_mut::<cmsghdr>(),
        false => (*mhdr).msg_control as *mut cmsghdr,
    }
}

pub unsafe fn CMSG_NXTHDR(header: *const msghdr, sub_header: *const cmsghdr) -> *mut cmsghdr {
    // no header contained
    if (*sub_header).cmsg_len < std::mem::size_of::<cmsghdr>() as _ {
        return std::ptr::null_mut::<cmsghdr>();
    };

    let next_sub_header =
        (sub_header as usize + CMSG_ALIGN((*sub_header).cmsg_len as _)) as *mut cmsghdr;
    let end_of_message = (*header).msg_control as usize + (*header).msg_controllen as usize;

    if (next_sub_header.offset(1)) as usize > end_of_message {
        return std::ptr::null_mut::<cmsghdr>();
    }

    if next_sub_header as usize + CMSG_ALIGN((*next_sub_header).cmsg_len as _) > end_of_message {
        return std::ptr::null_mut::<cmsghdr>();
    }

    next_sub_header as *mut cmsghdr
}

pub const unsafe fn CMSG_LEN(length: uint) -> uint {
    CMSG_ALIGN(std::mem::size_of::<cmsghdr>()) as uint + length
}

pub unsafe fn CMSG_DATA(cmsg: *const cmsghdr) -> *mut uchar {
    (cmsg as usize + CMSG_ALIGN(core::mem::size_of::<cmsghdr>())) as *mut uchar
}

pub unsafe fn FD_CLR(fd: int, set: *mut fd_set) {
}

pub unsafe fn FD_ISSET(fd: int, set: *const fd_set) -> bool {
    false
}

pub unsafe fn FD_SET(fd: int, set: *mut fd_set) {
}

pub unsafe fn FD_ZERO(set: *mut fd_set) {
}
