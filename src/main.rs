use localhost::log::init_logs;
use localhost::server::start;
use localhost::server::config::server_config;

fn main() {
    init_logs();
    start(server_config());
}
