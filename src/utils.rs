pub fn new_error<T>(msg: &str) -> Result<T, Box<dyn std::error::Error>> {
    Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        msg,
    )))
}
