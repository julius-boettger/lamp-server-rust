/*
enable govee debug mode:
    create .cargo/config.toml with content:
    [build]
    rustflags = "--cfg govee_debug"
*/

mod util;
mod control;
mod constants;

#[tokio::main]
async fn main() {
    // await async main loop (never terminates)
    control::main_loop().await;
}