use std::time::Instant;

use crate::connections::{AuthType, ConnectorType};
#[path = "specs.rs"]
mod specs;

#[derive(Debug, Clone)]
pub struct ConnectorHealthCheckResult {
    pub healthy: bool,
    pub message: String,
    pub details: serde_json::Value,
    pub latency_ms: u128,
}

#[derive(Clone)]
struct RegisteredHealthCheck {
    connector_type: ConnectorType,
    priority: u8,
    name: &'static str,
    checker: fn(&serde_json::Value, &serde_json::Value, &AuthType) -> Result<String, String>,
}

const PRIORITIZED_CONNECTOR_HEALTH_CHECKS: &[RegisteredHealthCheck] = &[
    RegisteredHealthCheck {
        connector_type: ConnectorType::Snowflake,
        priority: 100,
        name: "snowflake_health_check",
        checker: snowflake_health_check,
    },
    RegisteredHealthCheck {
        connector_type: ConnectorType::Salesforce,
        priority: 95,
        name: "salesforce_health_check",
        checker: salesforce_health_check,
    },
    RegisteredHealthCheck {
        connector_type: ConnectorType::Oracle,
        priority: 90,
        name: "oracle_health_check",
        checker: oracle_health_check,
    },
    RegisteredHealthCheck {
        connector_type: ConnectorType::Postgresql,
        priority: 80,
        name: "postgresql_health_check",
        checker: relational_health_check,
    },
    RegisteredHealthCheck {
        connector_type: ConnectorType::Mysql,
        priority: 75,
        name: "mysql_health_check",
        checker: relational_health_check,
    },
    RegisteredHealthCheck {
        connector_type: ConnectorType::RestApi,
        priority: 70,
        name: "rest_api_health_check",
        checker: rest_api_health_check,
    },
];

fn select_health_check(connector_type: &ConnectorType) -> Option<RegisteredHealthCheck> {
    PRIORITIZED_CONNECTOR_HEALTH_CHECKS
        .iter()
        .filter(|entry| entry.connector_type == *connector_type)
        .max_by_key(|entry| entry.priority)
        .cloned()
}

pub fn health_check(
    connector_type: &ConnectorType,
    auth_type: &AuthType,
    config: &serde_json::Value,
    credentials: &serde_json::Value,
) -> ConnectorHealthCheckResult {
    let start = Instant::now();
    let selected = select_health_check(connector_type);

    let (healthy, message, checker_name, priority) = match selected {
        Some(entry) => match (entry.checker)(config, credentials, auth_type) {
            Ok(message) => (true, message, entry.name, entry.priority),
            Err(message) => (false, message, entry.name, entry.priority),
        },
        None => (
            false,
            format!("No health check registered for connector type '{}'.", connector_type),
            "none",
            0,
        ),
    };

    ConnectorHealthCheckResult {
        healthy,
        message,
        details: serde_json::json!({
            "connector_type": connector_type.to_string(),
            "auth_type": auth_type.to_string(),
            "registry_checker": checker_name,
            "registry_priority": priority,
        }),
        latency_ms: start.elapsed().as_millis(),
    }
}

fn ensure_non_empty_string(obj: &serde_json::Map<String, serde_json::Value>, field: &str) -> Result<(), String> {
    let value = obj
        .get(field)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or_default();
    if value.is_empty() {
        Err(format!("{} must be a non-empty string", field))
    } else {
        Ok(())
    }
}

fn ensure_port(obj: &serde_json::Map<String, serde_json::Value>) -> Result<(), String> {
    let port = obj.get("port").and_then(|v| v.as_u64()).unwrap_or(0);
    if port == 0 || port > 65535 {
        Err("port must be an integer between 1 and 65535".to_string())
    } else {
        Ok(())
    }
}

fn ensure_credential_value(credentials: &serde_json::Map<String, serde_json::Value>, key: &str) -> Result<(), String> {
    let value = credentials
        .get(key)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or_default();
    if value.is_empty() {
        Err(format!("credentials.{} is required", key))
    } else {
        Ok(())
    }
}

fn ensure_https_url(value: &str, field: &str) -> Result<(), String> {
    if value.starts_with("https://") {
        Ok(())
    } else {
        Err(format!("{} must use https://", field))
    }
}

fn relational_health_check(
    config: &serde_json::Value,
    credentials: &serde_json::Value,
    auth_type: &AuthType,
) -> Result<String, String> {
    if !matches!(auth_type, AuthType::Credential) {
        return Err("relational connectors require credential auth".to_string());
    }

    let config_obj = config.as_object().ok_or("config must be an object")?;
    ensure_non_empty_string(config_obj, "host")?;
    ensure_non_empty_string(config_obj, "database")?;
    ensure_port(config_obj)?;

    let creds = credentials.as_object().ok_or("credentials must be an object")?;
    ensure_credential_value(creds, "username")?;
    ensure_credential_value(creds, "password")?;

    Ok("Host/port/database and credential fields validated".to_string())
}

fn oracle_health_check(
    config: &serde_json::Value,
    credentials: &serde_json::Value,
    auth_type: &AuthType,
) -> Result<String, String> {
    relational_health_check(config, credentials, auth_type)?;
    let config_obj = config.as_object().ok_or("config must be an object")?;
    ensure_non_empty_string(config_obj, "service_name")?;
    Ok("Oracle service_name and base relational fields validated".to_string())
}

fn snowflake_health_check(
    config: &serde_json::Value,
    credentials: &serde_json::Value,
    auth_type: &AuthType,
) -> Result<String, String> {
    if !matches!(auth_type, AuthType::Credential) {
        return Err("snowflake requires credential auth".to_string());
    }

    let config_obj = config.as_object().ok_or("config must be an object")?;
    ensure_non_empty_string(config_obj, "account")?;
    ensure_non_empty_string(config_obj, "warehouse")?;
    ensure_non_empty_string(config_obj, "database")?;

    let creds = credentials.as_object().ok_or("credentials must be an object")?;
    ensure_credential_value(creds, "username")?;
    ensure_credential_value(creds, "password")?;

    Ok("Snowflake account/warehouse/database and credentials validated".to_string())
}

fn salesforce_health_check(
    config: &serde_json::Value,
    credentials: &serde_json::Value,
    auth_type: &AuthType,
) -> Result<String, String> {
    if !matches!(auth_type, AuthType::Oauth) {
        return Err("salesforce requires oauth auth".to_string());
    }

    let config_obj = config.as_object().ok_or("config must be an object")?;
    let instance_url = config_obj
        .get("instance_url")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or_default();
    if instance_url.is_empty() {
        return Err("instance_url must be a non-empty string".to_string());
    }
    ensure_https_url(instance_url, "instance_url")?;

    let creds = credentials.as_object().ok_or("credentials must be an object")?;
    ensure_credential_value(creds, "access_token")?;

    Ok("Salesforce URL and oauth token validated".to_string())
}

fn rest_api_health_check(
    config: &serde_json::Value,
    credentials: &serde_json::Value,
    auth_type: &AuthType,
) -> Result<String, String> {
    if !matches!(auth_type, AuthType::ApiKey) {
        return Err("rest_api requires api_key auth".to_string());
    }

    let config_obj = config.as_object().ok_or("config must be an object")?;
    let base_url = config_obj
        .get("base_url")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or_default();
    if base_url.is_empty() {
        return Err("base_url must be a non-empty string".to_string());
    }
    if !(base_url.starts_with("http://") || base_url.starts_with("https://")) {
        return Err("base_url must start with http:// or https://".to_string());
    }

    let creds = credentials.as_object().ok_or("credentials must be an object")?;
    ensure_credential_value(creds, "api_key")?;

    Ok("REST API URL and api_key validated".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connector_lifecycle_registry_uses_prioritized_health_check() {
        let result = health_check(
            &ConnectorType::Snowflake,
            &AuthType::Credential,
            &serde_json::json!({"account": "acct", "warehouse": "wh", "database": "db"}),
            &serde_json::json!({"username": "u", "password": "p"}),
        );

        assert!(result.healthy);
        assert_eq!(result.details["registry_checker"], "snowflake_health_check");
    }

    #[test]
    fn connector_lifecycle_registry_fails_with_missing_credentials() {
        let result = health_check(
            &ConnectorType::Postgresql,
            &AuthType::Credential,
            &serde_json::json!({"host": "db.local", "port": 5432, "database": "analytics"}),
            &serde_json::json!({"username": "u"}),
        );

        assert!(!result.healthy);
        assert!(result.message.contains("credentials.password"));
    }

    #[test]
    fn connector_spec_contract_registry_ids_match_spec_ids() {
        for entry in PRIORITIZED_CONNECTOR_HEALTH_CHECKS {
            let spec = specs::get_connector_spec(&entry.connector_type);
            assert_eq!(spec.connector_id, entry.connector_type.to_string());
        }
    }

    #[test]
    fn connector_spec_contract_registry_auth_modes_are_supported() {
        let cases = [
            (
                ConnectorType::Postgresql,
                AuthType::Credential,
                serde_json::json!({"host": "db.local", "port": 5432, "database": "analytics"}),
            ),
            (
                ConnectorType::Mysql,
                AuthType::Credential,
                serde_json::json!({"host": "db.local", "port": 3306, "database": "analytics"}),
            ),
            (
                ConnectorType::Snowflake,
                AuthType::Credential,
                serde_json::json!({"account": "acct", "warehouse": "wh", "database": "db"}),
            ),
            (
                ConnectorType::Oracle,
                AuthType::Credential,
                serde_json::json!({"host": "db.local", "port": 1521, "service_name": "svc"}),
            ),
            (
                ConnectorType::Salesforce,
                AuthType::Oauth,
                serde_json::json!({"instance_url": "https://example.my.salesforce.com"}),
            ),
            (
                ConnectorType::RestApi,
                AuthType::ApiKey,
                serde_json::json!({"base_url": "https://api.example.com"}),
            ),
        ];

        for (connector_type, auth_type, config) in cases {
            let result = specs::validate_connector_spec(&connector_type, &auth_type, &config);
            assert!(result.is_ok(), "expected spec validation to pass");
        }
    }
}
