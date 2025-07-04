use std::sync::Arc;
use serde::Deserialize;
use itertools::Itertools;
use utoipa::{IntoParams, ToSchema};
use crate::constants;
#[allow(clippy::wildcard_imports)]
use crate::control::{state, timer::*};
use crate::util::{fn_queue, govee_api::{self, SetState}};
use axum::{
    Json,
    middleware,
    http::HeaderMap,
    extract::{self, State},
    http::StatusCode as Code
};

type Response<T> = Result<T, (Code, &'static str)>;

/// axum middleware to check authorization before evaluating a request
async fn validate_request(
    headers: HeaderMap,
    request: extract::Request,
    next: middleware::Next,
) -> Result<axum::response::Response, (Code, &'static str)> {
    check_authorization(&headers)?;
    // evaluate and return original request
    Ok(next.run(request).await)
}

fn check_authorization(headers: &HeaderMap) -> Response<()> {
    use crate::util::govee_secrets::api_key;
    let Some(value) = headers.get("authorization") else {
        return Err((Code::BAD_REQUEST, "authorization header is missing"));
    };

    let Ok(value) = value.to_str() else {
        return Err((Code::BAD_REQUEST, "authorization header value contains characters that are not visible ASCII"));
    };

    let Some(token) = value.strip_prefix("Bearer ") else {
        return Err((Code::BAD_REQUEST, "authorization header value is not of type bearer"));
    };

    // check for expected token
    if token.eq_ignore_ascii_case(sha256::digest(api_key()).as_str()) {
        Ok(())
    } else {
        Err((Code::UNAUTHORIZED, "expected sha256 hash of Govee API key as bearer token (case insensitive)"))
    }
}

#[utoipa::path(
    get,
    path = "/state",
    responses(
        (status = 200,
        description = "Successfully fetched current state of lamp.",
        body = govee_api::GetState),
        (status = 400,
        description = "Request did not match expected structure."),
        (status = 401,
        description = "Bearer authorization token was not sha256 hash of Govee API key."),
        (status = 500,
        description = "Fetching state failed, likely because of Govee API rate limit."),
    ),
    security(("authorization" = [])) // require auth
)]
async fn get_state() -> Response<Json<govee_api::GetState>> {
    govee_api::get_state().await.map_or(
        Err((Code::INTERNAL_SERVER_ERROR, "could not get state. likely because of Govee API rate limit.")),
        |state| Ok(Json(state))
    )
}

#[utoipa::path(
    get,
    path = "/clear_govee_queue",
    responses(
        (status = 200,
        description = "Successfully cleared queue of Govee API calls to make. Also queued setting brightness to default and turning lamp off."),
        (status = 400,
        description = "Request did not match expected structure."),
        (status = 401,
        description = "Bearer authorization token was not sha256 hash of Govee API key."),
    ),
    security(("authorization" = [])) // require auth
)]
async fn get_clear_govee_queue(
    State(function_queue): State<fn_queue::Queue>
) -> Response<&'static str> {
    let message = "queued clearing Govee API call queue, setting brightness and turning off";
    println!("{message}");
    fn_queue::enqueue(&function_queue, Arc::new(|govee_queue| {
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
    path = "/activate_reminder",
    responses(
        (status = 200,
        description = "Successfully queued bright orange color with high brightness to be active for about 20 seconds."),
        (status = 400,
        description = "Request did not match expected structure."),
        (status = 401,
        description = "Bearer authorization token was not sha256 hash of Govee API key."),
    ),
    security(("authorization" = [])) // require auth
)]
async fn get_activate_reminder(
    State(function_queue): State<fn_queue::Queue>
) -> Response<&'static str> {
    let message = "queued reminder activation";
    println!("{message}");
    fn_queue::enqueue(&function_queue, Arc::new(state::reminder)).await;
    Ok(message)
}

#[utoipa::path(
    get,
    path = "/activate_nightlamp",
    responses(
        (status = 200,
        description = "Successfully queued setting brightness to default for night and color to nice warm white."),
        (status = 400,
        description = "Request did not match expected structure."),
        (status = 401,
        description = "Bearer authorization token was not sha256 hash of Govee API key."),
    ),
    security(("authorization" = [])) // require auth
)]
async fn get_activate_nightlamp(
    State(function_queue): State<fn_queue::Queue>
) -> Response<&'static str> {
    let message = "queued nightlamp activation";
    println!("{message}");
    fn_queue::enqueue(&function_queue, Arc::new(state::nightlamp)).await;
    Ok(message)
}

#[utoipa::path(
    get,
    path = "/activate_daylamp",
    responses(
        (status = 200,
        description = "Successfully queued setting brightness to default for day and color to pleasant orange."),
        (status = 400,
        description = "Request did not match expected structure."),
        (status = 401,
        description = "Bearer authorization token was not sha256 hash of Govee API key."),
    ),
    security(("authorization" = [])) // require auth
)]
async fn get_activate_daylamp(
    State(function_queue): State<fn_queue::Queue>
) -> Response<&'static str> {
    let message = "queued daylamp activation";
    println!("{message}");
    fn_queue::enqueue(&function_queue, Arc::new(state::daylamp)).await;
    Ok(message)
}

#[utoipa::path(
    get,
    path = "/timers",
    responses(
        (status = 200,
        description = "Successfully returned array of current timers.",
        body = Vec<Timer>),
        (status = 400,
        description = "Request did not match expected structure."),
        (status = 401,
        description = "Bearer authorization token was not sha256 hash of Govee API key."),
    ),
    security(("authorization" = [])) // require auth
)]
async fn get_timers(
    State(timers): State<Timers>
) -> Response<Json<Vec<Timer>>> {
    Ok(Json(timers.lock().await.clone()))
}

#[utoipa::path(
    put,
    path = "/timers",
    // TimerAction (enum) cant implement IntoParams, so this doesnt work
    //params(Vec<Timer>), 
    responses(
        (status = 200,
        description = "Successfully set internal timers to provided timers. Duplicates were removed."),
        (status = 400,
        description = "Request did not match expected structure."),
        (status = 401,
        description = "Bearer authorization token was not sha256 hash of Govee API key."),
        (status = 422,
        description = "Valid JSON request body had unexpected contents."),
    ),
    security(("authorization" = [])) // require auth
)]
async fn put_timers(
    State(state): State<(Timers, SimpleTimers)>,
    extract::Json(new_timers): extract::Json<Vec<Timer>>
) -> Response<&'static str> {

    /// return given error message with status code UNPROCESSABLE_ENTITY if condition
    const fn error_if(condition: bool, message: &'static str) -> Response<()> {
        if condition {
            Err((Code::UNPROCESSABLE_ENTITY, message))
        } else {
            Ok(())
        }
    }

    // remove duplicates
    let new_timers = new_timers.into_iter().unique().collect_vec();

    // validate new timers
    for timer in &new_timers {
        error_if(*timer.get_timeday().get_hour() > 23, "timeday.hour must be <= 23")?;
        error_if(*timer.get_timeday().get_minute() > 59, "timeday.minute must be <= 59")?;
        error_if(timer.get_timeday().get_days().is_empty(), "timeday.days must not be empty")?;
        error_if(timer.get_timeday().get_days().len() > 7, "timeday.days must have <= 7 elements")?;
        error_if(timer.get_timeday().get_days().iter().any(|&d| d > 6), "every day in timeday.days has to be <= 6")?;
        match *timer.get_action() {
            TimerAction::Sunrise { duration_min, stay_on_for_min, sleep_min, nightlamp_min } => {
                error_if(duration_min < 1, "action.params.duration_min has to be >= 1")?;
                error_if(nightlamp_min > 0 && sleep_min < duration_min,
                    "action.params.sleep_min has to be >= action.params.duration_min if action.params.nightlamp_min is > 0")?;
                // limit for all: i16::MAX = 32767
                error_if(sleep_min > 32767, "action.params.sleep_min has to be <= 32767")?;
                error_if(duration_min > 32767, "action.params.duration_min has to be <= 32767")?;
                error_if(nightlamp_min > 32767, "action.params.nightlamp_min has to be <= 32767")?;
                error_if(stay_on_for_min > 32767, "action.params.stay_on_for_min has to be <= 32767")?;
            },
            TimerAction::BrightnessState { brightness } => {
                error_if(brightness < 1, "action.params.brightness has to be >= 1")?;
                error_if(brightness > 100, "action.params.brightness has to be <= 100")?;
            },
            _ => (),
        }
    }

    let (timers, simple_timers) = state;
    *timers.lock().await = new_timers;
    process_timers(&timers, &simple_timers).await;
    write_timers_to_file(&timers).await;
    Ok("timers updated.")
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
struct PowerState { power: bool }
#[utoipa::path(
    put,
    path = "/power",
    params(PowerState),
    responses(
        (status = 200,
        description = "Successfully queued requested power state."),
        (status = 400,
        description = "Request did not match expected structure."),
        (status = 401,
        description = "Bearer authorization token was not sha256 hash of Govee API key."),
        (status = 422,
        description = "Valid JSON request body had unexpected contents."),
    ),
    security(("authorization" = [])) // require auth
)]
async fn put_power(
    State(function_queue): State<fn_queue::Queue>,
    extract::Json(powerstate): extract::Json<PowerState>
) -> Response<&'static str> {
    let setstate = SetState::Power(powerstate.power);
    fn_queue::enqueue(&function_queue, Arc::new(move |govee_queue| {
        govee_queue.push_back(setstate);
    })).await;
    println!("queued {setstate:?}");
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
    responses(
        (status = 200,
        description = "Successfully queued requested brightness state."),
        (status = 400,
        description = "Request did not match expected structure."),
        (status = 401,
        description = "Bearer authorization token was not sha256 hash of Govee API key."),
        (status = 422,
        description = "Valid JSON request body had unexpected contents."),
    ),
    security(("authorization" = [])) // require auth
)]
async fn put_brightness(
    State(function_queue): State<fn_queue::Queue>,
    extract::Json(brightnessstate): extract::Json<BrightnessState>
) -> Response<&'static str> {

    if brightnessstate.brightness < 1 || brightnessstate.brightness > 100 {
        return Err((Code::UNPROCESSABLE_ENTITY, "brightness must be from 1 to 100"));
    }

    let setstate = SetState::Brightness(brightnessstate.brightness);
    fn_queue::enqueue(&function_queue, Arc::new(move |govee_queue| {
        govee_queue.push_back(setstate);
    })).await;

    println!("queued {setstate:?}");
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
    responses(
        (status = 200,
        description = "Successfully queued requested color state."),
        (status = 400,
        description = "Request did not match expected structure."),
        (status = 401,
        description = "Bearer authorization token was not sha256 hash of Govee API key."),
        (status = 422,
        description = "Valid JSON request body had unexpected contents."),
    ),
    security(("authorization" = [])) // require auth
)]
async fn put_color(
    State(function_queue): State<fn_queue::Queue>,
    extract::Json(colorstate): extract::Json<ColorState>
) -> Response<&'static str> {
    let setstate = SetState::Color((colorstate.r, colorstate.g, colorstate.b));
    fn_queue::enqueue(&function_queue, Arc::new(move |govee_queue| {
        govee_queue.push_back(setstate);
    })).await;
    println!("queued {setstate:?}");
    Ok("queued requested state")
}

/// start webserver. never terminates.
pub async fn start_server(function_queue: fn_queue::Queue, simple_timers: SimpleTimers) {
    use constants::net::{LOCALHOST, PORT};
    use utoipa_swagger_ui::SwaggerUi;
    use tokio::net::TcpListener;
    use axum::{response::Redirect, routing::{get, put}};
    use utoipa::{OpenApi, openapi::security::{SecurityScheme, Http, HttpAuthScheme}};

    /// utility struct for utoipa to register bearer http authorization.
    /// this is necessary for showing an "Authorize" button in swagger-ui.
    struct AuthHint;
    impl utoipa::Modify for AuthHint {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            if let Some(components) = openapi.components.as_mut() {
                components.add_security_scheme(
                    "authorization",
                    SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer))
                );
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
            get_activate_reminder,
            get_activate_nightlamp,
            get_activate_daylamp,
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
    let timers: Timers = load_timers(&simple_timers).await;

    // configure routes
    let app = axum::Router::new()
        // api routes
        .route("/state", get(get_state))
        .route("/clear_govee_queue", get(get_clear_govee_queue))
            .with_state(Arc::clone(&function_queue))
        .route("/activate_reminder", get(get_activate_reminder))
            .with_state(Arc::clone(&function_queue))
        .route("/activate_nightlamp", get(get_activate_nightlamp))
            .with_state(Arc::clone(&function_queue))
        .route("/activate_daylamp", get(get_activate_daylamp))
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

        // require authorization for the routes above with middleware
        .route_layer(middleware::from_fn(validate_request))

        // temporarily redirect root to swagger ui
        .route("/", get(|| async { Redirect::temporary("/swagger-ui") }))
        // swagger ui
        .merge(SwaggerUi::new("/swagger-ui")
            .url("/openapi.json", ApiDoc::openapi()));

    let address = std::net::SocketAddr::new(LOCALHOST, PORT);
    println!("WEB: starting server on http://{address} ...");
    axum::serve(TcpListener::bind(address).await.unwrap(), app).await.unwrap();
}