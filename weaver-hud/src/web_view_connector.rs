
#[cfg(feature="use-web-view")]
pub fn start() {
    use web_view::*;
    let size = (300, 300);
    let resizable = true;
    let debug = true;
    let init_cb = |_webview| {};
    let frontend_cb = |_webview: &mut _, args: &str, _userdata: &mut _| {
        crate::util::open_url(args);
    };
    let userdata = ();
    let config = crate::util::parse();
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