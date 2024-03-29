mod endpoints;
mod error;

use std::{
  io::{Error as IoError, ErrorKind as IoErrorKind},
  path::PathBuf,
};

use clap::Parser;
use poem::{
  get, handler,
  listener::TcpListener,
  web::{Data, Redirect, StaticFileRequest},
  EndpointExt, Response, Result, Route, Server,
};

/// Serve static files over HTTP.
#[derive(Clone, Parser)]
#[command(author, version, about)]
struct Args {
  /// The file or directory to serve.
  #[arg(default_value = ".")]
  path: PathBuf,

  /// The port to serve on.
  #[arg(long, default_value_t = 8000)]
  port: u16,

  /// The host to serve on.
  #[arg(long, default_value_t = String::from("0.0.0.0"))]
  host: String,
}

/// Serves a single file.
#[handler]
fn serve_file(Data(file): Data<&PathBuf>, req: StaticFileRequest) -> Result<Response> {
  endpoints::file(file, req)
}

#[handler]
fn index() -> Redirect {
  Redirect::see_other("/get/")
}

#[tokio::main]
async fn main() -> Result<(), IoError> {
  let args = Args::parse();
  let path = args.path.clone();

  let app = if path.is_file() {
    Route::new().at("/get/", get(serve_file))
  } else if path.is_dir() {
    Route::new()
      .at("/get/*path", get(endpoints::get))
      .at("/by-type/*path", get(endpoints::by_type))
      .at("/thumbnail/*path", get(endpoints::thumbnail))
  } else {
    return Err(std::io::Error::new(IoErrorKind::NotFound, format!("{path:?} not found")));
  };

  let app = app.at("/", get(index)).data(path);

  println!("serving {:?} at {}:{}...", args.path, args.host, args.port);

  Server::new(TcpListener::bind((args.host, args.port))).run(app).await
}
