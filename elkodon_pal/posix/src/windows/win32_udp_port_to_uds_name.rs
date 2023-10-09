use windows_sys::Win32::{
    Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE},
    Security::SECURITY_ATTRIBUTES,
    System::Memory::{
        CreateFileMappingA, MapViewOfFile, UnmapViewOfFile, VirtualAlloc, FILE_MAP_ALL_ACCESS,
        MEM_COMMIT, PAGE_READWRITE, SEC_RESERVE,
    },
};

use crate::posix::types::*;
use std::{
    cell::UnsafeCell,
    sync::atomic::{AtomicU64, Ordering},
};

const IS_INITIALIZED: u64 = 0xaffedeadbeef;
const INITIALIZATION_IN_PROGRESS: u64 = 0xbebebebebebebebe;
const SHM_SEGMENT_NAME: &[u8] = b"/port_to_uds_name_map\0";
const SHM_SIZE: usize = core::mem::size_of::<PortToUdsNameMap>();
pub(crate) const MAX_UDS_NAME_LEN: usize = 108;

struct Entry {
    aba_counter: AtomicU64,
    value: [UnsafeCell<[u8; MAX_UDS_NAME_LEN]>; 2],
}

impl Entry {
    fn initialize(&mut self) {
        self.aba_counter = AtomicU64::new(1);
        self.value = [
            UnsafeCell::new([0; MAX_UDS_NAME_LEN]),
            UnsafeCell::new([0; MAX_UDS_NAME_LEN]),
        ];
    }

    fn set(&self, value: &[u8]) {
        let current = self.aba_counter.load(Ordering::Acquire);
        unsafe {
            (*self.value[(current % 2) as usize].get()) = [0u8; MAX_UDS_NAME_LEN];
            (*self.value[(current % 2) as usize].get())[..value.len()].copy_from_slice(value);
        };
        self.aba_counter.fetch_add(1, Ordering::Release);
    }

    fn get(&self) -> [u8; MAX_UDS_NAME_LEN] {
        let current = self.aba_counter.load(Ordering::Acquire);
        let mut result;
        loop {
            result = unsafe { *self.value[((current - 1) % 2) as usize].get() };
            if current == self.aba_counter.load(Ordering::Acquire) {
                break;
            }
        }

        result
    }
}

#[repr(C)]
struct PortToUdsNameMap {
    init_check: AtomicU64,
    uds_names: [Entry; 65535],
}

impl PortToUdsNameMap {
    fn initialize(&mut self) {
        let current = self.init_check.load(Ordering::Relaxed);

        match self.init_check.compare_exchange(
            current,
            INITIALIZATION_IN_PROGRESS,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => {
                for i in 0..self.uds_names.len() {
                    self.uds_names[i].initialize();
                }
                self.init_check.store(IS_INITIALIZED, Ordering::Relaxed);
                return;
            }
            Err(_) => while self.init_check.load(Ordering::Relaxed) != IS_INITIALIZED {},
        }
    }

    fn set(&self, port: u16, name: &[u8]) {
        self.uds_names[port as usize].set(name);
    }

    fn get(&self, port: u16) -> [u8; MAX_UDS_NAME_LEN] {
        self.uds_names[port as usize].get()
    }

    pub fn get_port(&self, name: &[u8]) -> u16 {
        for i in 0..self.uds_names.len() {
            let entry_name = self.uds_names[i].get();
            let pos = entry_name.iter().position(|c| *c == 0).unwrap_or(0);
            if pos == 0 || pos > name.len() {
                continue;
            }

            if &entry_name[..pos] == &name[..pos] {
                return i as _;
            }
        }
        0
    }
}

#[doc(hidden)]
pub struct PortToUds {
    shm_handle: HANDLE,
    map: *const PortToUdsNameMap,
}

unsafe impl Send for PortToUds {}
unsafe impl Sync for PortToUds {}

impl Drop for PortToUds {
    fn drop(&mut self) {
        unsafe {
            win32call! { UnmapViewOfFile(self.map as isize)};
            win32call! { CloseHandle(self.shm_handle)};
        }
    }
}

impl PortToUds {
    pub fn new() -> Option<Self> {
        let handle: HANDLE = 0;
        let shm_handle = unsafe {
            win32call! { CreateFileMappingA(handle, 0 as *const SECURITY_ATTRIBUTES, PAGE_READWRITE | SEC_RESERVE, 0, SHM_SIZE as _, SHM_SEGMENT_NAME.as_ptr()), ignore ERROR_ALREADY_EXISTS}
        };

        if shm_handle == 0 {
            return None;
        }

        let has_created_shm = unsafe { GetLastError() != ERROR_ALREADY_EXISTS };

        let map_result = unsafe {
            win32call! {MapViewOfFile(shm_handle, FILE_MAP_ALL_ACCESS, 0, 0, SHM_SIZE as _)}
        };

        if map_result == 0 {
            unsafe {
                win32call! { CloseHandle(shm_handle) }
            };
            return None;
        }

        if unsafe {
            win32call! {VirtualAlloc(map_result as *const void, SHM_SIZE as _, MEM_COMMIT, PAGE_READWRITE).is_null()}
        } {
            unsafe {
                win32call! { UnmapViewOfFile(map_result as isize)};
                win32call! { CloseHandle(shm_handle)};
            }
            return None;
        }

        let map = map_result as *mut PortToUdsNameMap;

        if has_created_shm {
            unsafe { &mut *map }.initialize();
        }

        Some(Self { shm_handle, map })
    }

    pub fn set(&self, port: u16, name: &[u8]) {
        unsafe { (*self.map).set(port, name) }
    }

    pub fn get_port(&self, name: &[u8]) -> u16 {
        unsafe { (*self.map).get_port(name) }
    }

    pub fn reset(&self, port: u16) {
        self.set(port, &[0; MAX_UDS_NAME_LEN]);
    }
}

#[cfg(test)]
mod tests {
    use elkodon_bb_testing::assert_that;

    use super::PortToUds;

    #[test]
    fn win32_udp_port_to_uds_name_set_and_get_works() {
        let sut = PortToUds::new().unwrap();

        sut.set(12345, b"hello world");
        sut.set(54321, b"some other test");
        sut.set(819, b"fuuu");

        let is_equal = |rhs: [u8; 108], lhs: &[u8]| &rhs[..lhs.len()] == lhs;

        assert_that!(is_equal(sut.get(12345), b"hello world"), eq true);
        assert_that!(is_equal(sut.get(54321), b"some other test"), eq true);
        assert_that!(is_equal(sut.get(819), b"fuuu"), eq true);

        assert_that!(sut.get_port(b"hello world"), eq 12345);
        assert_that!(sut.get_port(b"some other test"), eq 54321);
        assert_that!(sut.get_port(b"fuuu"), eq 819);
        assert_that!(sut.get_port(b""), eq 0);
        assert_that!(sut.get_port(b"x"), eq 0);
    }

    #[test]
    fn win32_udp_port_to_uds_name_set_and_get_works_with_multiple_instances() {
        let sut = PortToUds::new().unwrap();

        sut.set(12345, b"hello world");
        sut.set(54321, b"some other test");
        sut.set(819, b"fuuu");
        sut.set(331, b"i am a prime");

        let sut2 = PortToUds::new().unwrap();

        sut2.set(123, b"all glory");
        sut2.set(456, b"to the one and only");
        sut2.set(789, b"hypnotoad");
        sut2.reset(331);

        let is_equal = |rhs: [u8; 108], lhs: &[u8]| &rhs[..lhs.len()] == lhs;

        assert_that!(is_equal(sut2.get(12345), b"hello world"), eq true);
        assert_that!(is_equal(sut2.get(54321), b"some other test"), eq true);
        assert_that!(is_equal(sut2.get(819), b"fuuu"), eq true);
        assert_that!(is_equal(sut2.get(331), b""), eq true);
        assert_that!(sut2.get_port(b"hello world"), eq 12345);
        assert_that!(sut2.get_port(b"some other test"), eq 54321);
        assert_that!(sut2.get_port(b"fuuu"), eq 819);

        assert_that!(is_equal(sut.get(123), b"all glory"), eq true);
        assert_that!(is_equal(sut.get(456), b"to the one and only"), eq true);
        assert_that!(is_equal(sut.get(789), b"hypnotoad"), eq true);
        assert_that!(is_equal(sut.get(331), b""), eq true);
        assert_that!(sut.get_port(b"all glory"), eq 123);
        assert_that!(sut.get_port(b"to the one and only"), eq 456);
        assert_that!(sut.get_port(b"hypnotoad"), eq 789);
    }
}
