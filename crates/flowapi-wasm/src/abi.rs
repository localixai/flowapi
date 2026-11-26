//! The module's exports: the host asks for a buffer, writes its request into it, and calls
//! `flowapi_generate`, `flowapi_references` or, in the second module (`sdk`), `flowapi_sdk`,
//! which take the buffer back and return a new one,
//! the answer: four bytes of its length (little-endian), then its UTF-8 JSON. The host reads it
//! and gives it back with `flowapi_free`. Every buffer is a boxed slice, freed with the length
//! it was made with; nothing is kept between calls (G6).
//!
//! In JavaScript (the npm package's `src/wasm.ts`):
//!
//! ```js
//! const at = flowapi_alloc(bytes.length)
//! new Uint8Array(memory.buffer, at, bytes.length).set(bytes)
//! const answer = flowapi_generate(at, bytes.length) // `at` is the module's again
//! const length = new DataView(memory.buffer).getUint32(answer, true)
//! const json = decoder.decode(new Uint8Array(memory.buffer, answer + 4, length))
//! flowapi_free(answer, length + 4)
//! ```
#![expect(
    unsafe_code,
    reason = "an export keeps its name (`no_mangle`), and a buffer the host hands back is \
              rebuilt from its pointer and length: the module's whole ABI, and only here"
)]

use std::ptr;

/// A buffer of `len` zeroed bytes for the host to write a request into. It is the module's
/// again once handed to `flowapi_generate`, `flowapi_references` or `flowapi_sdk`, or given to
/// `flowapi_free`.
#[unsafe(no_mangle)]
pub extern "C" fn flowapi_alloc(len: usize) -> *mut u8 {
    Box::into_raw(vec![0_u8; len].into_boxed_slice()).cast::<u8>()
}

/// Frees a buffer: an answer (its length is the four bytes' value plus four) or an unused one
/// from `flowapi_alloc`.
///
/// # Safety
///
/// `ptr` and `len` are a buffer this module made and has not freed or taken back.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn flowapi_free(ptr: *mut u8, len: usize) {
    drop(unsafe { take(ptr, len) });
}

/// Generates from the request in a buffer (`crate::generate`) and returns the answer's buffer.
///
/// # Safety
///
/// As `flowapi_free`: the buffer is taken back and freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn flowapi_generate(ptr: *mut u8, len: usize) -> *mut u8 {
    let request = unsafe { take(ptr, len) };
    give(&crate::generate(&request))
}

/// The references of the document in a buffer (`crate::references`: its URI and text as JSON),
/// as an answer's buffer.
///
/// # Safety
///
/// As `flowapi_free`: the buffer is taken back and freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn flowapi_references(ptr: *mut u8, len: usize) -> *mut u8 {
    let document = unsafe { take(ptr, len) };
    give(&crate::references(&document))
}

/// Runs the SDK command in a buffer (`crate::sdk`: `summary`, `draft`, `schema`, `spec`, `check`,
/// `repo` or `plan`) and returns the answer's buffer. Only in the second module (the `sdk` feature).
///
/// # Safety
///
/// As `flowapi_free`: the buffer is taken back and freed.
#[cfg(feature = "sdk")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn flowapi_sdk(ptr: *mut u8, len: usize) -> *mut u8 {
    let request = unsafe { take(ptr, len) };
    give(&crate::sdk(&request))
}

/// A buffer this module made, taken back.
///
/// # Safety
///
/// As `flowapi_free`.
unsafe fn take(ptr: *mut u8, len: usize) -> Box<[u8]> {
    unsafe { Box::from_raw(ptr::slice_from_raw_parts_mut(ptr, len)) }
}

/// An answer's buffer: its length in four bytes, then its text. A wasm32 memory holds at most
/// 4 GiB, so the length always fits.
fn give(text: &str) -> *mut u8 {
    let len = u32::try_from(text.len()).unwrap_or(u32::MAX);
    let mut bytes = Vec::with_capacity(text.len() + 4);
    bytes.extend_from_slice(&len.to_le_bytes());
    bytes.extend_from_slice(text.as_bytes());
    Box::into_raw(bytes.into_boxed_slice()).cast::<u8>()
}
