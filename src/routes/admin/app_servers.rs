use std::collections::HashMap;
use std::fs::read_dir;
use std::path::PathBuf;

use warp::Filter;
use tokio::task;
use tokio::sync::oneshot;

type Port = u16;

const HTML: &str = "html_";

pub struct Domain {
    port: Port,
    name: String,
    shutdown_signal: oneshot::Sender<()>,
}

pub fn start(name: String) -> oneshot::Sender<()> {
    let path: PathBuf = [".", "apps", &name].iter().collect();
    let server = warp::serve(warp::any().map(|| {
        "Hello!".to_string()
    }));

    let (tx, rx) = oneshot::channel();

    task::spawn(async {
        let (addr, srv) = server.bind_with_graceful_shutdown(([127u8,0,0,1], 8084u16), async {
                rx.await.ok();
        });
        srv.await
    });

    tx
//    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
//    let port: Port = listener.local_addr().unwrap().port();

//    let srv = HttpServer::new(move || {
//        println!("SUBSERVER {} STARTED", path.display());
//        App::new().route("/", web::get().to(app_handler))
//    })
//    .listen(listener)
//    .unwrap()
//    .run();

    // let port: Port =
    //
    // (port, server)
}

pub async fn app_handler() -> String {
    "Hello\n".to_string()
}

// pub async fn start(name: String) -> Result<AppCluster, Box<dyn Error>> {
//     let path: PathBuf = [".", "apps", &name].iter().collect();
//     let servers: HashMap<Port, JoinHandle<()>> = read_dir(path)?
//         .filter_map(|maybe_entry| {
//             let entry = maybe_entry.ok()?;
//             let path = entry.path();
//             let name = entry.file_name();
//             let filetype = entry.file_type();
//
//             if filetype.ok()?.is_dir() && name.to_str()?.starts_with(HTML) {
//                 Some(path)
//             } else {
//                 None
//             }
//         })
//         .fold(
//             HashMap::new(),
//             |mut map: HashMap<Port, JoinHandle<()>>, path: PathBuf| {
//                 let handle = start_single(path);
//                 map.insert(1, handle);
//                 map
//             },
//         );
//
//     Ok(AppCluster { servers, name })
// }

pub fn get_app_names() -> Vec<String> {
    let dir = read_dir("./apps").expect("The `apps` directory is missing or locked.");
    dir.filter_map(|res| {
        res.ok().and_then(|file| {
            file.file_type().ok().and_then(|file_type| {
                if file_type.is_dir() {
                    file.file_name().into_string().ok()
                } else {
                    None
                }
            })
        })
    })
    .collect()
}
