/// How to get the password.
#[derive(Debug, PartialEq)]
pub enum PasswordSource {
    /// Prompt the user for the password
    Prompt,
    /// Get from the key ring
    Keyring,
    /// Pass in
    PassIn(String),
    /// Read from WEAVER_PASSWORD
    Environment
}
