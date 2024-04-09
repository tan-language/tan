use crate::context::Context;

use self::http_client::setup_lib_http_client;

pub mod http_client;
// pub mod http_server;

// #todo network/smtp

pub fn setup_lib_network(context: &mut Context) {
    setup_lib_http_client(context);
    // setup_lib_http_server(context);
}
