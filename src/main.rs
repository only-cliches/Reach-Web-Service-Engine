#[macro_use]
extern crate derive_deref;
extern crate deno_core;
extern crate pin_project;

mod deno_runtime;
mod tsc_processor;

// use actix_files::Files;
use actix_web::{ HttpRequest, HttpResponse };
use tokio::runtime::Builder;

// use log::{ Level, Metadata, Record };


pub async fn root_handler(req: HttpRequest,
  _body: actix_web::web::Payload) -> actix_web::HttpResponse {

    let script_output = deno_runtime::run_string_script("
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
    req.path().to_owned());

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(script_output)

}


pub fn main() {
  #[cfg(windows)]
  colors::enable_ansi(); // For Windows 10

  env_logger::init();

  // let args: Vec<String> = env::args().collect();
  // let mut flags = deno_cli::flags::flags_from_vec(args);

  let mut single_rt = Builder::new()
      .basic_scheduler()
      .enable_all()
      .build()
      .unwrap();

  let local = tokio::task::LocalSet::new();
  let system_fut = actix_rt::System::run_in_tokio("main", &local);
  local.block_on(&mut single_rt, async {
    tokio::task::spawn_local(system_fut);

    println!("SETUP:");
    println!("{}", tsc_processor::run("Compile me.".to_owned()));

    let _ = actix_web::HttpServer::new(|| {

        actix_web::App::new()
            .service(actix_web::web::resource("*").to(root_handler))
            
    })
        .bind("127.0.0.1:8083")
        .unwrap()
        .run()
        .await;
  });
}
