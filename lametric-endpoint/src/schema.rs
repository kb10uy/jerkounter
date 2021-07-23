use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AppParameters {
    pub name: String,
    pub webhook_token: String,
}
