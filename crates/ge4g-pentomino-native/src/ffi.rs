//! Versioned additive ABI; sessions belong to the calling thread.
use crate::{BridgeError, ErrorCode, MAX_REQUEST_BYTES, NativeBridge, Response};
use std::{
    cell::RefCell,
    ffi::{CString, c_char},
    panic::{AssertUnwindSafe, catch_unwind},
};

thread_local! { static BRIDGE: RefCell<NativeBridge> = RefCell::new(NativeBridge::new()); }

#[unsafe(no_mangle)]
pub extern "C" fn ge4g_p4_v1_abi_version() -> u32 {
    1
}

/// Returns an owned NUL-terminated JSON response; free using this ABI's string_free.
///
/// # Safety
/// `data` must reference `len` readable bytes for the duration of the call.
/// Sessions must be used on their creating thread. A null pointer is rejected.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ge4g_p4_v1_request(data: *const u8, len: usize) -> *mut c_char {
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
        // SAFETY: the caller guarantees readable request memory; bounds checked above.
        let bytes = unsafe { std::slice::from_raw_parts(data, len) };
        BRIDGE.with(|bridge| bridge.borrow_mut().request(bytes))
    }))
    .unwrap_or_else(|_| Response {
        version: 1,
        ok: false,
        result: None,
        error: Some(BridgeError::new(
            ErrorCode::InternalFailure,
            "native request panicked",
        )),
    });
    // JSON encoding escapes embedded NULs, so a successful encoding is a C string.
    let bytes = serde_json::to_vec(&response).expect("response contains serializable bounded DTOs");
    CString::new(bytes)
        .expect("JSON has no literal NUL")
        .into_raw()
}

/// # Safety
/// `value` is null or a live pointer returned by ge4g_p4_v1_request, freed once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ge4g_p4_v1_string_free(value: *mut c_char) {
    if !value.is_null() {
        // SAFETY: ownership is transferred back by the caller exactly once.
        drop(unsafe { CString::from_raw(value) });
    }
}
