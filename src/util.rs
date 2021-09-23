pub fn parse_num(input: &str) -> Result<f64, String> {
    input
        .parse::<f64>()
        .map_err(|_| format!("Can't parse {} as a number", input))
}
