use std::{path::PathBuf, process::Stdio};

use poem::{
  error::{BadRequest, InternalServerError},
  handler,
  web::{Data, Path as WebPath},
  Body, Response, Result,
};
use tokio::process::Command;

use super::by_type::{Extension, Type};
use crate::error::Error;

pub fn supported(ext: &Extension) -> bool {
  matches!(Type::from(ext), Type::Image | Type::Video)
}

fn thumbnail_image(path: PathBuf) -> Result<Response> {
  let child = Command::new("convert")
    .arg(path)
    .args(["-auto-orient", "-thumbnail", "x200", "JPG:-"])
    .stdout(Stdio::piped())
    .spawn()
    .map_err(InternalServerError)?;

  let Some(stdout) = child.stdout else {
    return Err(InternalServerError(Error::NoStdout));
  };

  Ok(Response::builder().content_type("image/jpeg").body(Body::from_async_read(stdout)))
}

fn thumbnail_video(path: PathBuf) -> Result<Response> {
  let child = Command::new("ffmpeg")
    .arg("-i")
    .arg(path)
    .args([
      "-ss",
      "00:00:01.00",
      "-vf",
      "scale=320:320:force_original_aspect_ratio=decrease",
      "-vframes",
      "1",
      "-f",
      "image2pipe",
      "-",
    ])
    .stdout(Stdio::piped())
    .spawn()
    .map_err(InternalServerError)?;

  let Some(stdout) = child.stdout else {
    return Err(InternalServerError(Error::NoStdout));
  };

  Ok(Response::builder().content_type("image/jpeg").body(Body::from_async_read(stdout)))
}

/// Returns a thumbnail of `path`.
#[handler]
pub fn thumbnail(WebPath(path): WebPath<PathBuf>, Data(dir): Data<&PathBuf>) -> Result<Response> {
  match Type::from(&Extension::from(&path)) {
    Type::Image => thumbnail_image(dir.join(path)),
    Type::Video => thumbnail_video(dir.join(path)),
    t @ Type::Other => Err(BadRequest(Error::InvalidType(t))),
  }
}
