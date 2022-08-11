use std::collections::HashMap;

use my_json::json_reader::JsonFirstLineReader;

pub struct SignalrMessage<'s> {
    pub headers: Option<HashMap<String, String>>,
    pub invocation_id: Option<&'s str>,
    pub target: &'s str,
    pub arguments: &'s [u8],
}

impl<'s> SignalrMessage<'s> {
    pub fn parse(payload: &'s str) -> Self {
        let mut invocation_id = None;
        let mut target = None;
        let mut arguments = None;

        let json_reader = JsonFirstLineReader::new(payload.as_bytes());
        for line in json_reader {
            let line = line.unwrap();

            match line.get_name().unwrap() {
                "invocationId" => {
                    let result = line.get_value().unwrap();
                    invocation_id = result.as_str();
                }
                "arguments" => {
                    let result = line.get_value().unwrap();
                    arguments = result.as_bytes();
                }
                "target" => {
                    let result = line.get_value().unwrap();
                    target = result.as_str();
                }
                _ => {}
            }
        }

        if target.is_none() {
            panic!("Target is not found");
        }

        if arguments.is_none() {
            panic!("Arguments is not found");
        }
        Self {
            headers: None,
            invocation_id: invocation_id,
            target: target.unwrap(),
            arguments: arguments.unwrap(),
        }
    }
}
