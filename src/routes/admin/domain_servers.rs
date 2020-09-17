use std::path::PathBuf;

use serde::Serialize;
use tokio::sync::oneshot;
use tokio::task;
use warp::{ Filter, Rejection, Reply};

use crate::database::kv_filter;
// use crate::util::LogToOk;

type Port = u16;

#[derive(Serialize)]
pub struct Domain {
    port: Port,
    name: String,
    #[serde(skip)]
    shutdown_signal: oneshot::Sender<()>,
}

impl Domain {
    pub fn stop(self) {
        let Domain { name, shutdown_signal, .. } = self;
        match shutdown_signal.send(()) {
            Ok(_) => println!("Shut down server {}", name),
            Err(e) => eprintln!("Error shutting down server {}\n{:?}", name, e)
        };
    }
}

pub fn start(name: String) -> Domain {
    let path: PathBuf = [".", "apps", &name].iter().collect();
    let data_route = warp::path("api").and(warp::path("db")).and(kv_filter(name.clone()));
    let main = main_routes();
    let static_ = static_assets();
    let server = warp::serve(data_route.or(static_).or(main));

    let (tx, rx) = oneshot::channel();
    let (addr, srv) = server.bind_with_graceful_shutdown(([127u8, 0, 0, 1], 0u16), async {
        rx.await.ok();
    });

    task::spawn(async {
        srv.await
    });

    Domain { port: addr.port(), name, shutdown_signal: tx }

}

fn static_assets() -> impl warp::Filter<Extract = (impl Reply, ), Error = Rejection> + Clone {

    let path: PathBuf = [".", "public"].iter().collect();

    warp::get()
        .and(warp::path("static-assets"))
        .and(warp::fs::dir(path))

}

fn main_routes() -> impl warp::Filter<Extract = (impl Reply, ), Error = Rejection> + Clone {
    warp::get().map(|| {
        let title = "<title>This page</title>";
        let body_start = "<h1>This is the title</h1>";
        let body_end = "<p>The end</p>";
        format!(include_str!("../../../res/html/format.html"), title, body_start, body_end)
    })
}

// }
