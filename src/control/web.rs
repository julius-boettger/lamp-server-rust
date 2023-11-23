use axum::Json;
use std::sync::Arc;
use serde::Deserialize;
use axum::extract::{self, State};
use utoipa::{IntoParams, ToSchema};
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
async fn get_clear_govee_queue(
    State(mut function_queue): State<fn_queue::Queue>
) -> &'static str {
    let message = "queued clearing Govee API call queue";
    println!("{}", message);
    fn_queue::enqueue(&mut function_queue, Arc::new(|govee_queue| {
        println!("{} elements in govee queue, clearing...", govee_queue.len());
        govee_queue.clear();
        println!("queueing setting default brightness and turning off...");
        govee_queue.push_back(SetState::Brightness(constants::govee::default_brightness::DAY));
        govee_queue.push_back(SetState::Power(false));
    }));
    message
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
struct PowerState { value: bool }
#[utoipa::path(
    put,
    path = "/power",
    params(PowerState),
    responses((
        status = 200,
        description = "Queue requested power state. Return response message."
    ))
)]
async fn put_power(
    State(mut function_queue): State<fn_queue::Queue>,
    extract::Json(powerstate): extract::Json<PowerState>
) -> &'static str {
    let setstate = SetState::Power(powerstate.value);
    fn_queue::enqueue(&mut function_queue, Arc::new(move |govee_queue| {
        govee_queue.push_back(setstate);
    }));
    println!("queued {:?}", setstate);
    "queued requested state"
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
struct BrightnessState {
    #[ param(minimum = 1, maximum = 100)]
    #[schema(minimum = 1, maximum = 100)]
    value: u8
}
#[utoipa::path(
    put,
    path = "/brightness",
    params(BrightnessState),
    responses((
        status = 200,
        description = "Queue requested brightness state. Return response message."
    ))
)]
async fn put_brightness(
    State(mut function_queue): State<fn_queue::Queue>,
    extract::Json(brightnessstate): extract::Json<BrightnessState>
) -> &'static str {
    if brightnessstate.value < 1 || brightnessstate.value > 100 {
        // TODO return different status code
        return "wrong range";
    }
    let setstate = SetState::Brightness(brightnessstate.value);
    fn_queue::enqueue(&mut function_queue, Arc::new(move |govee_queue| {
        govee_queue.push_back(setstate);
    }));
    println!("queued {:?}", setstate);
    "queued requested state"
}

/// start webserver. never terminates.
pub async fn start_server(function_queue: fn_queue::Queue, simple_timers: SimpleTimers) {
    use crate::res::constants::net::*;
    use control::timer::Timers;
    use std::sync::Mutex;
    use axum::routing::{get, put};
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
            put_power,
            put_brightness,
        ),
        components(schemas(
            govee::GetState,
            PowerState,
            BrightnessState,
        ))
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
        .route("/clear_govee_queue", get(get_clear_govee_queue))
            .with_state(Arc::clone(&function_queue))
        .route("/power", put(put_power))
            .with_state(Arc::clone(&function_queue))
        .route("/brightness", put(put_brightness))
            .with_state(Arc::clone(&function_queue))
    ;

    // start server
    let address = std::net::SocketAddr::new(LOCALHOST, PORT);
    println!("WEB: starting server on http://{address} ...");
    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}