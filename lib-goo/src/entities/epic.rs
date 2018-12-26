//! Represent the current epic the user is working on. This is managed by the user from
//! the commmand line, it is an optional piece of information in the system.

#[derive(::serde::Deserialize, ::serde::Serialize)]
pub struct Epic {
    pub name: String,
}
