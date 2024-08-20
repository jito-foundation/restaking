use std::io::Write;

use chrono::Local;
use env_logger::{
    fmt::{Color, Formatter, Style, StyledValue},
    Env,
};
use log::Record;
pub fn init_logger() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format(format_log_message)
        .init();
}

fn format_log_message(buf: &mut Formatter, record: &Record) -> std::io::Result<()> {
    let mut style = buf.style();
    let level = colored_level(&mut style, record.level());

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");

    writeln!(
        buf,
        "[{} {} {}] {}",
        timestamp,
        level,
        record.target(),
        record.args()
    )
}

fn colored_level(style: &mut Style, level: log::Level) -> StyledValue<&'static str> {
    match level {
        log::Level::Trace => style.set_color(Color::Magenta).value("TRACE"),
        log::Level::Debug => style.set_color(Color::Blue).value("DEBUG"),
        log::Level::Info => style.set_color(Color::Green).value("INFO "),
        log::Level::Warn => style.set_color(Color::Yellow).value("WARN "),
        log::Level::Error => style.set_color(Color::Red).value("ERROR"),
    }
}
