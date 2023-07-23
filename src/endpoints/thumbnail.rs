use std::path::PathBuf;

use poem::{
  error::{BadRequest, InternalServerError},
  handler,
  web::{Data, Path as WebPath},
  Response, Result,
};
use tokio::process::Command;

use super::by_type::{Extension, Type};
use crate::error::Error;

pub fn supported(ext: &Extension) -> bool {
  matches!(Type::from(ext), Type::Image)
}

async fn thumbnail_image(path: PathBuf) -> Result<Response> {
  let output = Command::new("convert")
    .arg(path)
    .args(["-thumbnail", "x200", "JPG:-"])
    .output()
    .await
    .map_err(InternalServerError)?;

  if !output.status.success() {
    return Err(InternalServerError(Error::Thumbnail(output)));
  }

  Ok(Response::builder().content_type("image/jpeg").body(output.stdout))
}

/// Returns a thumbnail of `path`.
#[handler]
pub async fn thumbnail(WebPath(path): WebPath<PathBuf>, Data(dir): Data<&PathBuf>) -> Result<Response> {
  match Type::from(&Extension::from(&path)) {
    Type::Image => thumbnail_image(dir.join(path)).await,
    t @ Type::Other => Err(BadRequest(Error::InvalidType(t))),
  }
}
