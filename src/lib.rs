use worker::*;
use std::collections::HashMap;

mod utils;
mod image_resize;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    utils::set_panic_hook();

    let router = Router::new();

    router
        .get_async("/", |req, ctx| async move {
            let request_url = req.url().unwrap_or_else(|err| panic!("Could not parse '{:?}': {}", stringify!($var), err));
            let parsed_url = Url::parse(request_url.as_str())?;
            let hash_query: HashMap<_, _> = parsed_url.query_pairs().into_owned().collect();

            let image_src = hash_query.get("src").unwrap();
            let image_width = hash_query.get("w").unwrap();
            let image_quality = hash_query.get("q").unwrap();
            
            console_log!("{:?}" ,image_src);
            console_log!("{:?}" ,image_width);
            console_log!("{:?}" ,image_quality);

            let client = reqwest::Client::new();

            let resp = client
              .get(String::from(image_src))
              .send()
              .await
              .unwrap();
        
            match resp.status() {
              reqwest::StatusCode::OK => {
                let data = resp.bytes().await.expect("error loading bytes");
                let data = data.collect().expect("Unable to read data");
                let image = image::load_from_memory(&data).expect("Error loading image from memory");


                Response::ok(hash_query.get("src").unwrap())
              }
              reqwest::StatusCode::UNAUTHORIZED => return Response::error("Bad Request", 401),
              _ => return Response::error("Bad Request", 400)
          }
        })
        .get("/worker-version", |_, ctx| {
            let version = ctx.var("WORKERS_RS_VERSION")?.to_string();
            Response::ok(version)
        })
        .run(req, env)
        .await
}
