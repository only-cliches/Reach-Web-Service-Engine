use actix_web::{web, HttpRequest, Resource};

// This is a private, hidden module containing a static admin configuration object. This is built
// this way to prevent using these functions in any route except /api/admin, also defined below.
mod config {
    use crate::app_compiler::file_watcher;
    use std::sync::Mutex;

    const MEM_CORRUPT_ERR: &str = "Configuration memory corrupted.";

    lazy_static! {
        static ref CONFIGURATION: Mutex<AdminState> = Mutex::new(AdminState::new());
    }

    pub struct AdminState {
        compile_watcher: file_watcher::MaybeWatcher,
    }

    impl AdminState {
        pub fn new() -> AdminState {
            AdminState {
                compile_watcher: file_watcher::start(None),
            }
        }
    }

    pub fn start_file_watcher() {
        let mut config = CONFIGURATION.lock().expect(MEM_CORRUPT_ERR);
        let watcher = config.compile_watcher.take();
        config.compile_watcher = file_watcher::start(watcher);
    }

    pub fn stop_file_watcher() {
        let mut config = CONFIGURATION.lock().expect(MEM_CORRUPT_ERR);
        let watcher = config.compile_watcher.take();
        config.compile_watcher = file_watcher::stop(watcher);
    }

    pub fn init() {
        lazy_static::initialize(&CONFIGURATION);
    }
}

async fn put_file_watcher(
    _req: HttpRequest,
    _body: actix_web::web::Payload,
) -> actix_web::HttpResponse {
    config::start_file_watcher();
    "Started".into()
}

async fn delete_file_watcher(
    _req: HttpRequest,
    _body: actix_web::web::Payload,
) -> actix_web::HttpResponse {
    config::stop_file_watcher();
    "Stopped".into()
}

async fn put_domain(_req: HttpRequest, _body: actix_web::web::Payload) -> actix_web::HttpResponse {
    "Unimplemented".into()
}

async fn delete_domain(
    _req: HttpRequest,
    _body: actix_web::web::Payload,
) -> actix_web::HttpResponse {
    "Unimplemented".into()
}

// Returning a vec means that the whole set of admin resources can be built all at once here, and
// attached to the app where needed.
pub fn resources() -> Vec<Resource> {
    config::init();
    vec![
        web::resource("/api/admin/file_watcher")
            .route(web::put().to(put_file_watcher))
            .route(web::delete().to(delete_file_watcher)),
        web::resource("/api/admin/domains")
            .route(web::put().to(put_domain))
            .route(web::delete().to(delete_domain)),
    ]
}
