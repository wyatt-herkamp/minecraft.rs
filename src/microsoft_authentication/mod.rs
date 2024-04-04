pub mod account;
pub mod microsoft;
pub mod minecraft;
pub mod xbox;

pub use account::*;
pub use microsoft::*;
pub use minecraft::*;
use serde::{Deserialize, Serialize};
pub use xbox::*;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthProperties {
    pub azura_microsoft_client: String,
    pub microsoft_value: String,
    pub secret_id: String,
}
