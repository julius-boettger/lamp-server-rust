mod util;
mod control;
mod constants;

#[tokio::main]
async fn main() {
    // await async main loop (never terminates)
    control::main_loop().await;
}