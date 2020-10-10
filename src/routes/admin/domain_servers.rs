use std::path::PathBuf;

use serde::Serialize;
use tokio::sync::oneshot;
use tokio::task;
use warp::{Filter, Rejection, Reply};

use crate::database::{kv_filter, ADB};
use crate::util::LogToOk;

type Port = u16;

type DomainConfig = String;

//#[derive(Serialize)]
// pub struct DomainConfig {
//     apps: Vec<String>,
//     routes: Vec<(String, String)>,
//     data: String
// }

#[derive(Serialize)]
pub struct Domain {
    port: Port,
    name: String,
    init_data: DomainConfig, //rendered html file.
    #[serde(skip)]
    shutdown_signal: oneshot::Sender<()>,
}

impl Domain {
    pub fn stop(self) {
        let Domain {
            name,
            shutdown_signal,
            ..
        } = self;
        match shutdown_signal.send(()) {
            Ok(_) => println!("Shut down server {}", name),
            Err(e) => eprintln!("Error shutting down server {}\n{:?}", name, e),
        };
    }
}

pub fn start(name: String) -> Domain {
    let db_arc = ADB::new(&name);
    let db_resource = db_arc.clone();
    let db_middleware = warp::any().map(move || db_arc.clone());

    let data_route = warp::path("api")
        .and(warp::path("db"))
        .and(kv_filter(name.clone(), Some(db_resource)));
    let main = warp::any().and(db_middleware).map(get_main);
    let static_ = static_assets();
    let server = warp::serve(data_route.or(static_).or(main));

    let (tx, rx) = oneshot::channel();
    let (addr, srv) = server.bind_with_graceful_shutdown(([127u8, 0, 0, 1], 0u16), async {
        rx.await.ok();
    });

    task::spawn(async { srv.await });

    Domain {
        port: addr.port(),
        name,
        init_data: "\"Blank\"".to_string(),
        shutdown_signal: tx,
    }
}

pub fn static_assets() -> impl warp::Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let path: PathBuf = [".", "public"].iter().collect();

    warp::get()
        .and(warp::path("static-assets"))
        .and(warp::fs::dir(path))
}

fn get_main(db: ADB) -> String {
    let empty = "\"\"";

    let title = "<title>This page</title>";
    let config = db.get_value("config".to_string(), "domain".to_string())
        .map(|s| String::from_utf8(s).log_to_ok())
        .flatten()
        .unwrap_or_else(|| empty.to_string());

    let head = format!("{}\n<script>\n    var reachConfig = {}\n</script>", title, config);

    let body_start = "<h1>This is the title</h1>";
    let body_end = "<p>The end</p>";
    format!(
        include_str!("../../../root/res/html/format.html"),
        head, body_start, body_end
    )
}

// }
