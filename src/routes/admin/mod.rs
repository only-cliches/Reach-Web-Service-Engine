use actix_web::{http::StatusCode, web, HttpRequest, HttpResponse, Resource};
use std::error::Error;

mod app_servers;

// This is a private, hidden module containing a static admin configuration object. This is built
// this way to prevent using these functions in any route except /api/admin, also defined below.
mod config {

    use super::app_servers;
    use crate::app_compiler::file_watcher;

    use actix_web::dev::Server;
    use serde::Serialize;
    use tokio::sync::RwLock;

    use std::collections::HashMap;
    use std::error::Error;

    lazy_static! {
        static ref CONFIGURATION: RwLock<AdminState> = RwLock::new(AdminState::new());
    }

    pub struct AdminState {
        compile_watcher: file_watcher::MaybeWatcher,
        servers: HashMap<String, app_servers::AppCluster>,
    }

    impl AdminState {
        pub fn new() -> AdminState {
            let servers: HashMap<String, app_servers::AppCluster> = HashMap::new();
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
            let srv: app_servers::AppCluster = app_servers::start(name.clone()).await?;
            config.servers.insert(name, srv);
            Ok(true)
        }
    }

    pub async fn stop_app_server(name: String) {
        let mut config = CONFIGURATION.write().await;
        let srv: Option<app_servers::AppCluster> = config.servers.remove(&name);
        srv.map(|server| server.stop());
    }

    pub async fn init() {
        lazy_static::initialize(&CONFIGURATION);
        let mut config = CONFIGURATION.write().await;
        let panel = "control-panel".to_string();
        let srv: app_servers::AppCluster = app_servers::start(panel.clone())
            .await
            .expect("Cannot init control panel.");
        config.servers.insert(panel, srv);
    }
}

async fn put_file_watcher(
    _req: HttpRequest,
    _body: actix_web::web::Payload,
) -> actix_web::HttpResponse {
    config::start_file_watcher().await;
    "Started".into()
}

async fn delete_file_watcher(
    _req: HttpRequest,
    _body: actix_web::web::Payload,
) -> actix_web::HttpResponse {
    config::stop_file_watcher().await;
    "Stopped".into()
}

async fn get_apps() -> web::Json<Vec<config::ServerStatus>> {
    let apps = app_servers::get_app_names();
    let app_statuses = config::get_app_statuses(apps).await;
    web::Json(app_statuses)
}

async fn put_app(
    _req: HttpRequest,
    app_name: web::Path<String>,
    _body: actix_web::web::Payload,
) -> actix_web::HttpResponse {
    match config::start_app_server(app_name.clone()).await {
        Ok(is_new) => {
            if is_new {
                format!("Loaded app {}", app_name).into()
            } else {
                format!("App {} was already running", app_name).into()
            }
        }
        Err(e) => {
            eprintln!("Error loading {}:\n{:?}", app_name, e);
            HttpResponse::with_body(
                StatusCode::INTERNAL_SERVER_ERROR,
                "500 Internal Error".into(),
            )
        }
    }
}

async fn delete_app(
    _req: HttpRequest,
    name: String,
    _body: actix_web::web::Payload,
) -> actix_web::HttpResponse {
    config::stop_app_server(name).await;
    "Closed".into()
}

pub async fn init() {
    config::init().await;
}

// Returning a vec means that the whole set of admin resources can be built all at once here, and
// attached to the app where needed.
pub fn resources() -> Vec<Resource> {
    vec![
        web::resource("/api/admin/file_watcher")
            .route(web::put().to(put_file_watcher))
            .route(web::delete().to(delete_file_watcher)),
        web::resource("/api/admin/apps").route(web::get().to(get_apps)),
        web::resource("/api/admin/apps/{app_name}")
            .route(web::put().to(put_app))
            .route(web::delete().to(delete_app)),
    ]
}
