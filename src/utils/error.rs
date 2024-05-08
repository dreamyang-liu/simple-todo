#[derive(Debug)]
pub struct DatabaseError {
    message: String,
}

impl DatabaseError {
    pub fn new(message: &str) -> DatabaseError {
        DatabaseError {
            message: format!("{}", message.to_string()),
        }
    }
}