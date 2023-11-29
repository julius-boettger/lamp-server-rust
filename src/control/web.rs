use std::sync::Arc;
use serde::Deserialize;
use itertools::Itertools;
use utoipa::{IntoParams, ToSchema};
use crate::constants;
use crate::control::{state, timer::*};
use crate::util::{fn_queue, govee_api::{self, SetState}};
use axum::{
    Json,
    TypedHeader,
    http::StatusCode as Code,
    extract::{self, State},
    headers::{Authorization, authorization::Basic}
};

type Response<T> = Result<T, (Code, &'static str)>;

// TODO replace with middleware?
/// require basic authorization with password equal to sha256 of govee api key (case insensitive)
fn authorize(auth: Authorization<Basic>) -> Response<()> {
    use constants::govee_secrets::API_KEY;
    if !auth.0.password().eq_ignore_ascii_case(sha256::digest(API_KEY).as_str()) {
        return Err((Code::UNAUTHORIZED, "password in basic authorization header is incorrect. expected sha256 of Govee API key (case insensitive)."));
    }
    Ok(())
}

#[utoipa::path(
    get,
    path = "/state",
    responses((
        status = 200,
        description = "Get current state of lamp. Returns a default value on error.",
        body = GetState
    )),
    security(("authorization" = [])) // require auth
)]
async fn get_state(
    TypedHeader(auth): TypedHeader<Authorization<Basic>>
) -> Response<Json<govee_api::GetState>> {
    authorize(auth)?;
    match govee_api::get_state().await {
        Ok(state) => Ok(Json(state)),
        _ => Err((Code::INTERNAL_SERVER_ERROR,
                  "could not get state. likely because of Govee API rate limit."))
    }
}

#[utoipa::path(
    get,
    path = "/clear_govee_queue",
    responses((
        status = 200,
        description = "Clear queue of Govee API calls to make. Then set the brightness to a default value and turn the lamp off. Return response message."
    )),
    security(("authorization" = [])) // require auth
)]
async fn get_clear_govee_queue(
    TypedHeader(auth): TypedHeader<Authorization<Basic>>,
    State(mut function_queue): State<fn_queue::Queue>
) -> Response<&'static str> {
    authorize(auth)?;
    let message = "queued clearing Govee API call queue, setting brightness and turning off";
    println!("{}", message);
    fn_queue::enqueue(&mut function_queue, Arc::new(|govee_queue| {
        println!("{} elements in govee queue, clearing...", govee_queue.len());
        govee_queue.clear();
        println!("queueing setting default brightness and turning off...");
        govee_queue.push_back(SetState::Brightness(constants::brightness::DAY));
        govee_queue.push_back(SetState::Power(false));
    })).await;
    Ok(message)
}

#[utoipa::path(
    get,
    path = "/activate_nightlamp",
    responses((
        status = 200,
        description = "Set brightness to default for night and color to nice warm white. Return response message."
    )),
    security(("authorization" = [])) // require auth
)]
async fn get_activate_nightlamp(
    TypedHeader(auth): TypedHeader<Authorization<Basic>>,
    State(mut function_queue): State<fn_queue::Queue>
) -> Response<&'static str> {
    authorize(auth)?;
    let message = "queued nightlamp activation";
    println!("{}", message);
    fn_queue::enqueue(&mut function_queue, Arc::new(|govee_queue|
        state::nightlamp(govee_queue))).await;
    Ok(message)
}

#[utoipa::path(
    get,
    path = "/timers",
    responses((
        status = 200,
        description = "Get array of current timers.",
        body = Vec<Timer>
    )),
    security(("authorization" = [])) // require auth
)]
async fn get_timers(
    TypedHeader(auth): TypedHeader<Authorization<Basic>>,
    State(timers): State<Timers>
) -> Response<Json<Vec<Timer>>> {
    authorize(auth)?;
    Ok(Json(timers.lock().await.clone()))
}

#[utoipa::path(
    put,
    path = "/timers",
    // TimerAction (enum) cant implement IntoParams, so this doesnt work
    //params(Vec<Timer>), 
    responses((
        status = 200,
        description = "Set timers to provided array of timers. Duplicates will be removed. Return response message.",
    )),
    security(("authorization" = [])) // require auth
)]
async fn put_timers(
    TypedHeader(auth): TypedHeader<Authorization<Basic>>,
    State(state): State<(Timers, SimpleTimers)>,
    extract::Json(new_timers): extract::Json<Vec<Timer>>
) -> Response<&'static str> {
    authorize(auth)?;

    /// return given error message with status code UNPROCESSABLE_ENTITY if condition
    fn error_if(condition: bool, message: &'static str) -> Response<()> {
        match condition {
            true  => Err((Code::UNPROCESSABLE_ENTITY, message)),
            false => Ok(())
        }
    }

    // remove duplicates
    let new_timers = new_timers.into_iter().unique().collect_vec();

    // validate new timers
    for timer in new_timers.iter() {
        error_if(*timer.timeday.get_hour() > 23, "timeday.hour must be <= 23")?;
        error_if(*timer.timeday.get_minute() > 59, "timeday.minute must be <= 59")?;
        error_if(timer.timeday.get_days().is_empty(), "timeday.days must not be empty")?;
        error_if(timer.timeday.get_days().len() > 7, "timeday.days must have <= 7 elements")?;
        error_if(timer.timeday.get_days().iter().any(|&d| d > 6), "every day in timeday.days has to be <= 6")?;
        match timer.action {
            TimerAction::Sunrise { duration_min, stay_on_for_min, sleep_min, nightlamp_min } => {
                error_if(duration_min < 1, "action.params.duration_min has to be >= 1")?;
                error_if(nightlamp_min < 1, "action.params.nightlamp_min has to be >= 1")?;
                error_if(sleep_min < duration_min, "action.params.sleep_min has to be >= action.params.duration_min")?;
                // limit for all: i16::MAX = 32767
                error_if(sleep_min > 32767, "action.params.sleep_min has to be <= 32767")?;
                error_if(duration_min > 32767, "action.params.duration_min has to be <= 32767")?;
                error_if(nightlamp_min > 32767, "action.params.nightlamp_min has to be <= 32767")?;
                error_if(stay_on_for_min > 32767, "action.params.stay_on_for_min has to be <= 32767")?;
            }
        }
    }

    let (timers, simple_timers) = state;
    *timers.lock().await = new_timers;
    process_timers(&timers, &simple_timers).await;
    Ok("timers updated.")
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
    )),
    security(("authorization" = [])) // require auth
)]
async fn put_power(
    TypedHeader(auth): TypedHeader<Authorization<Basic>>,
    State(mut function_queue): State<fn_queue::Queue>,
    extract::Json(powerstate): extract::Json<PowerState>
) -> Response<&'static str> {
    authorize(auth)?;
    let setstate = SetState::Power(powerstate.power);
    fn_queue::enqueue(&mut function_queue, Arc::new(move |govee_queue| {
        govee_queue.push_back(setstate);
    })).await;
    println!("queued {:?}", setstate);
    Ok("queued requested state")
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
    )),
    security(("authorization" = [])) // require auth
)]
async fn put_brightness(
    TypedHeader(auth): TypedHeader<Authorization<Basic>>,
    State(mut function_queue): State<fn_queue::Queue>,
    extract::Json(brightnessstate): extract::Json<BrightnessState>
) -> Response<&'static str> {
    authorize(auth)?;

    if brightnessstate.brightness < 1 || brightnessstate.brightness > 100 {
        return Err((Code::UNPROCESSABLE_ENTITY, "brightness must be from 1 to 100"));
    }

    let setstate = SetState::Brightness(brightnessstate.brightness);
    fn_queue::enqueue(&mut function_queue, Arc::new(move |govee_queue| {
        govee_queue.push_back(setstate);
    })).await;

    println!("queued {:?}", setstate);
    Ok("queued requested state")
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
    )),
    security(("authorization" = [])) // require auth
)]
async fn put_color(
    TypedHeader(auth): TypedHeader<Authorization<Basic>>,
    State(mut function_queue): State<fn_queue::Queue>,
    extract::Json(colorstate): extract::Json<ColorState>
) -> Response<&'static str> {
    authorize(auth)?;
    let setstate = SetState::Color((colorstate.r, colorstate.g, colorstate.b));
    fn_queue::enqueue(&mut function_queue, Arc::new(move |govee_queue| {
        govee_queue.push_back(setstate);
    })).await;
    println!("queued {:?}", setstate);
    Ok("queued requested state")
}

/// start webserver. never terminates.
pub async fn start_server(function_queue: fn_queue::Queue, simple_timers: SimpleTimers) {
    use constants::net::*;
    use tokio::sync::Mutex;
    use utoipa_swagger_ui::SwaggerUi;
    use axum::{response::Redirect, routing::{get, put}};
    use utoipa::{OpenApi, openapi::security::{SecurityScheme, Http, HttpAuthScheme}};

    /// utility struct for utoipa to register basic http authorization.
    /// this is necessary for showing an "Authorize" button in swagger-ui.
    struct AuthHint;
    impl utoipa::Modify for AuthHint {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            if let Some(components) = openapi.components.as_mut() {
                components.add_security_scheme(
                    "authorization",
                    SecurityScheme::Http(Http::new(HttpAuthScheme::Basic))
                )
            }
        }
    }

    // set up utoipa swagger ui
    #[derive(OpenApi)]
    #[openapi(
        // use security scheme for basic http authorization
        modifiers(&AuthHint),
        paths(
            // functions with #[utoipa::path(...)]
            get_state,
            get_clear_govee_queue,
            put_power,
            put_brightness,
            put_color,
            get_timers,
            put_timers,
            get_activate_nightlamp,
        ),
        components(schemas(
            // enums/structs with #[derive(utoipa::ToSchema)]
            govee_api::GetState,
            PowerState,
            BrightnessState,
            ColorState,
            Timer
        ))
    )]
    struct ApiDoc;

    // higher level timers which will be converted and pushed to `simple_timers`
    let timers: Timers = Arc::new(Mutex::new(vec![]));

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
        .route("/activate_nightlamp", get(get_activate_nightlamp))
            .with_state(Arc::clone(&function_queue))
        .route("/power", put(put_power))
            .with_state(Arc::clone(&function_queue))
        .route("/brightness", put(put_brightness))
            .with_state(Arc::clone(&function_queue))
        .route("/color", put(put_color))
            .with_state(Arc::clone(&function_queue))
        .route("/timers", get(get_timers))
            .with_state(Arc::clone(&timers))
        .route("/timers", put(put_timers))
            .with_state((Arc::clone(&timers), Arc::clone(&simple_timers)))
    ;

    // start server
    let address = std::net::SocketAddr::new(LOCALHOST, PORT);
    println!("WEB: starting server on http://{address} ...");
    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}