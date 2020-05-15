#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate rand;
extern crate redis;
extern crate rocket_contrib;

use rand::distributions::Alphanumeric;
use rand::Rng;
use std::iter;
use std::sync::Mutex;

use rocket::http::RawStr;
use rocket::State;
use rocket::request::Form;
use rocket::response::Redirect;
use rocket::response::content::Json;
use redis::Commands;

#[derive(FromForm)]
struct GetURL {
    value: String
}

fn random_string() -> String {
    let chars: String = iter::repeat(())
        .map(|()| rand::thread_rng().sample(Alphanumeric))
        .take(7)
        .collect();

    return chars;
}

struct DB {
    connector: redis::Connection,
}

impl DB {
    fn new() -> DB {
        let connector = redis::Client::open("redis://127.0.0.1/")
            .unwrap()
            .get_connection()
            .unwrap();
        DB { connector }
    }

    fn set_associated_id_for_url(&mut self, url: String) -> Option<String> {
        let id = random_string();

        match &self.connector.set::<String, String, ()>(id.clone(), url) {
            Ok(_) => Some(id),
            Err(_) => None
        }
    }

    fn get_url_with_id(&mut self, id: String) -> Option<String> {
        match &self.connector.get::<String, String>(id) {
            Ok(url) => Some(url.to_string()),
            Err(_) => None
        }
    }
}

#[get("/<id>")]
fn get_id(db: State<Mutex<DB>>, id: &RawStr) -> Redirect {
    match db.lock().unwrap().get_url_with_id(id.to_string()) {
        Some(url) => Redirect::moved(url),
        None => Redirect::to("/")
    }
}

#[post("/shortener", data = "<url>")]
fn shortener(db: State<Mutex<DB>>, url: Form<GetURL>) -> Json<String> {
    match db.lock().unwrap().set_associated_id_for_url(url.value.clone()) {
        Some(id) => Json(format!(r#"{{ "data": {{ "id": "{}" }} }}"#, id)),
        None => Json(format!(r#"{{"error": {{"code": 501, "message": "" }} }}"#))
    }
}

fn main() {
    rocket::ignite()
        .manage(Mutex::new(DB::new()))
        .mount("/", routes![shortener, get_id])
        .launch();
}
