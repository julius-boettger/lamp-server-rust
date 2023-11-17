/*
debug mode:
    create .cargo/config.toml with content:
    [build]
    rustflags = "--cfg govee_debug"
*/

mod res;
mod util;
mod view;
mod control;

#[tokio::main]
async fn main() {
    // "fire and forget" async main loop
    tokio::spawn(control::main_loop());
    // await async server start (never terminates)
    control::web::start_server().await;
}