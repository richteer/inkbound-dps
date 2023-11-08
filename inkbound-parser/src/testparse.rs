
#[cfg(test)]
mod tests {

    #[test]
    fn test_logfile() {
        // TODO: use parse_log_to_json
        // env_logger::Builder::new()
        //     .filter_level(log::LevelFilter::Debug)
        //     .init();

        // debug!("parsing log...");

        let mut log_parser = crate::parser::LogParser::new();
        let mut data_log = crate::parser::DataLog::new();

        let file = std::fs::read_to_string("logfile.log").unwrap();
        let file: Vec<&str> = file.split('\n').collect();

        let events = log_parser.parse_lines(file.as_slice());
        data_log.handle_events(events);

        println!("{}", serde_json::to_string(&data_log).unwrap());
    }
}
