mod res;
mod util;

use res::constants::*;

#[utoipa::path(
    get,
    path = "/test",
    responses((
        status = 200,
        description = "Send an http GET-request and try to parse the response to json. Return success as string."
    ))
)]
async fn test() -> &'static str {
    let response = reqwest::get("https://httpbin.org/ip").await;
    if response.is_err() { return "error sending request"; }
    let json = response.unwrap().json::<std::collections::HashMap<String, String>>().await;
    if json.is_err() { return "error parsing json"; }
    "got json response"
}

#[tokio::main]
async fn main() {
    use axum::routing::get;
    use axum::response::Redirect;
    use utoipa::OpenApi;
    use utoipa_swagger_ui::SwaggerUi;

    #[derive(OpenApi)]
    #[openapi(
        paths(
            test,
        ),
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
        .route("/test", get(test));

    let address = std::net::SocketAddr::new(LOCALHOST, PORT);
    println!("starting server on http://{address} ...");
    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}