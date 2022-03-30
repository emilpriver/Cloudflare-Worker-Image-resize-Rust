use worker::*;
use std::collections::HashMap;
use std::str;
use std::io::Cursor;
use std::option::Option;
use webp::*;

mod utils;

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

    if !matches!(req.method(), Method::Get) {
      return Response::error("Method Not Allowed", 405);
  }

    router
        .get_async("/", |req, ctx| async move {
            let request_url = req.url().unwrap_or_else(|err| panic!("Could not parse '{:?}': {}", stringify!($var), err));
            let request_headers = req.headers();
            let parsed_url = Url::parse(request_url.as_str()).unwrap();
            let hash_query: HashMap<_, _> = parsed_url.query_pairs().into_owned().collect();

            let image_src = hash_query.get("src").unwrap();
            let image_width = hash_query.get("w").unwrap().parse::<u32>().unwrap();
            let image_quality = hash_query.get("q").unwrap().parse::<u8>().unwrap();

            let mut supported_image_format = request_headers
              .get("Accept")
              .unwrap_or(Some("image/jpeg".to_string()));
            let supported_image_format = format!("{:?}", &supported_image_format);

            let client = reqwest::Client::new();

            let resp = client
              .get(String::from(image_src))
              .send()
              .await
              .unwrap();
        
            match resp.status() {
              reqwest::StatusCode::OK => {
                let data = resp.bytes().await.expect("error loading bytes");
                let image = image::load_from_memory(&data).expect("Error loading image from memory");
                
                let mut new_image: Vec<u8> = Vec::new();

                let mut image_transform_format: image::ImageFormat = image::ImageFormat::Jpeg; 
                let mut image_transform_format_header: String = "image/jpeg".to_string();


                if formated_format_string.contains("image/avif") {

                } else if formated_format_string.contains("image/webp") {
                  // Create the WebP encoder for the above image
                  let encoder: Encoder = Encoder::from_image(&img).unwrap();
                  // Encode the image at a specified quality 0-100
                  let webp: WebPMemory = encoder.encode(90f32);
                } else {
                  image
                  .resize(image_width, u32::MAX, image::imageops::FilterType::Nearest)
                  .write_to(
                    &mut Cursor::new(&mut new_image),
                    image_transform_format
                  )
                  .expect("Error writing image");
                }

                let mut headers =worker::Headers::new();
                headers.set("Access-Control-Allow-Headers","Content-Type");
                headers.set("Content-Type",&image_transform_format_header);
                headers.set("Cache-Control", "max-age=2629746");

                let body: worker::ResponseBody = ResponseBody::Body(new_image);

                return Ok(Response::from_body(body).unwrap().with_headers(headers))
              }
              _ => return Ok(Response::error("Bad Request", 400).unwrap())
            }
        })
        .get("/worker-version", |_, ctx| {
            let version = ctx.var("WORKERS_RS_VERSION")?.to_string();
            Response::ok(version)
        })
        .run(req, env)
        .await
}
  