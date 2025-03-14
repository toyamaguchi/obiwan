use axum::extract::FromRef;
use axum::response::IntoResponse;
use serde_derive::{Deserialize, Serialize};

const STATIC_DIR: include_dir::Dir<'static> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/rsc/static");

#[derive(Clone, FromRef)]
struct AppState {
    hb: handlebars::Handlebars<'static>,
}

#[derive(Deserialize)]
struct PathParameter {
    path: String,
}

#[derive(Serialize)]
struct DirectoryEntry {
    is_file: bool,
    is_directory: bool,
    is_symlink: bool,
    file_name: String,
    path: String,
}

#[derive(Serialize)]
struct DirectoryEntries {
    items: Vec<DirectoryEntry>,
}

async fn index(hb: axum::extract::State<handlebars::Handlebars<'_>>) -> axum::response::Response {
    let content = hb.render("index", &{}).expect("failed to render");
    axum::response::Response::builder()
        .status(axum::http::StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(axum::body::Body::from(content))
        .unwrap()
}

async fn api_v1_file(
    path_parameter: axum::extract::Query<PathParameter>,
    range: Option<axum_extra::TypedHeader<axum_extra::headers::Range>>,
) -> axum::response::Response {
    let path_parameter = path_parameter.0;
    let path_buf = std::path::PathBuf::from(path_parameter.path);

    // check file existance
    let file = tokio::fs::File::open(&path_buf).await.unwrap();
    if file.metadata().await.is_err() {
        return axum::response::Response::builder()
            .status(axum::http::StatusCode::NOT_FOUND)
            .body(axum::body::Body::empty())
            .unwrap();
    }

    // create response body
    let body = axum_range::KnownSize::file(file).await.unwrap();
    let range = range.map(|axum_extra::TypedHeader(range)| range);
    let mut response = axum_range::Ranged::new(range, body).into_response();

    // set Content-Type
    let mime = match path_buf.extension() {
        Some(extension) => match extension.to_str().unwrap() {
            ".avi" => Some("video/video/x-msvideo"),
            ".mp4" => Some("video/mp4"),
            _ => None,
        },
        None => None,
    };
    if let Some(mime) = mime {
        let header_map = response.headers_mut();
        header_map.append("Content-Type", mime.parse().unwrap());
    }

    response
}

async fn api_v1_directory(
    path_parameter: axum::extract::Query<PathParameter>,
) -> axum::response::Response {
    let path_parameter = path_parameter.0;
    let path_buf = std::path::PathBuf::from(path_parameter.path);

    let mut directory_entries = DirectoryEntries { items: vec![] };

    // check file existance
    let mut read_dir = tokio::fs::read_dir(&path_buf).await.unwrap();
    loop {
        match read_dir.next_entry().await.unwrap() {
            Some(dir_entry) => {
                let file_type = dir_entry.file_type().await.unwrap();
                let directory_entry = DirectoryEntry {
                    is_file: file_type.is_file(),
                    is_directory: file_type.is_dir(),
                    is_symlink: file_type.is_symlink(),
                    file_name: dir_entry.file_name().into_string().unwrap(),
                    path: dir_entry.path().to_string_lossy().into_owned(),
                };
                directory_entries.items.push(directory_entry);
            }
            None => {
                break;
            }
        }
    }

    axum::Json(directory_entries).into_response()
}

pub async fn start() {
    let mut hb: handlebars::Handlebars<'static> = handlebars::Handlebars::new();
    hb.register_template_string("index", include_str!("../rsc/template/index.html"))
        .expect("failed to register template string: ../rsc/template/index.html");

    let app_state = AppState { hb };

    let static_dir_service = tower_serve_static::ServeDir::new(&STATIC_DIR);

    let app = axum::Router::new()
        .route("/", axum::routing::get(index))
        .route(r"/api/v1/file", axum::routing::get(api_v1_file))
        .route(r"/api/v1/directory", axum::routing::get(api_v1_directory))
        .nest_service("/static", static_dir_service)
        .with_state(app_state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
