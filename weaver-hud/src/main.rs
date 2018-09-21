extern crate clap;
extern crate serde_json;
#[cfg(target_os = "macos")]
extern crate web_view;

use serde_json as json;
use std::process::Command;

pub const APP_NAME: &str = env!["CARGO_PKG_NAME"];
const VERSION: &str = env!("CARGO_PKG_VERSION");
const DESCRIPTION: &str = env!["CARGO_PKG_DESCRIPTION"];

struct Config {
    port: u16,
}

// Parse the command line
fn parse() -> Config {
    use clap::{App, Arg };

    let matches = App::new(APP_NAME)
        .version(VERSION)
        .about(DESCRIPTION)
        .arg(
            Arg::with_name("version")
                .short("V")
                .help("Display the version"),
        ).arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .takes_value(true)
                .value_name("PORT")
                .help("Select a port to connect to the weaver server"),
        ).get_matches();

    let port = matches
        .value_of("port")
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(8466);
    Config { port }
}

#[cfg(target_os = "macos")]
fn main() {
    use web_view::*;
    let size = (300, 300);
    let resizable = true;
    let debug = true;
    let init_cb = |_webview| {};
    let frontend_cb = |_webview: &mut _, arg: &str, _userdata: &mut _| {
        if let Ok(v) = json::from_str::<json::Value>(arg) {
            match v["href"] {
                json::Value::String(ref s) => {
                    Command::new("/bin/sh")
                        .arg("-c")
                        .arg(format!("open {}", s))
                        .status()
                        .expect("running open");
                }
                _ => { println!("cannot process frontend callback {:?}", v)}
            }
        } else {
            println!("cannot decode frontend message {:?}", arg)
        }
    };
    let userdata = ();
    let config = parse();
    let url = format!("http://localhost:{}/hud", config.port);
    run(
        "Weaver",
        Content::Url(url),
        Some(size),
        resizable,
        debug,
        init_cb,
        frontend_cb,
        userdata,
    );
}

#[cfg(not(target_os = "macos"))]
fn main() {
    println!("not supported");
}
