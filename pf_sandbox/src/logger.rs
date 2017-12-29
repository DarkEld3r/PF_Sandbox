use env_logger::Builder;
use env_logger::fmt::Formatter;
use std::io::Write;
use std::io;
use std::env;
use log::{Record, Level};
use termcolor::Color;

pub fn init() {
    if let Ok(env_var) = env::var("PFS_LOG") {
        Builder::new().format(format).parse(&env_var).init()
    }
}

fn format(buf: &mut Formatter, record: &Record) -> io::Result<()> {
    let level = record.level();
    let level_color = match level {
        Level::Trace => Color::White,
        Level::Debug => Color::Blue,
        Level::Info  => Color::Green,
        Level::Warn  => Color::Yellow,
        Level::Error => Color::Red,
    };

    let write_level = write!(buf.color(level_color), "{:>5}:", level);
    let write_args = if let Some(module_path) = record.module_path() {
        writeln!(buf, " {} {}", module_path, record.args())
    }
    else {
        writeln!(buf, " {}", record.args())
    };

    write_level.and(write_args)
}