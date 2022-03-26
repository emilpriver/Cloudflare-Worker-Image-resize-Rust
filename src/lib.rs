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

    router
        .get("/", |req, ctx| {
            let request_url = req.url().unwrap_or_else(|err| panic!("Could not parse '{:?}': {}", stringify!($var), err));
            let url = Url::parse(request_url.as_str())?;
            let query = url.query();
            
            console_log!("{:?}" ,query);

            Response::ok(req.url()?.to_string())
        })
        .get("/worker-version", |_, ctx| {
            let version = ctx.var("WORKERS_RS_VERSION")?.to_string();
            Response::ok(version)
        })
        .run(req, env)
        .await
}
