use std::{
  io::{Error as IoError, ErrorKind as IoErrorKind},
  path::PathBuf,
};

use clap::Parser;
use indoc::formatdoc;
use poem::{
  error::{InternalServerError, NotFoundError},
  get, handler,
  listener::TcpListener,
  web::{Data, Html, Path, StaticFileRequest},
  EndpointExt, IntoResponse, Response, Result, Route, Server,
};

/// Serve static files over HTTP.
#[derive(Clone, Parser)]
#[command(author, version, about)]
struct Args {
  /// The file or directory to serve.
  path: PathBuf,

  /// The port to serve on.
  #[arg(long, default_value_t = 8000)]
  port: u16,

  /// The host to serve on.
  #[arg(long, default_value_t = String::from("0.0.0.0"))]
  host: String,
}

/// Serves a directory or file.
#[handler]
fn serve(Path(path): Path<PathBuf>, Data(dir): Data<&PathBuf>, req: StaticFileRequest) -> Result<Response> {
  let file = dir.join(path);

  if file.is_file() {
    Ok(req.create_response(file, true)?.into_response())
  } else if file.is_dir() {
    let files: Vec<String> = file
      .read_dir()
      .map_err(InternalServerError)?
      .into_iter()
      .filter_map(|entry| entry.ok().map(|entry| entry.file_name()))
      .map(|file| {
        formatdoc! {"
          <li>
            <a href=\"{file}\">{file}</a>
          </li>
        ",
          file = file.to_string_lossy()
        }
      })
      .collect();

    Ok(
      Html(formatdoc! {"
          <!DOCTYPE html>
          <html>
          <head>
            <meta http-equiv=\"Content-Type\" content=\"text/html; charset=utf-8\">
            <title>Directory {dir}</title>
          </head>
          <body>
            <h1>Directory {dir}</h1>
            <hr>
            <ul>
              {files}
            </ul>
            <hr>
          </body>
        </html>
      ",
        dir = dir.to_string_lossy(),
        files = files.join("\n"),
      })
      .into_response(),
    )
  } else {
    Err(NotFoundError.into())
  }
}

/// Serves a single file.
#[handler]
fn serve_file(Data(file): Data<&PathBuf>, req: StaticFileRequest) -> Result<impl IntoResponse> {
  Ok(req.create_response(file, true)?)
}

#[tokio::main]
async fn main() -> Result<(), IoError> {
  let args = Args::parse();
  let path = args.path.clone();

  let app = if path.is_file() {
    Route::new().at("/", get(serve_file)).data(path)
  } else if path.is_dir() {
    Route::new().at("/*path", get(serve)).data(path)
  } else {
    return Err(std::io::Error::new(IoErrorKind::NotFound, format!("{:?} not found", path)));
  };

  println!("serving {:?} at {}:{}...", args.path, args.host, args.port);

  Server::new(TcpListener::bind((args.host, args.port))).run(app).await
}
