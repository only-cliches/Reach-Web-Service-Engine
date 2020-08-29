use std::collections::HashMap;
use std::error::Error;
use std::fs::read_dir;
use std::net::TcpListener;
use std::path::PathBuf;

use actix_web::{dev::Server, web, App, HttpRequest, HttpServer};

type Port = u16;

const HTML: &str = "html_";

pub struct AppCluster {
    servers: HashMap<u16, Server>,
    name: String,
}

impl AppCluster {
    pub fn stop(self) {

        for server in self.servers.values() {
            server.stop(true);
        }

    }

    // pub fn new (name: String, servers: Vec<PathBuf>) -> Self {
    //     AppCluster { servers: HashMap::new(), name }
    // }
}

fn start_single(path: PathBuf) -> (Port, Server) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port: Port = listener.local_addr().unwrap().port();

    let srv = HttpServer::new(move || {
        println!("SUBSERVER {} STARTED", path.display());
        App::new().route("/", web::get().to(app_handler))
    })
    .listen(listener)
    .unwrap()
    .run();

    (port, srv)
}

pub async fn app_handler(
    _req: HttpRequest,
    _body: actix_web::web::Payload,
) -> actix_web::HttpResponse {
    "Hello\n".into()
}

pub async fn start(name: String) -> Result<AppCluster, Box<dyn Error>> {
    let path: PathBuf = [".", "apps", &name].iter().collect();
    let servers: HashMap<Port, Server> = read_dir(path)?
        .filter_map(|maybe_entry| {
            let entry = maybe_entry.ok()?;
            let path = entry.path();
            let name = entry.file_name();
            let filetype = entry.file_type();

            if filetype.ok()?.is_dir() && name.to_str()?.starts_with(HTML) {
                Some(path)
            } else {
                None
            }
        })
        .fold(
            HashMap::new(),
            |mut map: HashMap<Port, Server>, path: PathBuf| {
                let (port, server) = start_single(path);
                map.insert(port, server);
                map
            },
        );

    Ok(AppCluster { servers, name })
}

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
