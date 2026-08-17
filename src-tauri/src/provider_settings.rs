use crate::database;
use serde::{Deserialize, Serialize};
use std::sync::{OnceLock, RwLock};
use tauri::AppHandle;

const SETTINGS_FILENAME: &str = "paperstack-provider-settings.json";
const OPENALEX_ENV: &str = "OPENALEX_API_KEY";
const SEMANTIC_SCHOLAR_ENV: &str = "SEMANTIC_SCHOLAR_API_KEY";
const CROSSREF_ENV: &str = "CROSSREF_MAILTO";

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProviderOverrides {
    openalex_api_key: Option<String>,
    semantic_scholar_api_key: Option<String>,
    crossref_mailto: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProviderSettingsUpdate {
    openalex_api_key: Option<String>,
    semantic_scholar_api_key: Option<String>,
    crossref_mailto: Option<String>,
    #[serde(default)]
    clear_openalex_api_key: bool,
    #[serde(default)]
    clear_semantic_scholar_api_key: bool,
    #[serde(default)]
    clear_crossref_mailto: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProviderSettingStatus {
    has_override: bool,
    has_environment_value: bool,
    environment_variable: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProviderSettingsStatus {
    openalex: ProviderSettingStatus,
    semantic_scholar: ProviderSettingStatus,
    crossref: ProviderSettingStatus,
}

fn overrides() -> &'static RwLock<ProviderOverrides> {
    static OVERRIDES: OnceLock<RwLock<ProviderOverrides>> = OnceLock::new();
    OVERRIDES.get_or_init(|| RwLock::new(ProviderOverrides::default()))
}

pub(crate) fn initialize(app: &AppHandle) -> Result<(), String> {
    let path = database::app_data_directory(app)?.join(SETTINGS_FILENAME);
    let loaded = match std::fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str::<ProviderOverrides>(&contents)
            .map_err(|error| format!("Could not read provider settings: {error}"))?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => ProviderOverrides::default(),
        Err(error) => return Err(format!("Could not open provider settings: {error}")),
    };
    *overrides()
        .write()
        .map_err(|_| "Could not initialize provider settings".to_owned())? = sanitize(loaded);
    Ok(())
}

#[tauri::command]
pub(crate) fn get_provider_settings() -> Result<ProviderSettingsStatus, String> {
    status()
}

#[tauri::command]
pub(crate) fn save_provider_settings(
    app: AppHandle,
    settings: ProviderSettingsUpdate,
) -> Result<ProviderSettingsStatus, String> {
    let mut updated = overrides()
        .read()
        .map_err(|_| "Could not read provider settings".to_owned())?
        .clone();

    apply_update(
        &mut updated.openalex_api_key,
        settings.openalex_api_key,
        settings.clear_openalex_api_key,
    );
    apply_update(
        &mut updated.semantic_scholar_api_key,
        settings.semantic_scholar_api_key,
        settings.clear_semantic_scholar_api_key,
    );
    apply_update(
        &mut updated.crossref_mailto,
        settings.crossref_mailto,
        settings.clear_crossref_mailto,
    );
    updated = sanitize(updated);

    if let Some(mailto) = updated.crossref_mailto.as_deref() {
        if !mailto.contains('@') {
            return Err("Crossref contact email must be a valid email address".to_owned());
        }
    }

    let path = database::app_data_directory(&app)?.join(SETTINGS_FILENAME);
    let contents = serde_json::to_string_pretty(&updated)
        .map_err(|error| format!("Could not encode provider settings: {error}"))?;
    std::fs::write(&path, contents)
        .map_err(|error| format!("Could not save provider settings: {error}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
            .map_err(|error| format!("Could not protect provider settings: {error}"))?;
    }
    *overrides()
        .write()
        .map_err(|_| "Could not update provider settings".to_owned())? = updated;
    status()
}

pub(crate) fn openalex_api_key_candidates() -> Vec<Option<String>> {
    credential_candidates(
        override_value(|settings| &settings.openalex_api_key),
        environment_value(OPENALEX_ENV),
    )
}

pub(crate) fn semantic_scholar_api_key_candidates() -> Vec<Option<String>> {
    credential_candidates(
        override_value(|settings| &settings.semantic_scholar_api_key),
        environment_value(SEMANTIC_SCHOLAR_ENV),
    )
}

pub(crate) fn crossref_mailto() -> Option<String> {
    override_value(|settings| &settings.crossref_mailto).or_else(|| environment_value(CROSSREF_ENV))
}

fn status() -> Result<ProviderSettingsStatus, String> {
    let settings = overrides()
        .read()
        .map_err(|_| "Could not read provider settings".to_owned())?;
    Ok(ProviderSettingsStatus {
        openalex: setting_status(settings.openalex_api_key.is_some(), OPENALEX_ENV),
        semantic_scholar: setting_status(
            settings.semantic_scholar_api_key.is_some(),
            SEMANTIC_SCHOLAR_ENV,
        ),
        crossref: setting_status(settings.crossref_mailto.is_some(), CROSSREF_ENV),
    })
}

fn setting_status(has_override: bool, environment_variable: &'static str) -> ProviderSettingStatus {
    ProviderSettingStatus {
        has_override,
        has_environment_value: environment_value(environment_variable).is_some(),
        environment_variable,
    }
}

fn override_value(select: impl FnOnce(&ProviderOverrides) -> &Option<String>) -> Option<String> {
    overrides()
        .read()
        .ok()
        .and_then(|settings| select(&settings).clone())
}

fn environment_value(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn credential_candidates(
    override_value: Option<String>,
    environment_value: Option<String>,
) -> Vec<Option<String>> {
    let mut candidates = Vec::new();
    if let Some(value) = override_value {
        candidates.push(Some(value));
    }
    if let Some(value) = environment_value {
        if !candidates
            .iter()
            .any(|candidate| candidate.as_ref() == Some(&value))
        {
            candidates.push(Some(value));
        }
    }
    candidates.push(None);
    candidates
}

fn apply_update(target: &mut Option<String>, value: Option<String>, clear: bool) {
    if clear {
        *target = None;
    } else if let Some(value) = value.and_then(clean) {
        *target = Some(value);
    }
}

fn sanitize(settings: ProviderOverrides) -> ProviderOverrides {
    ProviderOverrides {
        openalex_api_key: settings.openalex_api_key.and_then(clean),
        semantic_scholar_api_key: settings.semantic_scholar_api_key.and_then(clean),
        crossref_mailto: settings.crossref_mailto.and_then(clean),
    }
}

fn clean(value: String) -> Option<String> {
    let value = value.trim().to_owned();
    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credentials_prefer_override_then_environment_then_anonymous() {
        assert_eq!(
            credential_candidates(Some("override".to_owned()), Some("environment".to_owned())),
            vec![
                Some("override".to_owned()),
                Some("environment".to_owned()),
                None
            ]
        );
    }

    #[test]
    fn credentials_do_not_retry_the_same_value() {
        assert_eq!(
            credential_candidates(Some("same".to_owned()), Some("same".to_owned())),
            vec![Some("same".to_owned()), None]
        );
    }

    #[test]
    fn blank_updates_do_not_replace_a_saved_value() {
        let mut value = Some("saved".to_owned());
        apply_update(&mut value, Some("   ".to_owned()), false);
        assert_eq!(value.as_deref(), Some("saved"));
        apply_update(&mut value, None, true);
        assert!(value.is_none());
    }
}
