#[cfg(target_os="macos")]
extern crate web_view;
extern crate serde_json;
use serde_json as json;
use std::process::Command;


#[cfg(target_os="macos")]
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
                    .status().expect("running open");
                },
                _ => {}
            }
        }
    };
    let userdata = ();
    run(
        "Weaver",
        Content::Url("http://localhost:8466/hud"),
        Some(size),
        resizable,
        debug,
        init_cb,
        frontend_cb,
        userdata,
    );
}

#[cfg(not(target_os="macos"))]
fn main() {
  println!("not supported");
}
