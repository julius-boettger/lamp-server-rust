use axum::Json;
use crate::util::govee;
use crate::res::constants;
use crate::control::{
    self,
    fn_queue,
    govee::SetState,
    timer::SimpleTimers
};

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

// TODO better return type?
#[utoipa::path(
    get,
    path = "/clear_govee_queue",
    responses((
        status = 200,
        description = "Clear queue of Govee API calls to make. Return response message."
    ))
)]
async fn get_clear_govee_queue(mut function_queue: fn_queue::Queue) -> &'static str {
    let message = "queued clearing Govee API call queue";
    println!("{}", message);
    fn_queue::enqueue(&mut function_queue, &|govee_queue| {
        println!("{} elements in govee queue, clearing...", govee_queue.len());
        govee_queue.clear();
        println!("queueing setting default brightness and turning off...");
        govee_queue.push_back(SetState::Brightness(constants::govee::default_brightness::DAY));
        govee_queue.push_back(SetState::Power(false));
    });
    message
}

/// start webserver. never terminates.
pub async fn start_server(function_queue: fn_queue::Queue, simple_timers: SimpleTimers) {
    use crate::res::constants::net::*;
    use control::timer::Timers;
    use std::sync::{Arc, Mutex};
    use axum::routing::get;
    use axum::response::Redirect;
    use utoipa::OpenApi;
    use utoipa_swagger_ui::SwaggerUi;

    // set up utoipa swagger ui
    #[derive(OpenApi)]
    #[openapi(
        paths(
            // functions with utoipa::path attributes
            get_state,
            get_clear_govee_queue,
        ),
        components(schemas(govee::GetState))
    )]
    struct ApiDoc;

    // higher level timers which will be converted and pushed to `simple_timers`
    let mut timers: Timers = Arc::new(Mutex::new(vec![]));

    // configure routes
    let app = axum::Router::new()

        // swagger ui
        .merge(SwaggerUi::new("/swagger-ui")
            .url("/openapi.json", ApiDoc::openapi()))

        // temporarily redirect root to swagger ui
        .route("/", get(|| async { Redirect::temporary("/swagger-ui") }))

        // actual api
        .route("/state", get(get_state))
        .route("/clear_govee_queue", get(|| async {
            get_clear_govee_queue(function_queue).await
        }))
    ;

    // start server
    let address = std::net::SocketAddr::new(LOCALHOST, PORT);
    println!("WEB: starting server on http://{address} ...");
    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}