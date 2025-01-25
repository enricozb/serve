use std::path::PathBuf;

use indoc::{formatdoc, indoc};
use poem::{
  error::{InternalServerError, NotFoundError},
  handler,
  web::{Data, Html, Path, StaticFileRequest},
  IntoResponse, Response, Result,
};

/// Serves a directory or file ordered by name.
#[handler]
pub fn get(Path(path): Path<PathBuf>, Data(dir): Data<&PathBuf>, req: StaticFileRequest) -> Result<Response> {
  let file = dir.join(path);

  if file.is_file() {
    super::file(file, req)
  } else if file.is_dir() {
    let mut files: Vec<PathBuf> = file
      .read_dir()
      .map_err(InternalServerError)?
      .filter_map(|entry| entry.ok().map(|entry| entry.path()))
      .collect();

    files.sort_by_key(|a| a.to_string_lossy().to_lowercase());

    let files: Vec<String> = files
      .into_iter()
      .map(|file| {
        formatdoc! {"
          <li>
            <a href=\"/get/{relative}\">{base}{tail}</a>
          </li>
        ",
          relative = file.strip_prefix(dir).unwrap().to_str().unwrap(),
          base = file.file_name().unwrap().to_string_lossy(),
          tail = if file.is_dir() { "/" } else { "" },
        }
      })
      .collect();

    let js = indoc! {r#"
      document.addEventListener("keydown", (event) => {
        if (event.key === "t") {
          const path = window.location.pathname;
          window.location.pathname = path.replace("/get/", "/by-type/");
        }
      });
    "#};

    Ok(
      Html(formatdoc! {"
        <!DOCTYPE html>
        <html>
          <head>
            <meta http-equiv=\"Content-Type\" content=\"text/html; charset=utf-8\">
            <meta name=\"color-scheme\" content=\"light dark\">
            <script>
              {js}
            </script>
            <title>Directory {file}</title>
          </head>
          <body>
            <h1>Directory {file}</h1>
            <hr>
            <ul>
              {files}
            </ul>
            <hr>
          </body>
        </html>
      ",
        file = file.to_string_lossy(),
        files = files.join("\n"),
      })
      .into_response(),
    )
  } else {
    Err(NotFoundError.into())
  }
}
