#[cfg(target_os = "macos")]
#[cfg(feature="use-tether")]
pub fn start() {
    let config = crate::util::parse();
    let url = format!("http://localhost:{}/hud", config.port);
    let bootstrap = format!("<a href=\"{}\">click</a><script>window.location=\"{}\";</script>", url, url);
    tether::builder()
        .html(&bootstrap)
        .handler(|_, args: &str| {
            crate::util::open_url(args);
        })
        .start();
}
