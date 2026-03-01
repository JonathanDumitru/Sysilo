pub mod api;
#[path = "../connectors/registry.rs"]
pub mod registry;

use serde::{Deserialize, Serialize};

/// Supported connector types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionLifecycleStatus {
    Draft,
    Tested,
    Active,
    Error,
}

impl ConnectionLifecycleStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Tested => "tested",
            Self::Active => "active",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionLifecycleAction {
    SaveDraft,
    TestSuccess,
    TestFailure,
    Activate,
}

pub fn normalize_status(raw: &str) -> ConnectionLifecycleStatus {
    match raw {
        "draft" | "untested" => ConnectionLifecycleStatus::Draft,
        "tested" => ConnectionLifecycleStatus::Tested,
        "active" => ConnectionLifecycleStatus::Active,
        "error" => ConnectionLifecycleStatus::Error,
        _ => ConnectionLifecycleStatus::Draft,
    }
}

pub fn determine_next_status(
    current_status: &str,
    last_test_status: Option<&str>,
    action: ConnectionLifecycleAction,
) -> Result<ConnectionLifecycleStatus, String> {
    let current = normalize_status(current_status);
    match action {
        ConnectionLifecycleAction::SaveDraft => Ok(ConnectionLifecycleStatus::Draft),
        ConnectionLifecycleAction::TestSuccess => Ok(ConnectionLifecycleStatus::Tested),
        ConnectionLifecycleAction::TestFailure => Ok(ConnectionLifecycleStatus::Error),
        ConnectionLifecycleAction::Activate => {
            if matches!(current, ConnectionLifecycleStatus::Tested | ConnectionLifecycleStatus::Active)
                && last_test_status == Some("success")
            {
                Ok(ConnectionLifecycleStatus::Active)
            } else {
                Err("connection must pass a successful test before activation".to_string())
            }
        }
    }
}

pub fn sanitize_config_for_response(config: &serde_json::Value) -> serde_json::Value {
    let mut cleaned = config.clone();
    if let Some(obj) = cleaned.as_object_mut() {
        obj.remove("_environment");
    }
    cleaned
}

fn field_as_non_empty_string<'a>(obj: &'a serde_json::Map<String, serde_json::Value>, field: &str) -> Option<&'a str> {
    obj.get(field).and_then(|v| v.as_str()).map(str::trim).filter(|v| !v.is_empty())
}

fn is_redacted_placeholder(value: &str) -> bool {
    let trimmed = value.trim();
    let lower = trimmed.to_ascii_lowercase();
    trimmed == "********"
        || trimmed == "******"
        || lower == "redacted"
        || lower == "<redacted>"
        || lower == "masked"
}

pub fn validate_and_normalize_credentials(
    auth_type: &AuthType,
    credentials: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let obj = credentials
        .as_object()
        .ok_or("credentials must be a JSON object")?;

    match auth_type {
        AuthType::Credential => {
            let username = field_as_non_empty_string(obj, "username")
                .ok_or("credentials.username is required for credential auth")?;
            let password = field_as_non_empty_string(obj, "password")
                .ok_or("credentials.password is required for credential auth")?;
            if is_redacted_placeholder(password) {
                return Err("credentials.password must be replaced with a new secret value".to_string());
            }
            Ok(serde_json::json!({ "username": username, "password": password }))
        }
        AuthType::Oauth => {
            let access_token = field_as_non_empty_string(obj, "access_token")
                .ok_or("credentials.access_token is required for oauth auth")?;
            if is_redacted_placeholder(access_token) {
                return Err("credentials.access_token must be replaced with a new token value".to_string());
            }
            Ok(serde_json::json!({ "access_token": access_token }))
        }
        AuthType::ApiKey => {
            let api_key = field_as_non_empty_string(obj, "api_key")
                .ok_or("credentials.api_key is required for api_key auth")?;
            if is_redacted_placeholder(api_key) {
                return Err("credentials.api_key must be replaced with a new key value".to_string());
            }
            Ok(serde_json::json!({ "api_key": api_key }))
        }
    }
}

pub fn has_credentials(credentials: &serde_json::Value) -> bool {
    credentials.as_object().map(|obj| !obj.is_empty()).unwrap_or(false)
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

    #[test]
    fn connector_lifecycle_blocks_activate_without_successful_test() {
        let err = determine_next_status("draft", Some("failure"), ConnectionLifecycleAction::Activate)
            .unwrap_err();
        assert!(err.contains("successful test"));
    }

    #[test]
    fn connector_lifecycle_allows_activate_after_successful_test() {
        let next = determine_next_status("tested", Some("success"), ConnectionLifecycleAction::Activate)
            .unwrap();
        assert_eq!(next, ConnectionLifecycleStatus::Active);
    }

    #[test]
    fn secret_handling_requires_replace_on_masked_values() {
        let err = validate_and_normalize_credentials(
            &AuthType::ApiKey,
            &serde_json::json!({"api_key": "********"}),
        )
        .unwrap_err();
        assert!(err.contains("replaced"));
    }

    #[test]
    fn secret_handling_normalizes_credential_payload() {
        let normalized = validate_and_normalize_credentials(
            &AuthType::Credential,
            &serde_json::json!({"username": "user", "password": "topsecret", "extra": "ignored"}),
        )
        .unwrap();
        assert_eq!(normalized, serde_json::json!({"username": "user", "password": "topsecret"}));
    }

    #[test]
    fn sanitize_config_removes_environment_from_responses() {
        let sanitized = sanitize_config_for_response(&serde_json::json!({"host": "db", "_environment": "dev"}));
        assert_eq!(sanitized, serde_json::json!({"host": "db"}));
    }
}
