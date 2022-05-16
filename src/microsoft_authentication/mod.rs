pub mod xbox;
pub mod minecraft;
pub mod microsoft;
#[cfg(feature = "account_api")]
pub mod account;
#[derive(Clone, Debug)]
pub struct AuthProperties {
    pub azura_microsoft_client: String,
    pub microsoft_value: String,
    pub secret_id: String,
}
