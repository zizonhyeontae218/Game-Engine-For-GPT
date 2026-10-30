//! Actual versioned C ABI and ownership/thread checks, without private access.
use ge4g_pentomino_native::{
    ErrorCode, Response, ge4g_p4_v1_request, ge4g_p4_v1_string_free, ge4g_p5_v1_abi_version,
    ge4g_p5_v1_request, ge4g_p5_v1_string_free,
};
use serde_json::{Value, json};
use std::ffi::CStr;
fn call_bytes(bytes: &[u8]) -> Response {
    // SAFETY: request bytes remain readable during the call.
    let ptr = unsafe { ge4g_p5_v1_request(bytes.as_ptr(), bytes.len()) };
    assert!(!ptr.is_null());
    // SAFETY: ABI response is owned NUL-terminated UTF8 and still live.
    let copy = unsafe { CStr::from_ptr(ptr) }.to_bytes().to_vec();
    // SAFETY: this frees exactly the allocation returned above, once.
    unsafe { ge4g_p5_v1_string_free(ptr) };
    serde_json::from_slice(&copy).unwrap()
}
fn call(command: Value) -> Response {
    call_bytes(&serde_json::to_vec(&json!({"version":1,"command":command})).unwrap())
}
fn create() -> u64 {
    let f = |id: &str, team: usize, x: i64| json!({"id":id,"team":team,"hp":10,"resource":3,"attack":4,"defense":1,"speed":2,"position":[x,72],"color":[48,144,255,255]});
    let r = call(
        json!({"op":"create","config":{"version":2,"width":160,"height":96,"tick_hz":60,"fighters":[f("blue",0,40),f("red",1,120)],"actions":[{"id":"strike","kind":"damage","target":"enemy","cost":0,"power":2,"effect":null,"duration":0}]},"seed":73,"mode":"linked"}),
    );
    assert!(r.ok, "{r:?}");
    r.result.unwrap()["session"].as_u64().unwrap()
}
#[test]
fn actual_c_calls_preserve_frame_buffers_and_validate_thread_and_abi_separation() {
    assert_eq!(ge4g_p5_v1_abi_version(), 1);
    let session = create();
    let stepped = call(
        json!({"op":"act","session":session,"command":{"id":1,"actor":0,"action":0,"target":1}}),
    );
    assert!(stepped.ok);
    assert_eq!(stepped.result.unwrap()["battle"]["fighters"][1]["hp"], 5);
    let request =
        serde_json::to_vec(&json!({"version":1,"command":{"op":"frame","session":session}}))
            .unwrap();
    // SAFETY: this request is readable until the call returns.
    let first = unsafe { ge4g_p5_v1_request(request.as_ptr(), request.len()) };
    // SAFETY: first remains live until its owning free below.
    let original = unsafe { CStr::from_ptr(first) }.to_bytes().to_vec();
    assert!(call(json!({"op":"observe","session":session})).ok);
    // SAFETY: no prior free; later requests must not overwrite owned responses.
    assert_eq!(unsafe { CStr::from_ptr(first) }.to_bytes(), original);
    // SAFETY: first response freed once by its owning ABI.
    unsafe { ge4g_p5_v1_string_free(first) };
    let response: Response = serde_json::from_slice(&original).unwrap();
    let frame = response.result.unwrap();
    assert_eq!(frame["rgba"].as_array().unwrap().len(), 160 * 96 * 4);
    let other = std::thread::spawn(move || call(json!({"op":"observe","session":session})))
        .join()
        .unwrap();
    assert_eq!(other.error.unwrap().code, ErrorCode::InvalidSession);
    let p4_bytes =
        serde_json::to_vec(&json!({"version":1,"command":{"op":"observe","session":session}}))
            .unwrap();
    // SAFETY: valid bytes and correctly matched P4 allocation/free pair.
    let p4 = unsafe { ge4g_p4_v1_request(p4_bytes.as_ptr(), p4_bytes.len()) };
    // SAFETY: live P4 response allocation.
    let p4_response: Response =
        serde_json::from_slice(unsafe { CStr::from_ptr(p4) }.to_bytes()).unwrap();
    // SAFETY: matched allocator and single free.
    unsafe { ge4g_p4_v1_string_free(p4) };
    assert_eq!(p4_response.error.unwrap().code, ErrorCode::InvalidSession);
    let save = call(json!({"op":"save","session":session})).result.unwrap();
    assert!(
        call(
            json!({"op":"act","session":session,"command":{"id":2,"actor":1,"action":0,"target":0}})
        )
        .ok
    );
    assert!(call(json!({"op":"restore","session":session,"save":save})).ok);
    assert_eq!(
        call(json!({"op":"frame","session":session}))
            .result
            .unwrap(),
        frame
    );
    assert!(call(json!({"op":"close","session":session})).ok);
    assert!(!call(json!({"op":"close","session":session})).ok);
}
#[test]
fn null_unknown_fields_invalid_version_and_duplicate_input_are_atomic() {
    // SAFETY: null is a documented rejected sentinel, not dereferenced.
    let ptr = unsafe { ge4g_p5_v1_request(std::ptr::null(), 1) };
    // SAFETY: owned ABI response still live.
    let response: Response =
        serde_json::from_slice(unsafe { CStr::from_ptr(ptr) }.to_bytes()).unwrap();
    // SAFETY: matched ABI single free.
    unsafe { ge4g_p5_v1_string_free(ptr) };
    assert_eq!(response.error.unwrap().code, ErrorCode::BudgetExceeded);
    assert!(!call_bytes(b"not json").ok);
    assert!(!call_bytes(br#"{"version":2,"command":{"op":"observe","session":1}}"#).ok);
    let session = create();
    let before = call(json!({"op":"save","session":session})).result;
    assert!(!call(json!({"op":"observe","session":session,"unknown":true})).ok);
    assert!(!call(json!({"op":"act","session":session,"command":{"id":1,"actor":0,"action":0,"target":1,"unknown":0}})).ok);
    assert_eq!(call(json!({"op":"save","session":session})).result, before);
    assert!(
        call(
            json!({"op":"act","session":session,"command":{"id":1,"actor":0,"action":0,"target":1}})
        )
        .ok
    );
    let after = call(json!({"op":"save","session":session})).result;
    assert!(
        !call(
            json!({"op":"act","session":session,"command":{"id":1,"actor":1,"action":0,"target":0}})
        )
        .ok
    );
    assert_eq!(call(json!({"op":"save","session":session})).result, after);
    assert!(call(json!({"op":"close","session":session})).ok);
}
