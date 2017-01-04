#[macro_use]
extern crate mime;
extern crate iron;
extern crate rustc_serialize;

use iron::status;
use iron::headers::ContentType;
use iron::prelude::*;

use rustc_serialize::base64::{self, ToBase64};
use rustc_serialize::json;

mod routing;

#[derive(RustcEncodable)]
pub struct Letter {
    title: String,
    message: String
}

fn json(_: &mut Request) -> IronResult<Response> {
    let letter = Letter {
        title: "PPAP!".to_string(),
        message: "I have a pen. I have an apple.".to_string()
    };
    let payload = json::encode(&letter).unwrap();
    Ok(Response::with((ContentType::json().0, status::Ok, payload)))
}

fn gif(_: &mut Request) -> IronResult<Response> {
    let px1 = "R0lGODlhAQABAIAAAP///wAAACH5BAEAAAAALAAAAAABAAEAAAICRAEAOw==";
    Ok(Response::with((mime!(Image/Gif), status::Ok, px1.as_bytes().to_base64(base64::STANDARD))))
}

fn bad(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with(status::BadRequest))
}

fn main() {
    let mut router = routing::Router::new();

    router.add_route("json".to_string(), json);
    router.add_route("gif".to_string(), gif);

    router.add_route("error".to_string(), bad);

    let host = "localhost:3000";

    println!("binding on {}", host);
    Iron::new(router).http(host).unwrap();
}
