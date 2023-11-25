pub mod govee;
pub mod timeday;

pub enum HttpMethod {
    Get,
    /// contains request body
    Put(String)
}

pub async fn send_api_request(method: HttpMethod, url: &str, headers: Option<Vec<(&str, &str)>>) -> Result<serde_json::Value, &'static str> {
    let client =  reqwest::Client::new();
    let mut request = match method {
        HttpMethod::Get    => client.get(url),
        HttpMethod::Put(_) => client.put(url)
    };

    // set headers (if given)
    if let Some(headers) = headers {
        for header in headers {
            request = request.header(header.0, header.1);
        }
    }

    // set body (if given)
    if let HttpMethod::Put(body) = method {
        request = request.body(body);
    }
    
    let result = request.send().await;
    let Ok(response) = result else {
        return Err("sending request failed");
    };

    let json = response.json::<serde_json::Value>().await;
    if let Ok(json) = json {
        Ok(json)
    } else {
        Err("parsing request response json failed")
    }
}