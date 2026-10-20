//! C-ABI Foreign Function Interface (FFI) for DQL
//! Memungkinkan bahasa lain (Python, Node.js, Go, C#, C++) memanggil DQL secara langsung.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use crate::compiler::{compile, Dialect};

/// Mengompilasi string query DQL menjadi SQL dialek tertentu.
/// Mengembalikan pointer string C yang harus dibebaskan menggunakan `dql_free_string`.
/// Jika terjadi kesalahan, fungsi mengembalikan NULL dan mengisi `error_out` dengan pesan kesalahan.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dql_compile(
    query_ptr: *const c_char,
    dialect_ptr: *const c_char,
    parameterized: bool,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    if query_ptr.is_null() {
        return std::ptr::null_mut();
    }

    let query_str = match unsafe { CStr::from_ptr(query_ptr) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = CString::new("String kueri bukan UTF-8 yang valid").unwrap().into_raw();
                }
            }
            return std::ptr::null_mut();
        }
    };

    let dialect = if dialect_ptr.is_null() {
        Dialect::Postgres
    } else {
        match unsafe { CStr::from_ptr(dialect_ptr) }.to_str() {
            Ok(s) => Dialect::parse(s).unwrap_or(Dialect::Postgres),
            Err(_) => Dialect::Postgres,
        }
    };

    match compile(query_str, dialect, parameterized) {
        Ok(res) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = std::ptr::null_mut();
                }
            }
            CString::new(res.sql).unwrap().into_raw()
        }
        Err(err) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = CString::new(err).unwrap().into_raw();
                }
            }
            std::ptr::null_mut()
        }
    }
}

/// Membebaskan memori string yang dialokasikan oleh DQL C-ABI
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dql_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            drop(CString::from_raw(ptr));
        }
    }
}
