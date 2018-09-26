mod util;
mod web_view_connector;
mod tether_connector;

#[cfg(target_os = "macos")]
fn main() {
    #[cfg(feature = "use-web-view")]
    {
        crate::web_view_connector::start();
    }
    #[cfg(feature = "use-tether")]
    {
        crate::tether_connector::start();
    }
}

#[cfg(not(target_os = "macos"))]
fn main() {
    println!("not supported");
}
