use std::env;
use log4rs::append::console::{ConsoleAppender, Target};
use log4rs::append::file::FileAppender;
use log4rs::Config;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::filter::threshold::ThresholdFilter;
use log::LevelFilter;


pub fn init_logger() {
    if let Err(e) = log4rs::init_file("log4rs.yaml", Default::default()) {
        println!("日志配置文件初始化失败:{e},使用默认日志配置");
        let mut builder = Config::builder();
        let mut root_builder = Root::builder();
        let level = log::LevelFilter::Info;
        let stderr = ConsoleAppender::builder()
            .target(Target::Stderr).build();
        let pattern = "{d(%Y-%m-%d %H:%M:%S)} {M} {L} {h({l})} - {m}{n}";

        if let Ok(file_path) = env::var("LOG_DIR") {
            let file = format!("{}{}", file_path, "/log.log");
            let logfile = FileAppender::builder()
                // Pattern: https://docs.rs/log4rs/*/log4rs/encode/pattern/index.html
                .encoder(Box::new(PatternEncoder::new(pattern)))
                .build(file)
                .expect("Failed to create file appender");
            let logfile_appender = Appender::builder()
                .build("logfile", Box::new(logfile));
            builder = builder.appender(logfile_appender);
            root_builder = root_builder.appender("logfile");
        };

        builder = builder.appender(Appender::builder()
            .filter(Box::new(ThresholdFilter::new(level)))
            .build("stderr", Box::new(stderr)));
        root_builder = root_builder.appender("stderr");

        let config = builder
            .build(
                root_builder.build(level)
            )
            .expect("Failed to build log4rs config");
        log4rs::init_config(config).expect("Failed to initialize log4rs");
    }
}