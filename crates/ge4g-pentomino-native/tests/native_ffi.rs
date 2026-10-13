//! Calls the exported versioned ABI directly and frees every owned response.
use ge4g_pentomino_native::{
    ErrorCode, ExecutionMode, FrameDto, InputRequest, NativeBridge, Response,
    ge4g_p4_v1_abi_version, ge4g_p4_v1_request, ge4g_p4_v1_string_free,
};
use serde_json::{Value, json};
use std::ffi::CStr;

const GAME: &[u8] = include_bytes!("../../../samples/pentomino_p4/game.json");

fn call_bytes(bytes: &[u8]) -> Response {
    // SAFETY: bytes are alive and readable until the call returns.
    let ptr = unsafe { ge4g_p4_v1_request(bytes.as_ptr(), bytes.len()) };
    assert!(!ptr.is_null());
    // SAFETY: this ABI returned an owned NUL-terminated response.
    let copy = unsafe { CStr::from_ptr(ptr) }.to_bytes().to_vec();
    // SAFETY: the live allocation is freed exactly once by its owning ABI.
    unsafe { ge4g_p4_v1_string_free(ptr) };
    serde_json::from_slice(&copy).unwrap()
}
fn call(command: Value) -> Response {
    call_bytes(&serde_json::to_vec(&json!({"version":1,"command":command})).unwrap())
}

#[test]
fn exported_abi_delivers_actual_native_frames_and_owned_response_lifetime() {
    assert_eq!(ge4g_p4_v1_abi_version(), 1);
    let config: Value = serde_json::from_slice(GAME).unwrap();
    let created = call(json!({"op":"create","config":config,"seed":73,"mode":"linked"}));
    assert!(created.ok);
    let session = created.result.unwrap()["session"].as_u64().unwrap();
    let mut rust = NativeBridge::new();
    let handle = rust.create(GAME, 73, ExecutionMode::Linked).unwrap();
    let stepped = call(
        json!({"op":"step","session":session,"input":{"version":1,"target_step":1,"controls":["attack"]}}),
    );
    assert!(stepped.ok);
    let expected_observation = rust
        .step(
            handle,
            InputRequest {
                version: 1,
                target_step: 1,
                controls: vec!["attack".into()],
            },
        )
        .unwrap();
    assert_eq!(
        stepped.result.unwrap(),
        serde_json::to_value(expected_observation).unwrap()
    );

    let request =
        serde_json::to_vec(&json!({"version":1,"command":{"op":"frame","session":session}}))
            .unwrap();
    // SAFETY: this request remains alive throughout the call.
    let first = unsafe { ge4g_p4_v1_request(request.as_ptr(), request.len()) };
    assert!(!first.is_null());
    // SAFETY: first remains allocated until freed below.
    let original = unsafe { CStr::from_ptr(first) }.to_bytes().to_vec();
    let other_response = call(json!({"op":"observe","session":session}));
    assert!(other_response.ok);
    // A later ABI call must not overwrite an earlier response buffer.
    // SAFETY: first has not yet been freed.
    assert_eq!(unsafe { CStr::from_ptr(first) }.to_bytes(), original);
    // SAFETY: first is freed exactly once after its final read.
    unsafe { ge4g_p4_v1_string_free(first) };
    let response: Response = serde_json::from_slice(&original).unwrap();
    assert!(response.ok);
    let frame: FrameDto = serde_json::from_value(response.result.unwrap()).unwrap();
    assert_eq!(frame, rust.frame(handle).unwrap());
    assert_eq!(
        frame.rgba.len(),
        frame.width as usize * frame.height as usize * 4
    );

    let saved = call(json!({"op":"save","session":session}));
    assert!(saved.ok);
    let save = saved.result.unwrap();
    assert!(call(json!({"op":"step","session":session,"input":{"version":1,"target_step":2,"controls":["attack"]}})).ok);
    assert!(call(json!({"op":"restore","session":session,"save":save})).ok);
    let restored_frame: FrameDto = serde_json::from_value(
        call(json!({"op":"frame","session":session}))
            .result
            .unwrap(),
    )
    .unwrap();
    assert_eq!(restored_frame, frame);
    assert!(call(json!({"op":"close","session":session})).ok);
    let closed = call(json!({"op":"frame","session":session}));
    assert!(!closed.ok);
    assert_eq!(closed.error.unwrap().code, ErrorCode::InvalidSession);
    let repeated = call(json!({"op":"close","session":session}));
    assert!(!repeated.ok);
    assert_eq!(repeated.error.unwrap().code, ErrorCode::InvalidSession);
}

#[test]
fn exported_abi_rejects_null_invalid_bytes_versions_and_bad_input_atomically() {
    // SAFETY: null is specifically supported as an invalid request sentinel.
    let ptr = unsafe { ge4g_p4_v1_request(std::ptr::null(), 10) };
    assert!(!ptr.is_null());
    // SAFETY: ptr is a live owned response allocation.
    let copy = unsafe { CStr::from_ptr(ptr) }.to_bytes().to_vec();
    // SAFETY: ptr is freed exactly once.
    unsafe { ge4g_p4_v1_string_free(ptr) };
    let null: Response = serde_json::from_slice(&copy).unwrap();
    assert!(!null.ok);
    assert!(null.error.is_some());
    // SAFETY: this ABI explicitly accepts a null free as a no-op.
    unsafe { ge4g_p4_v1_string_free(std::ptr::null_mut()) };
    for bytes in [
        b"".as_slice(),
        b"\xff".as_slice(),
        b"{".as_slice(),
        b"[]".as_slice(),
    ] {
        assert!(!call_bytes(bytes).ok);
    }
    let version = call_bytes(br#"{"version":2,"command":{"op":"observe","session":1}}"#);
    assert!(!version.ok);
    assert_eq!(version.error.unwrap().code, ErrorCode::VersionMismatch);
    let config: Value = serde_json::from_slice(GAME).unwrap();
    let created = call(json!({"op":"create","config":config,"seed":73,"mode":"linked"}));
    assert!(created.ok);
    let session = created.result.unwrap()["session"].as_u64().unwrap();
    let save = call(json!({"op":"save","session":session})).result.unwrap();
    let frame = call(json!({"op":"frame","session":session}))
        .result
        .unwrap();
    for controls in [
        json!([true]),
        json!(["unknown"]),
        json!(["attack", "attack"]),
    ] {
        let invalid = call(
            json!({"op":"step","session":session,"input":{"version":1,"target_step":1,"controls":controls}}),
        );
        assert!(!invalid.ok);
        assert_eq!(
            call(json!({"op":"save","session":session})).result.unwrap(),
            save
        );
        assert_eq!(
            call(json!({"op":"frame","session":session}))
                .result
                .unwrap(),
            frame
        );
    }
    let next = call(
        json!({"op":"step","session":session,"input":{"version":1,"target_step":1,"controls":["attack"]}}),
    );
    assert!(next.ok);
    assert_eq!(next.result.unwrap()["battle"]["hp"], json!([12, 7]));
    assert!(call(json!({"op":"close","session":session})).ok);
}
