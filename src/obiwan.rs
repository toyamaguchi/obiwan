use axum::extract::FromRef;

const STATIC_DIR: include_dir::Dir<'static> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/rsc/static");

#[derive(Clone, FromRef)]
struct AppState {
    hb: handlebars::Handlebars<'static>,
}

async fn index(hb: axum::extract::State<handlebars::Handlebars<'_>>) -> axum::response::Response {
    let content = hb.render("index", &{}).expect("failed to render");
    axum::response::Response::builder()
        .status(axum::http::StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(axum::body::Body::from(content))
        .unwrap()
}

pub async fn start() {
    let mut hb: handlebars::Handlebars<'static> = handlebars::Handlebars::new();
    hb.register_template_string("index", include_str!("../rsc/template/index.html"))
        .expect("failed to register template string: ../rsc/template/index.html");

    let app_state = AppState { hb: hb };

    let static_dir_service = tower_serve_static::ServeDir::new(&STATIC_DIR);

    let app = axum::Router::new()
        .route("/", axum::routing::get(index))
        .nest_service("/static", static_dir_service)
        .with_state(app_state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
