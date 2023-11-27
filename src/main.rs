/*
enable govee debug mode:
    create .cargo/config.toml with content:
    [build]
    rustflags = "--cfg govee_debug"
*/

// TODO check warnings
// TODO use more match instead of if
// TODO improve project structure

mod res;
mod util;
mod control;

#[tokio::main]
async fn main() {
    // await async main loop (never terminates)
    control::main_loop().await;
}