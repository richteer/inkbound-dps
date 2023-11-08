pub mod parser;
mod testparse;

pub fn parse_log_to_json(path: &String) -> String {
    let mut log_parser = crate::parser::LogParser::new();
    let mut data_log = crate::parser::DataLog::new();

    let file = std::fs::read_to_string(path).unwrap();
    let file: Vec<&str> = file.split('\n').collect();

    let events = log_parser.parse_lines(file.as_slice());
    data_log.handle_events(events);

    serde_json::to_string(&data_log).unwrap()
}

#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn it_works() {
    //     let result = add(2, 2);
    //     assert_eq!(result, 4);
    // }
}
