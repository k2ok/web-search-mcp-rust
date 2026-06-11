pub struct UserAgent {
    default: String,
}

impl UserAgent {
    pub fn new(default: String) -> Self {
        Self { default }
    }

    /// Returns the default User-Agent configured at startup.
    pub fn default(&self) -> &str {
        &self.default
    }

}
