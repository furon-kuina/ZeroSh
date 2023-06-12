pub fn parse_command_line(input: String) -> Option<Vec<String>> {
    match input.is_empty() {
        true => None,
        false => {
            let commands = input.split('|').map(|s| s.to_string()).collect::<Vec<_>>();
            Some(commands)
        }
    }
}
