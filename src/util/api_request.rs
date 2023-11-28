pub enum Method {
    Get,
    /// contains request body
    Put(String)
}

pub async fn send(method: Method, url: &str, headers: Option<Vec<(&str, &str)>>) -> Result<serde_json::Value, &'static str> {
    let client =  reqwest::Client::new();
    let mut request = match method {
        Method::Get    => client.get(url),
        Method::Put(_) => client.put(url)
    };

    // set headers (if given)
    if let Some(headers) = headers {
        for header in headers {
            request = request.header(header.0, header.1);
        }
    }

    // set body (if given)
    if let Method::Put(body) = method {
        request = request.body(body);
    }
    
    let result = request.send().await;
    let Ok(response) = result else {
        return Err("sending request failed");
    };

    let json = response.json::<serde_json::Value>().await;
    // replace error with custom message
    json.map_err(|_| "parsing request response json failed")
}