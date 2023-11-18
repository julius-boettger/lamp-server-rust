use axum::Json;
use crate::util::govee;

// TODO return different status code instead of default
#[utoipa::path(
    get,
    path = "/state",
    responses((
        status = 200,
        description = "Get current state of lamp. Returns a default value on error.",
        body = GetState
    ))
)]
async fn get_state() -> Json<govee::GetState> {
    let result = govee::get_state().await;
    if let Ok(state) = result {
        Json(state)
    } else {
        Json(govee::GetState {
            color: (255, 255, 255),
            brightness: 100,
            power: false
        })
    }
}

/// start webserver. never terminates
pub async fn start_server() {
    use crate::res::constants::net::*;
    use axum::routing::get;
    use axum::response::Redirect;
    use utoipa::OpenApi;
    use utoipa_swagger_ui::SwaggerUi;

    #[derive(OpenApi)]
    #[openapi(
        paths(
            // functions with utoipa::path attributes
            get_state
        ),
        components(schemas(govee::GetState)),
        tags((name = "lamp-server-rust", description = "API for interacting with my lamp"))
    )]
    struct ApiDoc;

    let app = axum::Router::new()
        // swagger ui
        .merge(SwaggerUi::new("/swagger-ui")
            .url("/openapi.json", ApiDoc::openapi()))
        // temporarily redirect root to swagger ui
        .route("/", get(|| async { Redirect::temporary("/swagger-ui") }))
        // actual api
        .route("/state", get(get_state));

    let address = std::net::SocketAddr::new(LOCALHOST, PORT);
    println!("starting server on http://{address} ...");
    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}