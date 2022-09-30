#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_dyn_templates;

mod ssh;

use rocket::fs::{FileServer, relative};
use rocket_dyn_templates::Template;

/// macro that wraps the `context` macro, and injects default parameters
macro_rules! ctx {
    ($($arg:tt)*) => {
        context! {
            domain: "git.ferris.land",
            $($arg)*
        }
    };
}

#[get("/")]
fn index() -> Template {
    Template::render("index", ctx! {})
}

#[launch]
fn rocket() -> _ {
    rocket::tokio::spawn(ssh::launch());

    rocket::build()
        .attach(Template::fairing())
        .mount("/", routes![index])
        .mount("/static", FileServer::from(relative!("static")))
}
