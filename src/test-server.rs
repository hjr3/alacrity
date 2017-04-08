#[macro_use]
extern crate rustful;

extern crate log;
extern crate env_logger;

use std::env;
use std::path::Path;
use rustful::{Server, Context, Response, TreeRouter};

fn index(_context: Context, response: Response) {
    response.send("Hello World");
}

fn large(_context: Context, response: Response) {
    let pwd = env!("CARGO_MANIFEST_DIR");
    let path = format!("{}/tests/jquery-1.7.1.min.js", pwd);
    let path = Path::new(&path);

    let _ = response.send_file(path)
        .or_else(|e| e.send_not_found("the file was not found"))
        .or_else(|e| e.ignore_send_error());
}

fn main() {

    let threads = env::var("THREADS").ok().and_then(|t| t.parse::<usize>().ok().or(None));

    env_logger::init().expect("Failed to init logger");

    Server {
            host: 12346.into(),
            handlers: insert_routes!{
            TreeRouter::new() => {
                "/" => Get: index as fn(Context, Response),
                "/large" => Get: large as fn(Context, Response),
            }
        },
            threads: threads,
            ..Server::default()
        }
        .run()
        .expect("Could not start server");
}