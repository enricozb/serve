use std::{
  collections::{btree_map::Entry, BTreeMap},
  path::PathBuf,
};

use indoc::formatdoc;
use poem::{
  error::{InternalServerError, NotFoundError},
  handler,
  web::{Data, Html, Path as WebPath, StaticFileRequest},
  IntoResponse, Response, Result,
};

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
enum Extension {
  Directory,
  Extension(String),
  Missing,
}

impl From<PathBuf> for Extension {
  fn from(path: PathBuf) -> Self {
    if path.is_dir() {
      Self::Directory
    } else if let Some(extension) = path.extension() {
      Self::Extension(extension.to_string_lossy().to_string())
    } else {
      Self::Missing
    }
  }
}

impl Extension {
  fn plural_name(&self) -> String {
    match self {
      Self::Extension(extension) => format!(".{extension} files"),
      Self::Directory => "Directories".to_string(),
      Self::Missing => "Extensionless".to_string(),
    }
  }
}

fn section<'a>(dir: &PathBuf, extension: Extension, mut paths: Vec<PathBuf>) -> Result<String> {
  // sort by lower case
  paths.sort_by(|a, b| a.to_string_lossy().to_lowercase().cmp(&b.to_string_lossy().to_lowercase()));

  let files: Vec<String> = paths
    .into_iter()
    .map(|file| {
      formatdoc! {"
          <li>
            <a href=\"/by-type/{relative}\">{base}{tail}</a>
          </li>
        ",
        relative = file.strip_prefix(dir).unwrap().to_str().unwrap(),
        base = file.file_name().unwrap().to_string_lossy(),
        tail = if file.is_dir() { "/" } else { "" },
      }
    })
    .collect();

  Ok(formatdoc! {"
      <h2>
        {extension}
      </h2>
      <ul>
        {files}
      </ul>
      <hr>
    ",
    extension = extension.plural_name(),
    files = files.join("\n"),
  })
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
    let child_path = if let Some(entry) = entry.ok() { entry.path() } else { continue };

    let extension = Extension::from(child_path.clone());

    let paths = match paths_by_extension.entry(extension) {
      Entry::Occupied(o) => o.into_mut(),
      Entry::Vacant(v) => v.insert(Vec::new()),
    };

    paths.push(child_path);
  }

  // for each key (extension):
  //  - create a section with a flexbox and the files,
  //  - order by lower case
  //  - add thumbnails

  let sections: Vec<String> = paths_by_extension
    .into_iter()
    .map(|(extension, paths)| section(&dir, extension, paths))
    .collect::<Result<_>>()?;

  Ok(
    Html(formatdoc! {"
        <!DOCTYPE html>
        <html>
          <head>
            <meta http-equiv=\"Content-Type\" content=\"text/html; charset=utf-8\">
            <meta name=\"color-scheme\" content=\"light dark\">
            <title>Directory {path}</title>
          </head>
          <body>
            <h1>Directory {path}</h1>
            <hr>
            {sections}
          </body>
        </html>
      ",
      path = path.to_string_lossy(),
      sections = sections.join("\n"),
    })
    .into_response(),
  )
}
