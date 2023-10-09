#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]

use crate::posix::types::*;
use crate::posix::Errno;

pub unsafe fn setsockopt(
    socket: int,
    level: int,
    option_name: int,
    option_value: *const void,
    option_len: socklen_t,
) -> int {
    crate::internal::setsockopt(socket, level, option_name, option_value, option_len)
}

pub unsafe fn getsockname(socket: int, address: *mut sockaddr, address_len: *mut socklen_t) -> int {
    crate::internal::getsockname(socket, address, address_len)
}

pub unsafe fn getsockopt(
    socket: int,
    level: int,
    option_name: int,
    option_value: *mut void,
    option_len: *mut socklen_t,
) -> int {
    crate::internal::getsockopt(socket, level, option_name, option_value, option_len)
}

pub unsafe fn bind(socket: int, address: *const sockaddr, address_len: socklen_t) -> int {
    crate::internal::bind(socket, address, address_len)
}

pub unsafe fn connect(socket: int, address: *const sockaddr, address_len: socklen_t) -> int {
    crate::internal::connect(socket, address, address_len)
}

pub unsafe fn socket(domain: int, socket_type: int, protocol: int) -> int {
    crate::internal::socket(domain, socket_type, protocol)
}

pub unsafe fn sendmsg(socket: int, message: *const msghdr, flags: int) -> ssize_t {
    crate::internal::sendmsg(socket, message, flags)
}

pub unsafe fn sendto(
    socket: int,
    message: *const void,
    length: size_t,
    flags: int,
    dest_addr: *const sockaddr,
    dest_len: socklen_t,
) -> ssize_t {
    let result = crate::internal::sendto(socket, message, length, flags, dest_addr, dest_len);

        if result == -1 {
            Errno::correct_errno(Errno::ENOBUFS, Errno::EAGAIN);
        }

    result
}

pub unsafe fn send(socket: int, message: *const void, length: size_t, flags: int) -> ssize_t {
    crate::internal::send(socket, message, length, flags)
}

pub unsafe fn recvmsg(socket: int, message: *mut msghdr, flags: int) -> ssize_t {
    crate::internal::recvmsg(socket, message, flags)
}

pub unsafe fn recvfrom(
    socket: int,
    buffer: *mut void,
    length: size_t,
    flags: int,
    address: *mut sockaddr,
    address_len: *mut socklen_t,
) -> ssize_t {
    crate::internal::recvfrom(socket, buffer, length, flags, address, address_len)
}

pub unsafe fn recv(socket: int, buffer: *mut void, length: size_t, flags: int) -> ssize_t {
    crate::internal::recv(socket, buffer, length, flags)
}
