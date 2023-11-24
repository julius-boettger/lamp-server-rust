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

// TODO return json instead of plain strings...?
// TODO document different status codes for wrong json, ...

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
            rgb_color: (255, 255, 255),
            brightness: 100,
            power: false
        })
    }
}

#[utoipa::path(
    get,
    path = "/clear_govee_queue",
    responses((
        status = 200,
        description = "Clear queue of Govee API calls to make. Then set the brightness to a default value and turn the lamp off. Return response message."
    ))
)]
async fn get_clear_govee_queue(
    State(mut function_queue): State<fn_queue::Queue>
) -> &'static str {
    let message = "queued clearing Govee API call queue, setting brightness and turning off";
    println!("{}", message);
    fn_queue::enqueue(&mut function_queue, Arc::new(|govee_queue| {
        println!("{} elements in govee queue, clearing...", govee_queue.len());
        govee_queue.clear();
        println!("queueing setting default brightness and turning off...");
        govee_queue.push_back(SetState::Brightness(constants::govee::default_brightness::DAY));
        govee_queue.push_back(SetState::Power(false));
    })).await;
    message
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
struct PowerState { power: bool }
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
    let setstate = SetState::Power(powerstate.power);
    fn_queue::enqueue(&mut function_queue, Arc::new(move |govee_queue| {
        govee_queue.push_back(setstate);
    })).await;
    println!("queued {:?}", setstate);
    "queued requested state"
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
struct BrightnessState {
    #[ param(minimum = 1, maximum = 100)]
    #[schema(minimum = 1, maximum = 100)]
    brightness: u8
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
    if brightnessstate.brightness < 1 || brightnessstate.brightness > 100 {
        // TODO return different status code
        return "wrong range";
    }

    let setstate = SetState::Brightness(brightnessstate.brightness);
    fn_queue::enqueue(&mut function_queue, Arc::new(move |govee_queue| {
        govee_queue.push_back(setstate);
    })).await;

    println!("queued {:?}", setstate);
    "queued requested state"
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
struct ColorState {
    #[ param(minimum = 0, maximum = 255)]
    #[schema(minimum = 0, maximum = 255)]
    r: u8,
    #[ param(minimum = 0, maximum = 255)]
    #[schema(minimum = 0, maximum = 255)]
    g: u8,
    #[ param(minimum = 0, maximum = 255)]
    #[schema(minimum = 0, maximum = 255)]
    b: u8
}
#[utoipa::path(
    put,
    path = "/color",
    params(ColorState),
    responses((
        status = 200,
        description = "Queue requested color state. Return response message."
    ))
)]
async fn put_color(
    State(mut function_queue): State<fn_queue::Queue>,
    extract::Json(colorstate): extract::Json<ColorState>
) -> &'static str {
    let setstate = SetState::Color((colorstate.r, colorstate.g, colorstate.b));
    fn_queue::enqueue(&mut function_queue, Arc::new(move |govee_queue| {
        govee_queue.push_back(setstate);
    })).await;
    println!("queued {:?}", setstate);
    "queued requested state"
}

/// start webserver. never terminates.
pub async fn start_server(function_queue: fn_queue::Queue, simple_timers: SimpleTimers) {
    use crate::res::constants::net::*;
    use control::timer::Timers;
    use tokio::sync::Mutex;
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
            put_color,
        ),
        components(schemas(
            govee::GetState,
            PowerState,
            BrightnessState,
            ColorState,
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
        .route("/color", put(put_color))
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