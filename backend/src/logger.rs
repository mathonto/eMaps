use std::error::Error;

use log4rs::append::console::ConsoleAppender;
use log4rs::config::Appender;
use log4rs::config::Root;
use log4rs::encode::pattern::PatternEncoder;
use log::LevelFilter;

const PKG_NAME: &str = env!("CARGO_PKG_NAME");

pub fn init() -> Result<(), Box<dyn Error>> {
    let pkg = log4rs::config::Logger::builder()
        .build(PKG_NAME, LevelFilter::Debug);
    let actix_web = log4rs::config::Logger::builder()
        .build("actix_web", LevelFilter::Info);

    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(&format())))
        .build();
    let appender = Appender::builder()
        .build("stdout", Box::new(stdout));
    let root = Root::builder()
        .appender("stdout")
        // disable logging for libs
        .build(LevelFilter::Off);

    let config = log4rs::config::Config::builder()
        .appender(appender)
        .logger(pkg)
        .logger(actix_web)
        .build(root)?;
    log4rs::init_config(config)?;
    Ok(())
}

fn format() -> String {
    let date = "{d(%H:%M:%S)}";
    let log_level = "[{l}]";
    let origin = "{M}";
    let message = "{m}{n}";

    let format = format!(
        "{} {} {} - {}",
        date,
        log_level,
        origin,
        message,
    );
    let colorized = format!("{{h({})}}", format);
    colorized
}
