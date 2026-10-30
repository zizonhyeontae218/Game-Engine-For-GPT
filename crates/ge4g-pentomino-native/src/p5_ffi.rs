//! Additive P5 C ABI, with owned UTF-8 responses and creating-thread sessions.
use crate::{BridgeError, ErrorCode, MAX_REQUEST_BYTES, Response};
use std::{
    cell::RefCell,
    ffi::{CString, c_char},
    panic::{AssertUnwindSafe, catch_unwind},
};
thread_local! { static BRIDGE:RefCell<crate::p5::NativeBridge>=RefCell::new(crate::p5::NativeBridge::new()); }
#[unsafe(no_mangle)]
pub extern "C" fn ge4g_p5_v1_abi_version() -> u32 {
    1
}
/// # Safety
/// `data` must point to `len` readable bytes for the call. Null is rejected.
/// All operations on a session must run on its creating thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ge4g_p5_v1_request(data: *const u8, len: usize) -> *mut c_char {
    let response = catch_unwind(AssertUnwindSafe(|| {
        if data.is_null() || len == 0 || len > MAX_REQUEST_BYTES {
            return Response {
                version: 1,
                ok: false,
                result: None,
                error: Some(BridgeError::new(
                    ErrorCode::BudgetExceeded,
                    "null, empty or oversized request",
                )),
            };
        }
        // SAFETY: caller supplies readable bytes and bounds were checked above.
        let bytes = unsafe { std::slice::from_raw_parts(data, len) };
        BRIDGE.with(|bridge| bridge.borrow_mut().request(bytes))
    }))
    .unwrap_or_else(|_| Response {
        version: 1,
        ok: false,
        result: None,
        error: Some(BridgeError::new(
            ErrorCode::InternalFailure,
            "P5 native request panicked",
        )),
    });
    CString::new(serde_json::to_vec(&response).expect("serializable response"))
        .expect("JSON escapes NUL")
        .into_raw()
}
/// # Safety
/// Null, or a live pointer returned by ge4g_p5_v1_request, freed exactly once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ge4g_p5_v1_string_free(value: *mut c_char) {
    if !value.is_null() {
        // SAFETY: caller transfers the live allocation back exactly once.
        drop(unsafe { CString::from_raw(value) });
    }
}
