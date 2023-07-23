use std::{
  collections::{btree_map::Entry, BTreeMap},
  path::PathBuf,
};

use indoc::{formatdoc, indoc};
use poem::{
  error::{InternalServerError, NotFoundError},
  handler,
  web::{Data, Html, Path as WebPath, StaticFileRequest},
  IntoResponse, Response, Result,
};

use super::thumbnail;

#[derive(Debug)]
pub enum Type {
  Image,
  Other,
}

impl From<&Extension> for Type {
  fn from(ext: &Extension) -> Self {
    match ext {
      Extension::Extension(ext) => match ext.as_ref() {
        "png" | "jpg" | "jpeg" | "tif" => Self::Image,
        _ => Self::Other,
      },
      _ => Self::Other,
    }
  }
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Extension {
  Directory,
  Extension(String),
  Missing,
}

impl From<&PathBuf> for Extension {
  fn from(path: &PathBuf) -> Self {
    if path.is_dir() {
      Self::Directory
    } else if let Some(ext) = path.extension() {
      Self::Extension(ext.to_string_lossy().to_lowercase())
    } else {
      Self::Missing
    }
  }
}

impl Extension {
  fn plural_name(&self) -> String {
    match self {
      Self::Extension(ext) => format!(".{ext} files"),
      Self::Directory => "Directories".to_string(),
      Self::Missing => "Extensionless".to_string(),
    }
  }
}

fn list_files(dir: &PathBuf, paths: Vec<PathBuf>) -> String {
  let files: Vec<String> = paths
    .into_iter()
    .map(|file| {
      formatdoc! {r#"
          <li>
            <a href="/by-type/{relative}">{base}{tail}</a>
          </li>
        "#,
        relative = file.strip_prefix(dir).unwrap().to_str().unwrap(),
        base = file.file_name().unwrap().to_string_lossy(),
        tail = if file.is_dir() { "/" } else { "" },
      }
    })
    .collect();

  formatdoc! {"
      <ul>
        {files}
      </ul>
    ",
    files = files.join("\n"),
  }
}

fn thumb_files(dir: &PathBuf, paths: Vec<PathBuf>) -> String {
  let files: Vec<String> = paths
    .into_iter()
    .map(|file| {
      formatdoc! {r#"
          <div class="entry">
            <a href="/by-type/{relative}">
              <div class="thumbnail">
                <img src="/thumbnail/{relative}"/>
              </div>
            </a>
            <a href="/by-type/{relative}">{base}{tail}</a>
          </div>
        "#,
        relative = file.strip_prefix(dir).unwrap().to_str().unwrap(),
        base = file.file_name().unwrap().to_string_lossy(),
        tail = if file.is_dir() { "/" } else { "" },
      }
    })
    .collect();

  formatdoc! {r#"
      <div class="thumbnail-list">
        {files}
      </div>
    "#,
    files = files.join("\n"),
  }
}

fn section(dir: &PathBuf, ext: &Extension, mut paths: Vec<PathBuf>) -> String {
  paths.sort_by_key(|a| a.to_string_lossy().to_lowercase());

  let files = if thumbnail::supported(ext) {
    thumb_files(dir, paths)
  } else {
    list_files(dir, paths)
  };

  formatdoc! {"
      <h2>
        {ext}
      </h2>
      {files}
      <hr>
    ",
    ext = ext.plural_name(),
  }
}

/// Serves a directory or file ordered by type with thumbnails.
#[handler]
pub fn by_type(WebPath(path): WebPath<PathBuf>, Data(dir): Data<&PathBuf>, req: StaticFileRequest) -> Result<Response> {
  let path = dir.join(path);

  if path.is_file() {
    return Ok(req.create_response(path, true)?.into_response());
  }

  if !path.is_dir() {
    return Err(NotFoundError.into());
  }

  let mut paths_by_extension: BTreeMap<Extension, Vec<PathBuf>> = BTreeMap::new();

  for entry in path.read_dir().map_err(InternalServerError)? {
    let child_path = if let Ok(entry) = entry { entry.path() } else { continue };

    let ext = Extension::from(&child_path);

    let paths = match paths_by_extension.entry(ext) {
      Entry::Occupied(o) => o.into_mut(),
      Entry::Vacant(v) => v.insert(Vec::new()),
    };

    paths.push(child_path);
  }

  let sections: Vec<String> = paths_by_extension.into_iter().map(|(ext, paths)| section(dir, &ext, paths)).collect();

  let css = indoc! {"
    .thumbnail-list {
      display: flex;
      flex-wrap: wrap;
      gap: 16px;
    }

    .thumbnail-list .entry {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 4px;
    }

    .thumbnail-list .entry .thumbnail {
      height: 200px;
      width: 200px;

      border: 1px solid #fff4;
    }

    .thumbnail-list .entry .thumbnail img {
      width: 100%;
      height: 100%;
      object-fit: contain;
    }
  "};

  Ok(
    Html(formatdoc! {r#"
        <!DOCTYPE html>
        <html>
          <head>
            <meta http-equiv="Content-Type" content="text/html; charset=utf-8">
            <meta name="color-scheme" content="light dark">
            <style>
              {css}
            </style>
            <title>Directory {path}</title>
          </head>
          <body>
            <h1>Directory {path}</h1>
            <hr>
            {sections}
          </body>
        </html>
      "#,
      path = path.to_string_lossy(),
      sections = sections.join("\n"),
    })
    .into_response(),
  )
}
