/// How to get the password.
#[derive(Debug, PartialEq)]
pub enum PasswordSource {
    /// Prompt the user for the password
    Prompt,
    /// Get from the key ring
    Keyring
}
