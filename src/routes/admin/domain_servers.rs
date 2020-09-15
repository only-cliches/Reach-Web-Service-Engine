use std::collections::HashMap;
use std::fs::read_dir;
use std::path::PathBuf;

use serde::Serialize;
use tokio::sync::oneshot;
use tokio::task;
use warp::{reject::Reject, reply::with_status, Filter, Rejection, Reply};
use hyper::Body::wrap_stream;
use walkdir::WalkDir;

use crate::database::kv_filter;
use crate::util::LogToOk;

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
    let data_route = warp::path("api").and(kv_filter(name.clone()));
    let hello = warp::any().map(|| "Hello!".to_string());
    let static_ = static_assets();
    let server = warp::serve(data_route.or(static_).or(hello));

    let (tx, rx) = oneshot::channel();
    let (addr, srv) = server.bind_with_graceful_shutdown(([127u8, 0, 0, 1], 0u16), async {
        rx.await.ok();
    });

    task::spawn(async {
        srv.await
    });

    Domain { port: addr.port(), name, shutdown_signal: tx }
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

fn static_assets() -> impl warp::Filter<Extract = (impl Reply, ), Error = Rejection> + Clone {

    let path: PathBuf = [".", "public"].iter().collect();

    let mut route =  warp::get()
        .and(warp::path("static-assets"))
        .and(warp::fs::dir(path))

    // let mut route = warp::path("static-assets")
    //     .and(warp::path::tail())
    //     .map(|path| {
    //         let mut path_buf = PathBuf::new();
    //         path_buf.push("apps");
    //
    //         let iter = path.as_str().split('/')
    //             .filter(|segment| segment != ".." && segment.len() > 0 );
    //
    //         let org = iter.next()?;
    //         let app = iter.next()?;
    //         let ver = iter.next()?;
    //
    //         path_buf.push(org.to_string_lossy());
    //         path_buf.push(app.to_string_lossy());
    //         path_buf.push("target");
    //         path_buf.push("client");
    //         path_buf.push(org.to_string_lossy());
    //         path_buf.push(app.to_string_lossy());
    //
    //         while let Some(segment) = iter.next() {
    //             path_buf.push(segment);
    //         };
    //
    //
    //
    //     }).map(|option_file: Option<String>| {
    //         match option_file {
    //             Some(string) => string,
    //             None => "404"
    //         }
    //     });
    //
    // route
    //
    // let mut walker = WalkDir::new("apps").min_depth(2).max_depth(2).into_iter();
    // let mut route = warp::path("static-assets");
    //
    // for line_result in walker {
    //     match line_result.log_to_ok() {
    //         Some(line) => {
    //             let mut line = line.into_path();
    //             line.push("target");
    //             line.push("client");
    //             route = route.and(warp::fs::dir(dbg!{line}));
    //         },
    //         None => ()
    //     }
    // }

    // let mut line = walker.next().unwrap().unwrap().into_path();
    // line.push("target");
    // line.push("client");
    // let route2 = route.and(warp::fs::dir(line));
    //
    // for maybe_line in walker {
    //     let line
    // }
    //
    // let mut line = walker.next().unwrap().unwrap().into_path();
    // line.push("target");
    // line.push("client");
    // route.and(warp::fs::dir(line))

    route.and(warp::post().map(|| "Nothing here"))

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

// pub fn get_app_names() -> Vec<String> {
//     let dir = read_dir("./apps").expect("The `apps` directory is missing or locked.");
//     dir.filter_map(|res| {
//         res.ok().and_then(|file| {
//             file.file_type().ok().and_then(|file_type| {
//                 if file_type.is_dir() {
//                     file.file_name().into_string().ok()
//                 } else {
//                     None
//                 }
//             })
//         })
//     })
//     .collect()
// }
