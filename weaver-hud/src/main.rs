#[cfg(osx)]
extern crate web_view;


#[cfg(osx)]
fn main() {
    use web_view::*;
    let size = (300, 300);
    let resizable = true;
    let debug = true;
    let init_cb = |_webview| {};
    let frontend_cb = |_webview: &mut _, _arg: &_, _userdata: &mut _| {};
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

#[cfg(not(osxa))]
fn main() {
  println!("not supported");
}
