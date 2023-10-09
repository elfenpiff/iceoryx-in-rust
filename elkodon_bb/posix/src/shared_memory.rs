//! Provides access to a POSIX [`SharedMemory`]Object used to share memory between processes.
//!
//! # Important
//!
//! When constructing objects into the memory one MUST ensure that the memory representation is
//! identical in every process. Therefore, it is important to add `#[repr(C)]` to the struct. If
//! this struct is a composite every member must have `#[repr(C)]` enabled.
//!
//! # Examples
//!
//! ## Create non-existing shared memory.
//!
//! ```
//! use elkodon_bb_posix::shared_memory::*;
//! use elkodon_bb_system_types::file_name::FileName;
//! use elkodon_bb_container::semantic_string::*;
//!
//! let name = FileName::new(b"someShmName").unwrap();
//! let mut shm = SharedMemoryBuilder::new(&name)
//!                     .is_memory_locked(false)
//!           // the SharedMemoryCreationBuilder is used from here on
//!                     .creation_mode(CreationMode::PurgeAndCreate)
//!                     .size(1024)
//!                     .permission(Permission::OWNER_ALL)
//!                     .zero_memory(true)
//!                     .create()
//!                     .expect("failed to create shared memory");
//!
//! println!("shm name: {}", shm.name());
//! println!("shm addr: {:?}", shm.base_address());
//! println!("shm size: {}", shm.size());
//!
//! // set the first byte of the shared memory
//! shm.as_mut_slice()[0] = 0xFF;
//! ```
//!
//! ## Open existing shared memory.
//!
//! ```no_run
//! use elkodon_bb_posix::shared_memory::*;
//! use elkodon_bb_system_types::file_name::FileName;
//! use elkodon_bb_container::semantic_string::*;
//!
//! let name = FileName::new(b"someShmName").unwrap();
//! let shm = SharedMemoryBuilder::new(&name)
//!                     .is_memory_locked(false)
//!                     .open_existing(AccessMode::Read)
//!                     .expect("failed to open shared memory");
//!
//! // print the first byte of the shared memory
//! println!("first byte: {}", shm.as_slice()[0]);
//! ```

use crate::file::{FileStatError, FileTruncateError};
use crate::file_descriptor::*;
use crate::handle_errno;
use crate::memory_lock::{MemoryLock, MemoryLockCreationError};
use crate::signal::SignalHandler;
use crate::system_configuration::Limit;
use elkodon_pal_posix::posix::errno::Errno;
use elkodon_pal_posix::posix::POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING;
use elkodon_pal_posix::posix::POSIX_SUPPORT_PERSISTENT_SHARED_MEMORY;
use elkodon_pal_posix::*;
use elkodon_pal_settings::PATH_SEPARATOR;
use elkodon_bb_container::semantic_string::*;
use elkodon_bb_elementary::enum_gen;
use elkodon_bb_log::{error, fail, fatal_panic, trace, warn};
use elkodon_bb_system_types::file_name::*;
use elkodon_bb_system_types::file_path::*;
use elkodon_bb_system_types::path::*;

use std::ptr::NonNull;

pub use crate::access_mode::AccessMode;
pub use crate::creation_mode::CreationMode;
pub use crate::permission::Permission;

enum_gen! { SharedMemoryCreationError
  entry:
    SizeDoesNotFit,
    InsufficientMemory,
    InsufficientMemoryToBeMemoryLocked,
    UnsupportedSizeOfZero,
    InsufficientPermissions,
    MappedRegionLimitReached,
    PerProcessFileHandleLimitReached,
    SystemWideFileHandleLimitReached,
    NameTooLong,
    InvalidName,
    AlreadyExist,
    DoesNotExist,
    UnableToMapAtEnforcedBaseAddress,
    UnknownError(i32)
  mapping:
    FileTruncateError,
    FileStatError,
    MemoryLockCreationError,
    SharedMemoryRemoveError
}

enum_gen! { SharedMemoryRemoveError
  entry:
    InsufficientPermissions,
    DoesNotExist,
    UnknownError(i32)
}

/// The builder for the [`SharedMemory`].
#[derive(Debug)]
pub struct SharedMemoryBuilder {
    name: FileName,
    size: usize,
    is_memory_locked: bool,
    has_ownership: bool,
    permission: Permission,
    creation_mode: Option<CreationMode>,
    zero_memory: bool,
    access_mode: AccessMode,
    enforce_base_address: Option<u64>,
}

impl SharedMemoryBuilder {
    pub fn new(name: &FileName) -> Self {
        SharedMemoryBuilder {
            name: *name,
            size: 0,
            is_memory_locked: false,
            permission: Permission::OWNER_ALL,
            access_mode: AccessMode::None,
            has_ownership: true,
            creation_mode: None,
            zero_memory: true,
            enforce_base_address: None,
        }
    }

    /// Locks the shared memory into the heap. If this is enabled swapping of the
    /// created shared memory segment is no longer possible.
    pub fn is_memory_locked(mut self, value: bool) -> Self {
        self.is_memory_locked = value;
        self
    }

    /// Sets a base address for the shared memory which is enforced. When the shared memory
    /// could not mapped at the provided address the creation fails.
    pub fn enforce_base_address(mut self, value: u64) -> Self {
        self.enforce_base_address = Some(value);
        self
    }

    /// Opens an already existing shared memory. In contrast to its counterpart
    /// [`SharedMemoryBuilder::open_existing()`] it will not print an error message when the
    /// shared memory does not exist.
    pub fn try_open_existing(
        mut self,
        access_mode: AccessMode,
    ) -> Result<SharedMemory, SharedMemoryCreationError> {
        self.access_mode = access_mode;
        Self::open(self, true)
    }

    /// Opens an already existing shared memory.
    pub fn open_existing(
        mut self,
        access_mode: AccessMode,
    ) -> Result<SharedMemory, SharedMemoryCreationError> {
        self.access_mode = access_mode;
        Self::open(self, false)
    }

    fn open(
        mut self,
        quiet_when_shm_does_not_exist: bool,
    ) -> Result<SharedMemory, SharedMemoryCreationError> {
        let msg = "Unable to open shared memory";
        let fd = SharedMemory::shm_open(&self.name, &self, quiet_when_shm_does_not_exist)?;

        let actual_shm_size = fail!(from self, when fd.metadata(),
                "{} since a failure occurred while acquiring the file attributes.", msg)
        .size();
        self.size = actual_shm_size as usize;

        let base_address = fail!(from self, when SharedMemory::mmap(&fd, &self),
                        "{} since the memory could not be mapped.", msg);

        if self.enforce_base_address.is_some()
            && self.enforce_base_address.unwrap() != base_address as u64
        {
            fail!(from self, with SharedMemoryCreationError::UnableToMapAtEnforcedBaseAddress,
                "{} since the memory was mapped at {:X} which is not enforced base address.", msg, base_address as u64);
        }

        let shm = SharedMemory {
            name: self.name,
            base_address: base_address as *mut u8,
            size: actual_shm_size as usize,
            has_ownership: false,
            memory_lock: None,
            file_descriptor: fd,
        };

        trace!(from shm, "open");
        Ok(shm)
    }

    /// Creates a new shared memory segment.
    pub fn creation_mode(mut self, creation_mode: CreationMode) -> SharedMemoryCreationBuilder {
        self.access_mode = AccessMode::ReadWrite;
        self.creation_mode = Some(creation_mode);
        SharedMemoryCreationBuilder { config: self }
    }
}

#[derive(Debug)]
pub struct SharedMemoryCreationBuilder {
    config: SharedMemoryBuilder,
}

impl SharedMemoryCreationBuilder {
    /// Sets the permissions of the new shared memory
    pub fn permission(mut self, value: Permission) -> Self {
        self.config.permission = value;
        self
    }

    /// Zero the memory of the shared memory. It can serve to purposes.
    /// * Ensure that the memory is clean before using it.
    /// * Ensure that enough memory is actually available. On some operating systems the memory is
    ///   only virtually allocated and when it is later required but there is not enough memory
    ///   left the application fails.
    pub fn zero_memory(mut self, value: bool) -> Self {
        self.config.zero_memory = value;
        self
    }

    /// The size of the shared memory.
    pub fn size(mut self, size: usize) -> Self {
        self.config.size = size;
        self
    }

    /// Defines if a newly created [`SharedMemory`] owns the underlying resources. If they are not
    /// owned they will not be cleaned up and can be opened later but they need to be explicitly
    /// removed.
    pub fn has_ownership(mut self, value: bool) -> Self {
        self.config.has_ownership = value;
        self
    }

    /// Creates the shared memory segment.
    pub fn create(mut self) -> Result<SharedMemory, SharedMemoryCreationError> {
        let msg = "Unable to create shared memory";

        let shm_created;
        let fd = match self
            .config
            .creation_mode
            .expect("CreationMode must be set on creation")
        {
            CreationMode::CreateExclusive => {
                shm_created = true;
                SharedMemory::shm_create(&self.config.name, &self.config)?
            }
            CreationMode::PurgeAndCreate => {
                shm_created = true;
                fail!(from self.config, when SharedMemory::shm_unlink(&self.config.name, UnlinkMode::QuietWhenNotExisting),
                    "Failed to remove already existing shared memory.");
                SharedMemory::shm_create(&self.config.name, &self.config)?
            }
            CreationMode::OpenOrCreate => {
                match SharedMemory::shm_open(&self.config.name, &self.config, true) {
                    Ok(fd) => {
                        shm_created = false;
                        self.config.has_ownership = false;
                        fd
                    }
                    Err(SharedMemoryCreationError::DoesNotExist) => {
                        shm_created = true;
                        SharedMemory::shm_create(&self.config.name, &self.config)?
                    }
                    Err(v) => return Err(v),
                }
            }
        };

        let base_address = fail!(from self.config, when SharedMemory::mmap(&fd, &self.config),
                                    "{} since the memory could not be mapped.", msg)
            as *mut u8;

        if self.config.enforce_base_address.is_some()
            && self.config.enforce_base_address.unwrap() != base_address as u64
        {
            fail!(from self.config, with SharedMemoryCreationError::UnableToMapAtEnforcedBaseAddress,
                "{} since the memory was mapped at {:X} which is not enforced base address.", msg, base_address as u64);
        }

        let mut shm = SharedMemory {
            name: self.config.name,
            base_address,
            size: self.config.size,
            has_ownership: self.config.has_ownership,
            memory_lock: None,
            file_descriptor: fd,
        };

        if !shm_created {
            let actual_shm_size = fail!(from self.config, when shm.metadata(),
                    "{} since a failure occurred while acquiring the file attributes.", msg)
            .size();
            if actual_shm_size as usize != self.config.size {
                fail!(from self.config, with SharedMemoryCreationError::SizeDoesNotFit,
                    "{} since the actual size {} is not equal to the configured size {}.", msg, actual_shm_size, self.config.size);
            }

            trace!(from shm, "open");
            return Ok(shm);
        }

        fail!(from self.config, when shm.truncate(self.config.size), "{} since the shared memory truncation failed.", msg);

        let actual_shm_size = fail!(from self.config, when shm.metadata(),
                "{} since a failure occurred while acquiring the file attributes.", msg)
        .size();
        if actual_shm_size as usize != self.config.size {
            fail!(from self.config, with SharedMemoryCreationError::SizeDoesNotFit,
                "{} since the actual size {} is not equal to the configured size {}.", msg, actual_shm_size, self.config.size);
        }

        if self.config.is_memory_locked {
            shm.memory_lock = Some(
                fail!(from self.config, when unsafe { MemoryLock::new(shm.base_address.cast(), shm.size) },
                        "{} since the memory lock failed.", msg),
            )
        }

        if self.config.zero_memory {
            if POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING {
                match SignalHandler::call_and_fetch(|| unsafe {
                    posix::memset(shm.base_address as *mut posix::void, 0, self.config.size);
                }) {
                    None => (),
                    Some(v) => {
                        fail!(from self.config, with SharedMemoryCreationError::InsufficientMemory,
                            "{} since a signal {} was raised while zeroing the memory. Is enough memory available on the system?", msg, v);
                    }
                }
            } else {
                unsafe { posix::memset(shm.base_address as *mut posix::void, 0, self.config.size) };
            }
        }

        trace!(from shm, "create");
        Ok(shm)
    }
}

/// A POSIX shared memory object which is build by the [`SharedMemoryBuilder`].
#[derive(Debug)]
pub struct SharedMemory {
    name: FileName,
    size: usize,
    base_address: *mut u8,
    has_ownership: bool,
    file_descriptor: FileDescriptor,
    memory_lock: Option<MemoryLock>,
}

impl Drop for SharedMemory {
    fn drop(&mut self) {
        if !self.base_address.is_null() {
            if unsafe { posix::munmap(self.base_address as *mut posix::void, self.size) } != 0 {
                fatal_panic!(from self, "This should never happen! Unable to unmap since the base address or range is invalid.");
            }
            trace!(from self, "close");
        }

        if self.has_ownership {
            match Self::shm_unlink(&self.name, UnlinkMode::PrintErrorWhenNotExisting) {
                Ok(_) => {
                    trace!(from self, "delete");
                }
                Err(_) => {
                    error!(from self, "Failed to cleanup shared memory.");
                }
            }
        }
    }
}

#[derive(PartialEq, Eq)]
enum UnlinkMode {
    QuietWhenNotExisting,
    PrintErrorWhenNotExisting,
}

impl SharedMemory {
    /// Returns true if the shared memory exists and is accessible, otherwise false.
    pub fn does_exist(name: &FileName) -> bool {
        let file_path =
            FilePath::from_path_and_file(&Path::new(&[PATH_SEPARATOR; 1]).unwrap(), name).unwrap();
        FileDescriptor::new(unsafe {
            posix::shm_open(
                file_path.as_c_str(),
                AccessMode::Read.as_oflag(),
                Permission::NONE.as_mode(),
            )
        })
        .is_some()
    }

    /// Returns if the posix implementation supports persistent shared memory, meaning that when every
    /// shared memory handle got out of scope the underlying OS resource remains.
    pub fn does_support_persistency() -> bool {
        POSIX_SUPPORT_PERSISTENT_SHARED_MEMORY
    }

    /// Returns true if the shared memory object has the ownership of the underlying posix shared
    /// memory. Ownership implies hereby that the posix shared memory is removed as soon as this
    /// object goes out of scope.
    pub fn has_ownership(&self) -> bool {
        self.has_ownership
    }

    /// Releases the ownership of the underlying posix shared memory. If the object goes out of
    /// scope the shared memory is no longer removed.
    pub fn release_ownership(&mut self) {
        self.has_ownership = false
    }

    /// Acquires the ownership of the underlying posix shared memory. If the object goes out of
    /// scope the shared memory will be removed.
    pub fn acquire_ownership(&mut self) {
        self.has_ownership = true
    }

    /// Removes a shared memory file.
    pub fn remove(name: &FileName) -> Result<(), SharedMemoryRemoveError> {
        match Self::shm_unlink(name, UnlinkMode::PrintErrorWhenNotExisting) {
            Ok(_) => {
                trace!(from "SharedMemory::remove", "\"{}\"", name);
                Ok(())
            }
            Err(v) => Err(v),
        }
    }

    /// Returns a list of all shared memory objects
    pub fn list() -> Vec<FileName> {
        let mut result = vec![];

        let raw_shm_names = unsafe { posix::shm_list() };
        for name in &raw_shm_names {
            match unsafe { FileName::from_c_str(name.as_ptr() as *mut i8) } {
                Ok(f) => result.push(f),
                Err(_) => (),
            }
        }

        result
    }

    /// returns the name of the shared memory
    pub fn name(&self) -> &FileName {
        &self.name
    }

    /// returns the base address of the shared memory. The base address is always aligned to the
    /// page size, this implies that it is aligned with every possible type.
    /// No further alignment required!
    pub fn base_address(&self) -> NonNull<u8> {
        match NonNull::new(self.base_address) {
            Some(v) => v,
            None => {
                fatal_panic!(from self,
                    "This should never happen! A valid shared memory object should never contain a base address with null value.");
            }
        }
    }

    /// returns the size of the shared memory
    pub fn size(&self) -> usize {
        self.size
    }

    /// returns a slice to the memory
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.base_address, self.size) }
    }

    /// returns a mutable slice to the memory
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.base_address, self.size) }
    }

    fn shm_create(
        name: &FileName,
        config: &SharedMemoryBuilder,
    ) -> Result<FileDescriptor, SharedMemoryCreationError> {
        let file_path =
            FilePath::from_path_and_file(&Path::new(&[PATH_SEPARATOR; 1]).unwrap(), name).unwrap();
        let fd = FileDescriptor::new(unsafe {
            posix::shm_open(
                file_path.as_c_str(),
                CreationMode::CreateExclusive.as_oflag() | config.access_mode.as_oflag(),
                config.permission.as_mode(),
            )
        });

        if let Some(v) = fd {
            return Ok(v);
        }

        let msg = "Unable to create shared memory";
        handle_errno!(SharedMemoryCreationError, from config,
            Errno::EACCES => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
            Errno::EINVAL => (InvalidName, "{} since the provided name \"{}\" is invalid.", msg, name),
            Errno::EEXIST => (AlreadyExist, "{} since it already exists.", msg),
            Errno::EMFILE => (PerProcessFileHandleLimitReached, "{} since the per-process file handle limit was reached.", msg),
            Errno::ENFILE => (SystemWideFileHandleLimitReached, "{} since the system-wide file handle limit was reached.", msg),
            Errno::ENAMETOOLONG => (NameTooLong, "{} since the name exceeds the maximum supported length of {}.", msg, Limit::MaxFileNameLength.value() ),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }

    fn shm_open(
        name: &FileName,
        config: &SharedMemoryBuilder,
        quiet_when_shm_does_not_exist: bool,
    ) -> Result<FileDescriptor, SharedMemoryCreationError> {
        let file_path =
            FilePath::from_path_and_file(&Path::new(&[PATH_SEPARATOR; 1]).unwrap(), name).unwrap();
        let fd = FileDescriptor::new(unsafe {
            posix::shm_open(
                file_path.as_c_str(),
                config.access_mode.as_oflag(),
                Permission::NONE.as_mode(),
            )
        });

        if let Some(v) = fd {
            return Ok(v);
        }

        let msg = "Unable to open shared memory";
        handle_errno!(SharedMemoryCreationError, from config,
            quiet_when quiet_when_shm_does_not_exist, Errno::ENOENT => (DoesNotExist, "{} since the shared memory does not exist.", msg),
            Errno::EACCES => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
            Errno::EINVAL => (InvalidName, "{} since the provided name \"{}\" is invalid.", msg, name),
            Errno::EMFILE => (PerProcessFileHandleLimitReached, "{} since the per-process file handle limit was reached.", msg),
            Errno::ENFILE => (SystemWideFileHandleLimitReached, "{} since the system-wide file handle limit was reached.", msg),
            Errno::ENAMETOOLONG => (NameTooLong, "{} since the name exceeds the maximum supported length of {}.", msg, Limit::MaxFileNameLength.value() ),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }

    fn mmap(
        file_descriptor: &FileDescriptor,
        config: &SharedMemoryBuilder,
    ) -> Result<*mut posix::void, SharedMemoryCreationError> {
        let base_address = unsafe {
            posix::mmap(
                std::ptr::null_mut::<posix::void>(),
                config.size,
                config.access_mode.as_protflag(),
                posix::MAP_SHARED,
                file_descriptor.native_handle(),
                0,
            )
        };

        if base_address != posix::MAP_FAILED {
            return Ok(base_address);
        }

        let msg = "Unable to map shared memory";
        handle_errno!(SharedMemoryCreationError, from config,
            Errno::EAGAIN => (InsufficientMemoryToBeMemoryLocked, "{} since a previous mlockall() enforces all mappings to be memory locked but this mapping cannot be locked due to insufficient memory.", msg),
            Errno::EINVAL => (UnsupportedSizeOfZero, "{} since a size of zero is not supported.", msg),
            Errno::EMFILE => (MappedRegionLimitReached, "{} since the number of mapped regions would exceed the process or system limit.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }

    fn shm_unlink(name: &FileName, mode: UnlinkMode) -> Result<(), SharedMemoryRemoveError> {
        let file_path =
            FilePath::from_path_and_file(&Path::new(&[PATH_SEPARATOR; 1]).unwrap(), name).unwrap();
        if unsafe { posix::shm_unlink(file_path.as_c_str()) } == 0 {
            return Ok(());
        }

        let msg = "Unable to remove shared memory device file";
        match posix::Errno::get() {
            posix::Errno::EACCES => {
                error!("{} \"{}\" due to insufficient permissions.", msg, name);
                Err(SharedMemoryRemoveError::InsufficientPermissions)
            }
            posix::Errno::ENOENT => {
                if mode == UnlinkMode::PrintErrorWhenNotExisting {
                    warn!(
                        "{} \"{}\" since the shared memory does not exist anymore.",
                        msg, name
                    );
                    Err(SharedMemoryRemoveError::DoesNotExist)
                } else {
                    Ok(())
                }
            }
            v => {
                error!(
                    "{} \"{}\" since an unknown error occurred ({}).",
                    msg, name, v
                );
                Err(SharedMemoryRemoveError::UnknownError(v as i32))
            }
        }
    }
}

impl FileDescriptorBased for SharedMemory {
    fn file_descriptor(&self) -> &FileDescriptor {
        &self.file_descriptor
    }
}

impl FileDescriptorManagement for SharedMemory {}
