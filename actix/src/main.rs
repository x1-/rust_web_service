extern crate actix;
extern crate actix_web;
extern crate futures;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;

// use actix;
use actix_web::{http, server, App,  AsyncResponder, Either, Error, HttpRequest, HttpResponse, Path, Query, Responder};
use futures::future::{Future, result};

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Deserialize)]
struct Info {
    username: String,
}
#[derive(Serialize)]
struct Greeting {
    message: String,
    status: i32
}

impl Responder for Greeting {
    type Item = HttpResponse;
    type Error = Error;

    fn respond_to<S>(self, req: &HttpRequest<S>) -> Result<HttpResponse, Error> {
        let body = serde_json::to_string(&self)?;

        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body))
    }
}

struct MyState {
    share_info: String,
    access_count: AtomicUsize
}

fn health(_: HttpRequest<Arc<MyState>>) -> &'static str {
    "ok"
}

fn greeting(_: HttpRequest<Arc<MyState>>) -> impl Responder {
    Greeting {
        message: "hello".to_string(),
        status: 100
    }
}

fn counter(req: HttpRequest<Arc<MyState>>) -> String {
    let state = req.state();
    let count = state.access_count.fetch_add(1, Ordering::SeqCst) + 1;
    format!("Share info: {}, Access count: {}", state.share_info, count)
}

fn complex(data: (Path<String>, Query<Info>)) -> impl Responder {
    let (path, query) = data;
    Greeting {
        message: format!("Welcome {} {}!", query.username, path.into_inner()),
        status: 100
    }
}

type RegisterResult = Either<HttpResponse, Box<Future<Item=HttpResponse, Error=Error>>>;

fn either(path: Path<u32>) -> impl Responder {
    if path.into_inner() == 0 {
        Either::A(
            HttpResponse::BadRequest().body("Bad data"))
    } else {
        Either::B(
            result::<HttpResponse, Error>(Ok(HttpResponse::Ok()
                   .content_type("text/html")
                   .body(format!("Hello!")))).responder())
    }
}


fn main() {
    let sys = actix::System::new("my example");

    let app_state = Arc::new(MyState {
        share_info  : String::from("my example server"),
        access_count: AtomicUsize::new(0)
    });
    server::HttpServer::new(move ||
        App::with_state(app_state.clone())
            .route("/hc", http::Method::GET, health)
            .resource("/complex/{party}", |r| r.method(http::Method::GET).with(complex))
            .resource("/counter", |r| r.f(counter))
            .resource("/greeting", |r| r.f(greeting))
            .resource("/either/{num}", |r| r.method(http::Method::GET).with(either))
    )
        .workers(4)
        .backlog(64)  // Generally set in the 64-2048 range. Default value is 2048.
        .keep_alive(server::KeepAlive::Timeout(75)) // By default keep alive is set to a Os.
        .server_hostname("my-example.local".to_string())
        .bind("127.0.0.1:8088")
        .unwrap()
        .start();

    println!("Started http server on: 127.0.0.1:8088");

    let _ = sys.run();
}
