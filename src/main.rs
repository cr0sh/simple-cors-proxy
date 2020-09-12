use tide::{
    http::headers::HeaderValue, security::CorsMiddleware, security::Origin, Request, Response,
};

const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

#[async_std::main]
async fn main() {
    #[cfg(debug_assertions)]
    tide::log::start();

    #[cfg(not(debug_assertions))]
    tide::log::with_level(tide::log::LevelFilter::Error);

    let mut app = tide::new();

    let cors = CorsMiddleware::new()
        .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false);

    app.with(cors.clone()).at("/check").get(|_| async {
        Ok(format!(
            "simple-cors-proxy v{}",
            VERSION.unwrap_or("Unknown")
        ))
    });

    app.with(cors)
        .at("/get/*url")
        .get(|req: Request<()>| async move {
            let url: String = req.param("url")?;
            tide::log::info!("url: {}", url);
            let mut resp = surf::get(url).await.map_err(|e| anyhow::anyhow!(e))?;
            let headers = resp
                .headers()
                .iter()
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .collect::<Vec<_>>();

            let mut resp = Response::builder(tide::StatusCode::Ok)
                .body(resp.body_bytes().await?)
                .header("Access-Control-Allow-Origin", "*");

            for (k, v) in headers {
                resp = resp.header(k.as_str(), v.as_str());
            }

            return anyhow::Result::Ok(resp.build());
        });

    app.listen("127.0.0.1:5803")
        .await
        .unwrap_or_else(|e| tide::log::error!("Error: {}", e));
}
