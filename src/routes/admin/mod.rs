use http::method::Method;
use warp::{ Filter, Rejection, Reply, reply::with_status, reject::Reject };
use std::convert::Infallible;

mod app_servers;

// This is a private, hidden module containing a static admin configuration object. This is built
// this way to prevent using these functions in any route except /api/admin, also defined below.
mod config {

    use super::app_servers;
    use crate::app_compiler::file_watcher;

    use serde::Serialize;
    use tokio::sync::RwLock;
    use tokio::sync::oneshot;

    use std::collections::HashMap;
    use std::error::Error;

    lazy_static! {
        static ref CONFIGURATION: RwLock<AdminState> = RwLock::new(AdminState::new());
    }

    pub struct AdminState {
        compile_watcher: file_watcher::MaybeWatcher,
        servers: HashMap<String, oneshot::Sender<()>>,
    }

    impl AdminState {
        pub fn new() -> AdminState {
            let servers: HashMap<String, oneshot::Sender<()>> = HashMap::new();
            AdminState {
                compile_watcher: file_watcher::start(None),
                servers,
            }
        }
    }

    #[derive(Serialize)]
    pub struct ServerStatus {
        name: String,
        is_running: bool,
    }

    pub async fn start_file_watcher() {
        let mut config = CONFIGURATION.write().await;
        let watcher = config.compile_watcher.take();
        config.compile_watcher = file_watcher::start(watcher);
    }

    pub async fn stop_file_watcher() {
        let mut config = CONFIGURATION.write().await;
        let watcher = config.compile_watcher.take();
        config.compile_watcher = file_watcher::stop(watcher);
    }

    pub async fn get_app_statuses(names: Vec<String>) -> Vec<ServerStatus> {
        let config = CONFIGURATION.read().await;
        names
            .into_iter()
            .map(|name| {
                let is_running = config.servers.contains_key(&name);
                ServerStatus { name, is_running }
            })
            .collect()
    }

    pub async fn start_app_server(name: String) -> Result<bool, Box<dyn Error>> {
        let mut config = CONFIGURATION.write().await;
        if config.servers.contains_key(&name) {
            Ok(false)
        } else {
            println!("{:?}", name);
            let srv = app_servers::start(name.clone());
            config.servers.insert(name, srv);
            Ok(true)
        }
    }

    pub async fn stop_app_server(name: String) {
        let mut config = CONFIGURATION.write().await;
        let srv: Option<oneshot::Sender<()>> = config.servers.remove(&name);
        srv.map(|server| ());
    }

    pub async fn init() {
        lazy_static::initialize(&CONFIGURATION);
        let mut config = CONFIGURATION.write().await;
        let panel = "control-panel".to_string();
        let srv = app_servers::start(panel.clone());
        config.servers.insert(panel, srv);
    }
}

async fn file_watcher(method: Method) -> Result<impl warp::Reply, Rejection> {
    match method {
        Method::PUT => {
            config::start_file_watcher().await;
            Ok(with_status("Started", http::StatusCode::OK))
        },
        Method::DELETE => {
            config::stop_file_watcher().await;
            Ok(with_status("Stopped", http::StatusCode::OK))
        },
        _ => Ok(with_status("Invalid method", http::StatusCode::METHOD_NOT_ALLOWED))
    }
}

#[derive(Debug)]
enum AdminError {
    AppSerializeError,
    DomainLoadingError,
}
impl Reject for AdminError {}

async fn get_apps() -> Result<String, Rejection> {
    let apps = app_servers::get_app_names();
    let app_statuses = config::get_app_statuses(apps).await;
    serde_json::to_string(&app_statuses).map_err(|e| warp::reject::custom(AdminError::AppSerializeError))
}

async fn put_app( app_name: String ) -> Result<String, Rejection> {
    config::start_app_server(app_name.clone())
        .await
        .map(|is_new| {
            if is_new {
                format!("Loaded app {}", app_name).to_string()
            } else {
                format!("App {} was already running", app_name).to_string()
            }
        })
        .map_err(|e| {
            eprintln!("Error loading {}:\n{:?}", app_name, e);
            warp::reject::custom(AdminError::DomainLoadingError)
        })
}

async fn delete_app( name: String ) -> Result<String, Infallible> {
    config::stop_app_server(name).await;
    Ok("Closed".into())
}

pub async fn init() {
    config::init().await;
}

// Returning a vec means that the whole set of admin resources can be built all at once here, and
// attached to the app where needed.
pub fn resources() -> impl warp::Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let admin_path = warp::path("api")
            .and(warp::path("admin"));

    let file_watcher = admin_path
        .and(warp::path("file_watcher"))
        .and(warp::method())
        .and_then(file_watcher);

    let apps = admin_path
        .and(warp::path("apps"));

    let get_apps_route = apps
        .and(warp::get())
        .and_then(get_apps);

    let put_app_route = apps
        .and(warp::path::param())
        .and(warp::put())
        .and_then(put_app);

    let delete_app_route = apps
        .and(warp::path::param())
        .and(warp::delete())
        .and_then(delete_app);

    file_watcher.or(get_apps_route).or(put_app_route).or(delete_app_route)

}
