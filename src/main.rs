mod res;
mod util;

#[tokio::main]
async fn main() {
    // example http request
    let response = reqwest::get("https://httpbin.org/ip").await.expect("error sending request");
    let json = response.json::<std::collections::HashMap<String, String>>().await.expect("response is not json");
    println!("got json response:\n{:#?}", json);
}