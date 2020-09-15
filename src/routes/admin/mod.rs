use http::method::Method;
use std::convert::Infallible;
use warp::{reject::Reject, reply::with_status, Filter, Rejection, Reply};

mod domain_servers;

// This is a private, hidden module containing a static admin configuration object. This is built
// this way to prevent using these functions in any route except /api/admin, also defined below.
mod config {

    use super::domain_servers;
    use super::domain_servers::Domain;
    use crate::app_compiler::file_watcher;

    use serde::Serialize;
    use tokio::sync::RwLock;

    use std::collections::HashMap;
    use std::error::Error;

    lazy_static! {
        static ref CONFIGURATION: RwLock<AdminState> = RwLock::new(AdminState::new());
    }

    pub struct AdminState {
        compile_watcher: file_watcher::MaybeWatcher,
        servers: HashMap<String, Domain>,
    }

    impl AdminState {
        pub fn new() -> AdminState {
            let servers: HashMap<String, Domain> = HashMap::new();
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

    pub async fn get_domain_statuses() -> Result<String, serde_json::Error> {
        let config = CONFIGURATION.read().await;
        serde_json::to_string(&config.servers)
        // names
        //     .into_iter()
        //     .map(|name| {
        //         let is_running = config.servers.contains_key(&name);
        //         ServerStatus { name, is_running }
        //     })
        //     .collect()
    }

    pub async fn start_domain_server(name: String) -> Result<bool, Box<dyn Error>> {
        let mut config = CONFIGURATION.write().await;
        if config.servers.contains_key(&name) {
            Ok(false)
        } else {
            println!("{} not already active", name);
            let srv = domain_servers::start(name.clone());
            config.servers.insert(name, srv);
            Ok(true)
        }
    }

    pub async fn stop_domain_server(name: String) {
        let mut config = CONFIGURATION.write().await;
        let srv: Option<Domain> = config.servers.remove(&name);
        srv.map(|server| {
            server.stop();
        });
    }

    pub async fn init() {
        lazy_static::initialize(&CONFIGURATION);
        let mut config = CONFIGURATION.write().await;
        let panel = "control-panel".to_string();
        let srv = domain_servers::start(panel.clone());
        config.servers.insert(panel, srv);
    }
}

async fn file_watcher(method: Method) -> Result<impl warp::Reply, Rejection> {
    match method {
        Method::PUT => {
            config::start_file_watcher().await;
            Ok(with_status("Started", http::StatusCode::OK))
        }
        Method::DELETE => {
            config::stop_file_watcher().await;
            Ok(with_status("Stopped", http::StatusCode::OK))
        }
        _ => Ok(with_status(
            "Invalid method",
            http::StatusCode::METHOD_NOT_ALLOWED,
        )),
    }
}

#[derive(Debug)]
enum AdminError {
    AppSerializeError,
    DomainLoadingError,
}
impl Reject for AdminError {}

async fn get_domains() -> Result<String, Rejection> {
    config::get_domain_statuses().await
        .map_err(|e| warp::reject::custom(AdminError::AppSerializeError))
}

async fn put_domain(domain_name: String) -> Result<String, Rejection> {
    config::start_domain_server(domain_name.clone())
        .await
        .map(|is_new| {
            if is_new {
                format!("Loaded domain {}", domain_name).to_string()
            } else {
                format!("App {} was already running", domain_name).to_string()
            }
        })
        .map_err(|e| {
            eprintln!("Error loading {}:\n{:?}", domain_name, e);
            warp::reject::custom(AdminError::DomainLoadingError)
        })
}

async fn delete_domain(name: String) -> Result<String, Infallible> {
    config::stop_domain_server(name).await;
    Ok("Closed".into())
}

pub async fn init() {
    config::init().await;

    put_domain("example_domain".to_string()).await.unwrap();
}

// Returning a vec means that the whole set of admin resources can be built all at once here, and
// attached to the domain where needed.
pub fn resources() -> impl warp::Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let admin_path = warp::path("api").and(warp::path("admin"));

    let file_watcher = admin_path
        .and(warp::path("file_watcher"))
        .and(warp::method())
        .and_then(file_watcher);

    let domains = admin_path.and(warp::path("domains"));

    let get_domains_route = domains.and(warp::get()).and_then(get_domains);

    let put_domain_route = domains
        .and(warp::path::param())
        .and(warp::put())
        .and_then(put_domain);

    let delete_domain_route = domains
        .and(warp::path::param())
        .and(warp::delete())
        .and_then(delete_domain);

    file_watcher
        .or(get_domains_route)
        .or(put_domain_route)
        .or(delete_domain_route)
}
