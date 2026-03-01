use crate::connections::{AuthType, ConnectorType};

#[derive(Debug, Clone)]
pub struct ConnectorSpec {
    pub connector_id: &'static str,
    pub auth_modes: &'static [AuthType],
    pub required_config_fields: &'static [&'static str],
}

const CREDENTIAL_AUTH_MODES: &[AuthType] = &[AuthType::Credential];
const OAUTH_AUTH_MODES: &[AuthType] = &[AuthType::Oauth];
const API_KEY_AUTH_MODES: &[AuthType] = &[AuthType::ApiKey];

pub fn get_connector_spec(connector_type: &ConnectorType) -> ConnectorSpec {
    match connector_type {
        ConnectorType::Postgresql => ConnectorSpec {
            connector_id: "postgresql",
            auth_modes: CREDENTIAL_AUTH_MODES,
            required_config_fields: &["host", "port", "database"],
        },
        ConnectorType::Mysql => ConnectorSpec {
            connector_id: "mysql",
            auth_modes: CREDENTIAL_AUTH_MODES,
            required_config_fields: &["host", "port", "database"],
        },
        ConnectorType::Snowflake => ConnectorSpec {
            connector_id: "snowflake",
            auth_modes: CREDENTIAL_AUTH_MODES,
            required_config_fields: &["account", "warehouse", "database"],
        },
        ConnectorType::Oracle => ConnectorSpec {
            connector_id: "oracle",
            auth_modes: CREDENTIAL_AUTH_MODES,
            required_config_fields: &["host", "port", "service_name"],
        },
        ConnectorType::Salesforce => ConnectorSpec {
            connector_id: "salesforce",
            auth_modes: OAUTH_AUTH_MODES,
            required_config_fields: &["instance_url"],
        },
        ConnectorType::RestApi => ConnectorSpec {
            connector_id: "rest_api",
            auth_modes: API_KEY_AUTH_MODES,
            required_config_fields: &["base_url"],
        },
    }
}

pub fn validate_connector_spec(
    connector_type: &ConnectorType,
    auth_type: &AuthType,
    config: &serde_json::Value,
) -> Result<(), String> {
    let spec = get_connector_spec(connector_type);
    if !spec.auth_modes.iter().any(|mode| mode == auth_type) {
        return Err(format!(
            "Invalid auth_type '{}' for connector '{}'. Supported auth modes: {}",
            auth_type,
            spec.connector_id,
            spec.auth_modes
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .join(", ")
        ));
    }

    let obj = config.as_object().ok_or("config must be a JSON object")?;
    let missing: Vec<&str> = spec
        .required_config_fields
        .iter()
        .filter(|field| !obj.contains_key(**field))
        .copied()
        .collect();

    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Missing required config fields for {}: {}",
            spec.connector_id,
            missing.join(", ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connector_spec_contract_includes_expected_connector_ids() {
        let ids = vec![
            get_connector_spec(&ConnectorType::Postgresql).connector_id,
            get_connector_spec(&ConnectorType::Mysql).connector_id,
            get_connector_spec(&ConnectorType::Snowflake).connector_id,
            get_connector_spec(&ConnectorType::Oracle).connector_id,
            get_connector_spec(&ConnectorType::Salesforce).connector_id,
            get_connector_spec(&ConnectorType::RestApi).connector_id,
        ];

        assert_eq!(
            ids,
            vec![
                "postgresql",
                "mysql",
                "snowflake",
                "oracle",
                "salesforce",
                "rest_api"
            ]
        );
    }

    #[test]
    fn connector_spec_contract_rejects_mismatched_auth_type() {
        let err = validate_connector_spec(
            &ConnectorType::Salesforce,
            &AuthType::Credential,
            &serde_json::json!({"instance_url": "https://example.my.salesforce.com"}),
        )
        .unwrap_err();

        assert!(err.contains("Invalid auth_type"));
        assert!(err.contains("oauth"));
    }

    #[test]
    fn connector_spec_contract_detects_missing_required_fields() {
        let err = validate_connector_spec(
            &ConnectorType::Postgresql,
            &AuthType::Credential,
            &serde_json::json!({"host": "localhost"}),
        )
        .unwrap_err();

        assert!(err.contains("port"));
        assert!(err.contains("database"));
    }
}
