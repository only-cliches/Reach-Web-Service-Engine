#[macro_use]
extern crate derive_deref;
#[macro_use]
extern crate lazy_static;
extern crate deno_core;
extern crate pin_project;
extern crate serde;

mod app_compiler;
mod database;
mod deno_runtime;
mod routes;
mod util;

use std::path::PathBuf;
use warp::Filter;

const ROOT_FOLDER: &str = ".";

#[tokio::main]
pub async fn main() {
    #[cfg(windows)]
    colors::enable_ansi(); // For Windows 10

    env_logger::init();

    async {
        println!("SETUP:");
        routes::admin::init().await;
        app_compiler::compile_all();

        let admin_routes = routes::admin::resources();
        let static_files = routes::admin::static_assets();

        let filepath = ["root", "res", "html", "admin.html"]
            .iter()
            .collect::<PathBuf>()
            .canonicalize()
            .expect("Infallible file path");
        let html = warp::path("admin").and(warp::fs::file(filepath));

        let admin_server = warp::serve(admin_routes.or(static_files).or(html));
        println!("Admin server loaded");
        admin_server.run(([127u8, 0, 0, 1], 8083u16)).await;
    }
    .await;
}
