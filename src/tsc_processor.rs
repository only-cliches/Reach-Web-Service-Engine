use deno_core::ZeroCopyBuf;

use std::str;
use std::sync::mpsc::channel;

use crate::deno_runtime::Isolate;

pub fn run(compilation_string: String) -> String {
    let (tx, rx) = channel();

    let mut iso: Isolate = Isolate::new();

    iso.register_sync_op(
        "return_string",
        move |_, zero_copy_bufs: &mut [ZeroCopyBuf]| {
            let buf = zero_copy_bufs[0].clone();
            let result = str::from_utf8(&*buf).unwrap().to_owned();
            tx.send(result).unwrap();
            "".to_owned()
        },
    );

    iso.register_sync_op(
        "get_string",
        move |_, _zero_copy_bufs: &mut [ZeroCopyBuf]| compilation_string.clone(),
    );

    iso.core_isolate
        .execute("RequireJS", include_str!("../res/require.js"))
        .unwrap();

    iso.core_isolate
        .execute("Typescript", include_str!("../res/typescript/index.min.js"))
        .unwrap();

    iso.core_isolate
        .execute("Retrieve and process", include_str!("../res/typescript/deno_adapter.js"))
        .unwrap();

    rx.recv().unwrap()
}
