use std::process::Output;

use crate::endpoints::Type;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("invalid type: {0:?}")]
  InvalidType(Type),

  #[error("thumbnail: {0:?}")]
  Thumbnail(Output),
}
