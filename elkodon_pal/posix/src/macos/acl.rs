#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]

use crate::posix::types::*;

pub unsafe fn acl_get_perm(permset: acl_permset_t, perm: acl_perm_t) -> int {
    //crate::internal::acl_get_perm(permset, perm)
    -1
}

pub unsafe fn acl_init(count: int) -> acl_t {
    //crate::internal::acl_init(count)
    0
}

pub unsafe fn acl_free(data: *mut void) -> int {
    //crate::internal::acl_free(data)
    -1
}

pub unsafe fn acl_valid(acl: acl_t) -> int {
    //crate::internal::acl_valid(acl)
    -1
}

pub unsafe fn acl_create_entry(acl: *mut acl_t, entry: *mut acl_entry_t) -> int {
    //crate::internal::acl_create_entry(acl, entry)
    -1
}

pub unsafe fn acl_get_entry(acl: acl_t, entry_id: int, entry: *mut acl_entry_t) -> int {
    //crate::internal::acl_get_entry(acl, entry_id, entry)
    -1
}

pub unsafe fn acl_add_perm(permset: acl_permset_t, perm: acl_perm_t) -> int {
    //crate::internal::acl_add_perm(permset, perm)
    -1
}

pub unsafe fn acl_clear_perms(permset: acl_permset_t) -> int {
    //crate::internal::acl_clear_perms(permset)
    -1
}

pub unsafe fn acl_get_permset(entry: acl_entry_t, permset: *mut acl_permset_t) -> int {
    //crate::internal::acl_get_permset(entry, permset)
    -1
}

pub unsafe fn acl_set_permset(entry: acl_entry_t, permset: acl_permset_t) -> int {
    //crate::internal::acl_set_permset(entry, permset)
    -1
}

pub unsafe fn acl_get_qualifier(entry: acl_entry_t) -> *mut void {
    //crate::internal::acl_get_qualifier(entry)
    0 as *mut void
}

pub unsafe fn acl_set_qualifier(entry: acl_entry_t, tag_qualifier: *const void) -> int {
    //crate::internal::acl_set_qualifier(entry, tag_qualifier)
    -1
}

pub unsafe fn acl_get_tag_type(entry: acl_entry_t, acl_tag_type: *mut acl_tag_t) -> int {
    //crate::internal::acl_get_tag_type(entry, acl_tag_type)
    -1
}

pub unsafe fn acl_set_tag_type(entry: acl_entry_t, acl_tag_type: acl_tag_t) -> int {
    //crate::internal::acl_set_tag_type(entry, acl_tag_type)
    -1
}

pub unsafe fn acl_get_fd(fd: int) -> acl_t {
    //crate::internal::acl_get_fd(fd)
    0
}

pub unsafe fn acl_set_fd(fd: int, acl: acl_t) -> int {
    //crate::internal::acl_set_fd(fd, acl)
    -1
}

pub unsafe fn acl_to_text(acl: acl_t, len_p: *mut ssize_t) -> *const char {
    //crate::internal::acl_to_text(acl, len_p)
    0 as *const char
}

pub unsafe fn acl_from_text(buf_p: *const char) -> acl_t {
    //crate::internal::acl_from_text(buf_p)
    0
}
