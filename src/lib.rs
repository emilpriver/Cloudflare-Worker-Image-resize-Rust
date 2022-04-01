use std::collections::HashMap;
use std::io::Cursor;
use worker::*;

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

    // use `if` instead
    if req.method() != Method::Get {
        return Response::error("Method Not Allowed", 405);
    }

    router
        .get_async("/", |req, _ctx| async move {
            let request_url = req
                .url()
                .unwrap_or_else(|err| panic!("Could not parse '{:?}': {}", stringify!($var), err));

            let request_headers = req.headers();
            let parsed_url = Url::parse(request_url.as_str()).unwrap();
            let hash_query: HashMap<_, _> = parsed_url.query_pairs().into_owned().collect();

            let image_src = match hash_query.get("src") {
                Some(val) => val,
                None => return Ok(Response::error("Missing src", 400).unwrap()),
            };

            let image_width = match hash_query.get("w") {
                Some(val) => val.parse::<u32>().unwrap(),
                None => return Ok(Response::error("Missing width", 400).unwrap()),
            };

            let image_quality = match hash_query.get("src") {
                Some(val) => val.parse::<u8>().unwrap(),
                None => 80,
            };

            let supported_image_format = request_headers
                .get("Accept")
                .unwrap_or_else(|_| Some("image/jpeg".to_string()));

            let client = reqwest::Client::new();

            let resp = client.get(image_src).send().await;

            match resp.status() {
                reqwest::StatusCode::OK => {
                    let image_to_bytes = match resp {
                        Some(resp) => match resp.bytes().await {
                            Some(bytes) => bytes,
                            None => {
                                return Ok(Response::error(
                                    "Error converting image into bytes",
                                    400,
                                )
                                .unwrap())
                            }
                        },
                        None => {
                            return Ok(
                                Response::error("Error loading image from origin", 400).unwrap()
                            )
                        }
                    };

                    let image = match image::load_from_memory(&image_to_bytes) {
                        Some(value) => value,
                        None => {
                            return Ok(
                                Response::error("Error loading image from memory", 400).unwrap()
                            )
                        }
                    };

                    // Type inferred
                    // Does not have to be mutable as it's not changing
                    let image_transform_format = image::ImageFormat::Jpeg;
                    // Type inferred
                    // Does not have to be mutable as it's not changing
                    let image_transform_format_header = "image/jpeg";

                    // if format!("{:?}", supported_image_format).contains("image/webp") {
                    //   image_transform_format = image::ImageFormat::WebP;
                    //   image_transform_format_header = "image/webp".to_string();
                    // }

                    // if format!("{:?}", supported_image_format).contains("image/avif") {
                    //   image_transform_format =  image::ImageFormat::Avif;
                    //   image_transform_format_header = "image/avif".to_string();
                    // }

                    let image = match image.resize(
                        image_width,
                        u32::MAX,
                        image::imageops::FilterType::Nearest,
                    ) {
                        Some(value) => value,
                        None => {
                            return Ok(Response::error("Error when resizing image", 400).unwrap())
                        }
                    };

                    // Remove the type for `new_image` as it's inferred
                    let mut new_image =
                        Vec::with_capacity(image.width() as usize * image.height() as usize);

                    image
                        .write_to(&mut Cursor::new(&mut new_image), image_transform_format)
                        .expect("Error writing image");

                    let mut headers = worker::Headers::new();
                    let _ = headers.set("Access-Control-Allow-Headers", "Content-Type");
                    let _ = headers.set("Content-Type", &image_transform_format_header);
                    let _ = headers.set("Cache-Control", "max-age=2629746");

                    // Type is inferred
                    let body = ResponseBody::Body(new_image);

                    // Implicit return (learn to love it)
                    Ok(Response::from_body(body).unwrap().with_headers(headers))
                }
                _ => Ok(Response::error("Bad Request", 400).unwrap()),
            }
        })
        .get("/worker-version", |_, ctx| {
            let version = ctx.var("WORKERS_RS_VERSION")?.to_string();
            Response::ok(version)
        })
        .run(req, env)
        .await
}
