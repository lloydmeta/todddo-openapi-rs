use api;

static LOG_ENV_KEY: &str = "RUST_LOG";

fn main() -> Result<(), std::io::Error> {
    setup_logging();
    api::run_server()
}

fn setup_logging() {
    let _ = std::env::var(LOG_ENV_KEY)
        .map_err(|_| std::env::set_var(LOG_ENV_KEY, "info,actix_web=info,api=info"));
    env_logger::init();
}
