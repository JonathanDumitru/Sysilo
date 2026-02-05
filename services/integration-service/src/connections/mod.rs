pub mod api;

use serde::{Deserialize, Serialize};

/// Supported connector types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorType {
    Postgresql,
    Mysql,
    Snowflake,
    Oracle,
    Salesforce,
    RestApi,
}

impl std::fmt::Display for ConnectorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Postgresql => write!(f, "postgresql"),
            Self::Mysql => write!(f, "mysql"),
            Self::Snowflake => write!(f, "snowflake"),
            Self::Oracle => write!(f, "oracle"),
            Self::Salesforce => write!(f, "salesforce"),
            Self::RestApi => write!(f, "rest_api"),
        }
    }
}

/// Authentication types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    Credential,
    Oauth,
    ApiKey,
}

impl std::fmt::Display for AuthType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Credential => write!(f, "credential"),
            Self::Oauth => write!(f, "oauth"),
            Self::ApiKey => write!(f, "api_key"),
        }
    }
}

/// Validate that config contains required fields for the connector type.
/// Returns Ok(()) if valid, Err(message) if missing fields.
pub fn validate_config(
    connector_type: &ConnectorType,
    config: &serde_json::Value,
) -> Result<(), String> {
    let obj = config.as_object().ok_or("config must be a JSON object")?;

    let required_fields: &[&str] = match connector_type {
        ConnectorType::Postgresql => &["host", "port", "database"],
        ConnectorType::Mysql => &["host", "port", "database"],
        ConnectorType::Snowflake => &["account", "warehouse", "database"],
        ConnectorType::Oracle => &["host", "port", "service_name"],
        ConnectorType::Salesforce => &["instance_url"],
        ConnectorType::RestApi => &["base_url"],
    };

    let missing: Vec<&str> = required_fields
        .iter()
        .filter(|f| !obj.contains_key(**f))
        .copied()
        .collect();

    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Missing required config fields for {}: {}",
            connector_type,
            missing.join(", ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_config_postgresql_valid() {
        let config = serde_json::json!({"host": "localhost", "port": 5432, "database": "mydb"});
        assert!(validate_config(&ConnectorType::Postgresql, &config).is_ok());
    }

    #[test]
    fn test_validate_config_postgresql_missing_field() {
        let config = serde_json::json!({"host": "localhost"});
        let err = validate_config(&ConnectorType::Postgresql, &config).unwrap_err();
        assert!(err.contains("port"));
        assert!(err.contains("database"));
    }

    #[test]
    fn test_validate_config_salesforce_valid() {
        let config = serde_json::json!({"instance_url": "https://myorg.salesforce.com"});
        assert!(validate_config(&ConnectorType::Salesforce, &config).is_ok());
    }

    #[test]
    fn test_validate_config_not_object() {
        let config = serde_json::json!("not an object");
        let err = validate_config(&ConnectorType::Postgresql, &config).unwrap_err();
        assert!(err.contains("JSON object"));
    }

    #[test]
    fn test_connector_type_serialization() {
        let ct = ConnectorType::RestApi;
        let json = serde_json::to_string(&ct).unwrap();
        assert_eq!(json, "\"rest_api\"");
    }

    #[test]
    fn test_auth_type_serialization() {
        let at = AuthType::ApiKey;
        let json = serde_json::to_string(&at).unwrap();
        assert_eq!(json, "\"api_key\"");
    }
}
