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
    control::web::start_server().await;
}