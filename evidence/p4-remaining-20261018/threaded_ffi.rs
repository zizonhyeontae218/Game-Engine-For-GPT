//! Additive QA test copied into native/tests only in the Linux job.
use ge4g_pentomino_native::{ge4g_p4_v1_request, ge4g_p4_v1_string_free};
use serde_json::{Value, json};
use std::{collections::BTreeSet, ffi::CStr, sync::{Arc, Barrier}};

fn call(command: Value) -> Value {
    let bytes=serde_json::to_vec(&json!({"version":1,"command":command})).unwrap();
    // SAFETY: input bytes remain readable, response is copied before exactly one free.
    unsafe {
        let p=ge4g_p4_v1_request(bytes.as_ptr(),bytes.len());assert!(!p.is_null());
        let result=serde_json::from_slice(CStr::from_ptr(p).to_bytes()).unwrap();
        ge4g_p4_v1_string_free(p);result
    }
}

#[test]
fn independent_thread_sessions_global_handles_and_exit_cleanup() {
    let barrier=Arc::new(Barrier::new(8));
    let workers:Vec<_>=(0..8).map(|thread| {
        let barrier=barrier.clone();
        std::thread::spawn(move || {
            let cfg:Value=serde_json::from_str(include_str!("../../../samples/pentomino_p4/game.json")).unwrap();
            barrier.wait();let mut handles=Vec::new();
            for cycle in 0..16 {
                let created=call(json!({"op":"create","config":cfg,"seed":thread*100+cycle,"mode":"linked"}));
                assert_eq!(created["ok"],true);let h=created["result"]["session"].as_u64().unwrap();handles.push(h);
                for step in 1..=6 {
                    let r=call(json!({"op":"step","session":h,"input":{"version":1,"target_step":step,"controls":[]}}));
                    assert_eq!(r["ok"],true);assert_eq!(r["result"]["core_tick"],step*2);
                }
                let frame=call(json!({"op":"frame","session":h}));assert_eq!(frame["ok"],true);
                assert_eq!(frame["result"]["rgba"].as_array().unwrap().len(),160*96*4);
                if cycle<15 {
                    assert_eq!(call(json!({"op":"close","session":h}))["ok"],true);
                    assert_eq!(call(json!({"op":"observe","session":h}))["error"]["code"],"invalid_session");
                } // Last session deliberately reclaimed by thread-local destruction.
            }
            handles
        })
    }).collect();
    let handles:Vec<_>=workers.into_iter().flat_map(|w|w.join().unwrap()).collect();
    assert_eq!(handles.len(),128);assert_eq!(handles.iter().collect::<BTreeSet<_>>().len(),128);
    for h in handles {assert_eq!(call(json!({"op":"observe","session":h}))["error"]["code"],"invalid_session");}
}
