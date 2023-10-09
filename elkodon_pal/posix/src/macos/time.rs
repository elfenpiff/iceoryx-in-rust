#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]

use crate::posix::types::*;
use crate::posix::Errno;

pub unsafe fn clock_gettime(clock_id: clockid_t, tp: *mut timespec) -> int {
    crate::internal::clock_gettime(clock_id, tp)
}

pub unsafe fn clock_settime(clock_id: clockid_t, tp: *const timespec) -> int {
    crate::internal::clock_settime(clock_id, tp)
}

pub unsafe fn clock_nanosleep(
    clock_id: clockid_t,
    flags: int,
    rqtp: *const timespec,
    rmtp: *mut timespec,
) -> int {
    if clock_id != crate::posix::CLOCK_REALTIME {
        Errno::set(Errno::EINVAL);
        return -1;
    }

    crate::internal::nanosleep(rqtp, rmtp)
}
