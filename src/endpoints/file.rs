use std::{path::Path, process::Stdio};

use poem::{error::InternalServerError, web::StaticFileRequest, Body, IntoResponse, Response, Result};
use tokio::process::Command;

use super::by_type::{Extension, Type};
use crate::error::Error;

pub fn convert_image<P: AsRef<Path>>(path: P) -> Result<Response> {
  let child = Command::new("convert")
    .arg(path.as_ref())
    .args(["JPG:-"])
    .stdout(Stdio::piped())
    .spawn()
    .map_err(InternalServerError)?;

  let Some(stdout) = child.stdout else {
    return Err(InternalServerError(Error::NoStdout));
  };

  Ok(Response::builder().content_type("image/jpeg").body(Body::from_async_read(stdout)))
}

/// Returns the content of a single file, converting it if necessary.
pub fn file<P: AsRef<Path>>(path: P, req: StaticFileRequest) -> Result<Response> {
  let path = path.as_ref();

  match Type::from(&Extension::from(path)) {
    Type::Image => convert_image(path),
    Type::Video | Type::Other => Ok(req.create_response(path, true)?.into_response()),
  }
}
