#[macro_use]
extern crate derive_deref;
#[macro_use]
extern crate lazy_static;
extern crate deno_core;
extern crate pin_project;
extern crate serde;

mod app_compiler;
mod deno_runtime;
mod routes;

// use actix_files::Files;
use actix_web::{web, HttpRequest, HttpResponse};

// use log::{ Level, Metadata, Record };

pub async fn root_handler(
    req: HttpRequest,
    _body: actix_web::web::Payload,
) -> actix_web::HttpResponse {
    let script_output = deno_runtime::run_string_script(
        "
    var ops = Deno.core.ops();
    var input_str = utf8ArrayToString(Deno.core.dispatch(ops['get_string']));
    var output_str = `<html>
    <head>
      <title>This page is for ${input_str}</title>
    </head>
    <body>
      Hello, ${input_str}
    </body>
</html>`
    Deno.core.dispatch(ops['return_string'], to_uint8array(output_str));",
        req.path().to_owned(),
    );

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(script_output)
}

#[tokio::main]
pub async fn main() {
    #[cfg(windows)]
    colors::enable_ansi(); // For Windows 10

    env_logger::init();

    // let args: Vec<String> = env::args().collect();
    // let mut flags = deno_cli::flags::flags_from_vec(args);

    // let mut runtime = Builder::new()
    //     .threaded_scheduler()
    //     .enable_all()
    //     .build()
    //     .unwrap();

    let local = tokio::task::LocalSet::new();
    let _system_fut = actix_rt::System::run_in_tokio("main", &local);
    async {
        println!("SETUP:");
        app_compiler::compile_all();
        routes::admin::init().await;
        // let control_panel = app_servers::start_app("control-panel").await;

        // Do not do application-wide initialization beyond this point.
        let server: actix_web::dev::Server = actix_web::HttpServer::new(|| {
            println!("THREAD STARTED");
            let mut app = actix_web::App::new();
            for res in routes::admin::resources() {
                app = app.service(res);
            }
            app.service(web::resource("*").to(root_handler))
        })
        .bind("127.0.0.1:8083")
        .unwrap()
        .run();
        server.await.unwrap();
    }
    .await;
}
