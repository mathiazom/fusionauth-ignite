mod config;

use std::env;
use fusionauth_rust_client::apis::configuration::{ApiKey, Configuration};
use fusionauth_rust_client::apis::default_api::{create_application_with_id, patch_system_configuration_with_id, patch_tenant_with_id, register, retrieve_application, retrieve_application_with_id, retrieve_user, update_application_with_id};
use fusionauth_rust_client::models::{Application, ApplicationRequest, CorsConfiguration, JwtConfiguration, LoginConfiguration, OAuth2Configuration, RefreshTokenExpirationPolicy, RefreshTokenSlidingWindowConfiguration, RegistrationRequest, SystemConfiguration, SystemConfigurationRequest, Tenant, TenantRequest, User, UserRegistration};
use fusionauth_rust_client::models::HttpMethod::{Get, Post};
use serde_json::json;
use crate::config::{read_config, Config};

const FUSIONAUTH_DEFAULT_APPLICATION_NAME: &str = "FusionAuth";
const CONFIG_PATH: &str = "config.kdl";

#[tokio::main]
async fn main() {
    let Ok(default_tenant_id) = env::var("FUSIONAUTH_DEFAULT_TENANT_ID") else {
        println!("Missing FUSIONAUTH_DEFAULT_TENANT_ID");
        return;
    };  
    let config = read_config(CONFIG_PATH)
        .expect("");
    println!("default_tenant_id: {:?}", default_tenant_id);
    println!("Configuration: {:#?}", config);
    let client_config = Configuration {
        base_path: config.base_path.clone(),
        api_key: Some(ApiKey { prefix: None, key: config.api_key.clone() }),
        ..Default::default()
    };
    init_system(&client_config, &config).await;
    init_admin_user(&client_config, &config).await;
    // TODO: init theme
    // TODO: init email templates
    init_tenant(&client_config, &config, &default_tenant_id).await;
    init_application(&client_config, &config).await;
}

async fn init_system(client_config: &Configuration, config: &Config) {
    patch_system_configuration_with_id(
        &client_config,
        Some(SystemConfigurationRequest {
            system_configuration: Some(SystemConfiguration {
                audit_log_configuration: None,
                cors_configuration: Some(CorsConfiguration {
                    allow_credentials: Some(true),
                    allowed_methods: Some(vec![Get, Post]),
                    allowed_origins: Some(vec![config.external_url.clone()]),
                    enabled: Some(true),
                    ..CorsConfiguration::new()
                }.into()),
                ..SystemConfiguration::new()
            }.into()),
        }),
    ).await.expect("Failed to initialize system configuration");
}

async fn init_admin_user(client_config: &Configuration, config: &Config) {
    if retrieve_user(client_config, Some(&config.admin.username), None, None, None, None, None).await.is_ok() {
        println!("Admin user already exists");
        return;
    }
    let apps_res = retrieve_application(client_config, None, None).await;
    let Ok(Some(apps)) = apps_res.map(|r| { r.applications }) else {
        println!("Failed to retrieve applications");
        return;
    };
    let default_app = apps
        .into_iter()
        .find(|app| {
            app.name
                .as_ref()
                .map_or(false, |name| name == FUSIONAUTH_DEFAULT_APPLICATION_NAME)
        });
    let Some(default_app) = default_app else {
        println!("Failed to retrieve default app");
        return;
    };
    println!("Using default application: {} ({})", default_app.name.clone().unwrap(), default_app.id.clone().unwrap());
    register(client_config, None, Some(RegistrationRequest {
        registration: Some(UserRegistration {
            application_id: default_app.id,
            roles: Some(vec!["admin".into()]),
            ..UserRegistration::new()
        }.into()),
        user: Some(User {
            username: Some(config.admin.username.clone()),
            password: Some(config.admin.password.clone()),
            ..User::new()
        }.into()),
        skip_verification: Some(true),
        ..RegistrationRequest::new()
    })).await.expect("Failed to register admin user to default application");
    println!("Successfully registered admin user '{}'", config.admin.username);
}

async fn init_tenant(client_config: &Configuration, config: &Config, tenant_id: &str) {
    patch_tenant_with_id(client_config, tenant_id, None, Some(TenantRequest {
        tenant: Some(Tenant {
            issuer: Some(config.issuer.clone()),
            // TODO: email configs
            ..Default::default()
        }.into()),
        ..Default::default()
    })).await.expect("Failed to initialize default tenant");
    println!("Successfully patched default tenant '{}'", tenant_id);
}

async fn init_application(client_config: &Configuration, config: &Config) {
    let application_id = &config.application.id;

    let application_request = Some(ApplicationRequest {
        application: Some(Application {
            name: Some(config.application.name.clone()),
            jwt_configuration: Some(JwtConfiguration {
                enabled: Some(true),
                refresh_token_expiration_policy: Some(RefreshTokenExpirationPolicy::SlidingWindowWithMaximumLifetime),
                refresh_token_sliding_window_configuration: Some(RefreshTokenSlidingWindowConfiguration {
                    maximum_time_to_live_in_minutes: Some(525600),
                }.into()),
                refresh_token_time_to_live_in_minutes: Some(43200),
                time_to_live_in_seconds: Some(3600),
                ..Default::default()
            }.into()),
            oauth_configuration: Some(OAuth2Configuration {
                client_secret: Some(config.application.oauth.client_secret.clone()),
                generate_refresh_tokens: Some(true),
                require_registration: Some(false),
                enabled_grants: Some(vec!(json!("authorization_code"), json!("refresh_token"))),
                authorized_origin_urls: Some(config.application.oauth.authorized_origin_urls.clone()),
                authorized_redirect_urls: Some(config.application.oauth.authorized_redirect_urls.clone()),
                logout_url: Some(config.application.oauth.logout_url.clone()),
                ..Default::default()
            }.into()),
            login_configuration: Some(LoginConfiguration {
                allow_token_refresh: Some(true),
                generate_refresh_tokens: Some(true),
                require_authentication: Some(true),
            }.into()),
            ..Default::default()
        }.into()),
        ..Default::default()
    });
    let result = retrieve_application_with_id(client_config, application_id, None).await;
    if result.is_ok() {
        println!("Successfully retrieved application with id '{}'", application_id);
        update_application_with_id(client_config, application_id, None, None, application_request).await.expect("Failed to update application");
        println!("Successfully updated application with id '{}'", application_id);
        return;
    }
    println!("No existing application with id '{}'", application_id);
    create_application_with_id(client_config, application_id, None, application_request).await.expect("Failed to create application");
    println!("Successfully created application with id '{}'", application_id);
}