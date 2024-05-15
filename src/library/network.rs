pub mod http_server;

use crate::context::Context;

use self::http_server::setup_lib_http_server;

// #todo extract http/server as foreign dyn-lib
// #todo network/smtp

pub fn setup_lib_network(context: &mut Context) {
    setup_lib_http_server(context);
}
