use deno_core::CoreIsolate;
use deno_core::CoreIsolateState;
use deno_core::Op;
use deno_core::ResourceTable;
use deno_core::Script;
use deno_core::StartupData;
use deno_core::ZeroCopyBuf;

use pin_project::pin_project;
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::Context;
use std::task::Poll;

use std::str;
use std::sync::mpsc::channel;

#[pin_project]
pub struct Isolate {
    #[pin]
    pub core_isolate: CoreIsolate,
    pub state: State,
}

#[derive(Clone, Default, Deref)]
pub struct State(Rc<RefCell<StateInner>>);

#[derive(Default)]
pub struct StateInner {
    _resource_table: ResourceTable,
}

impl Future for Isolate {
    type Output = <CoreIsolate as Future>::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();
        this.core_isolate.poll(cx)
    }
}

impl Isolate {
    pub fn new() -> Self {
        let startup_data = StartupData::Script(Script {
            source: include_str!("../res/prelude.js"),
            filename: "prelude.js",
        });

        let isolate = Self {
            core_isolate: CoreIsolate::new(startup_data, false),
            state: Default::default(),
        };

        isolate
    }

    pub fn register_sync_op<F>(&mut self, name: &'static str, handler: F)
    where
        F: 'static + Fn(State, &mut [ZeroCopyBuf]) -> String, //Result<u32, Box<dyn Error>>,
    {
        let state = self.state.clone();
        let core_handler = move |_isolate_state: &mut CoreIsolateState,
                                 mut zero_copy_bufs: &mut [ZeroCopyBuf]|
              -> Op {
            // assert!(!zero_copy_bufs.is_empty());
            let state = state.clone();

            let result: String = handler(state, &mut zero_copy_bufs);

            // let buf: &[u8] = if zero_copy_bufs.len() > 0 { &zero_copy_bufs[0] } else { &[] };
            let buf = result.as_bytes();

            Op::Sync(Box::from(buf))
        };

        self.core_isolate.register_op(name, core_handler);
    }

    pub fn _register_op<F>(
        &mut self,
        name: &'static str,
        handler: impl Fn(State, &mut Box<[u8]>) -> F + Copy + 'static,
    ) where
        F: Future<Output = i32>,
    {
        let state = self.state.clone();
        let core_handler = move |_isolate_state: &mut CoreIsolateState,
                                 zero_copy_bufs: &mut [ZeroCopyBuf]|
              -> Op {
            assert!(!zero_copy_bufs.is_empty());
            let state = state.clone();
            let mut buf: Box<[u8]> = Box::from(&zero_copy_bufs[0] as &[u8]);

            let fut = async move {
                let _op = handler(state, &mut buf).await;
                buf
            };

            Op::Async(Box::pin(fut))
        };

        self.core_isolate.register_op(name, core_handler);
    }
}

pub fn run_string_script(script: &str, input: String) -> String {
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
        move |_, _zero_copy_bufs: &mut [ZeroCopyBuf]| input.clone(),
    );

    iso.core_isolate.execute("AfterPrelude", script).unwrap();

    rx.recv().unwrap()
}
