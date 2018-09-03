## lib-server

Implement the Weaver server functionality. There are two parts of the server,
one for rendering the static html and a second one for the API.


## Pages

Most of the content is generated on the server side. The server side
templates are written in (tera)[http://crates.io/tera].

The static assets are embedded in the application. They consist of templates
and css files. We use the (bulma)[http://bulma.io] css library and webpack to
build our css. This code is under `web` and is build using the `web/build.sh`
script. The `cargo` build process invokes this script, see the code in
`build.rs`.



## Api

The api calls are hosted under `src/api`.

## Dev mode

When working on the static assets it is more convenient to start the server
in foreground with the following command:

    RUST_BACKTRACE=1 WEAVER=info cargo run -p weaver-server -- -p 8888 start --fg


When the assets are changing in a separate console run the following command to refresh them:

    curl -v http://127.0.0.1:8888/reload