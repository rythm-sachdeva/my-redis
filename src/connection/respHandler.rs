
pub fn parse_resp(input: &str) -> Vec<String> {
    let mut lines = input.lines();
    let mut result = Vec::new();

    if let Some(line) = lines.next() {
        if !line.starts_with('*') {
            return result;
        }
    }

    while let Some(line) = lines.next() {
        if line.starts_with('$') {
            if let Some(data) = lines.next() {
                result.push(data.to_string());
            }
        }
    }
    result
}