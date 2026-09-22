use std::io::IsTerminal;

use console::Term;

use crate::{
    domain::ToolAction,
    error::{Error, Result},
};

#[cfg(feature = "codeforces")]
const KEYRING_SERVICE: &str = "cp-cli";
#[cfg(feature = "codeforces")]
const CODEFORCES_KEYRING_USER: &str = "codeforces";
#[cfg(feature = "codeforces")]
const CODEFORCES_CREDENTIAL_VERSION: u8 = 1;

pub(crate) async fn codeforces_login(
    progress: impl FnMut(usize, Option<u64>),
) -> Result<ToolAction> {
    #[cfg(feature = "codeforces")]
    {
        require_terminal()?;
        let terminal = Term::stderr();
        terminal.write_line("Enter the API key and secret issued for your Codeforces account.")?;
        terminal.write_str("Codeforces API key: ")?;
        let key = terminal.read_line()?;
        terminal.write_str("Codeforces API secret: ")?;
        let secret = terminal.read_secure_line()?;
        let credentials = platform_codeforces::ApiCredentials::new(key.trim(), secret.trim())?;
        platform_codeforces::Client::new()?
            .authenticated_friends(&credentials, progress)
            .await?;
        save_codeforces_credentials(key.trim(), secret.trim())?;
        Ok(action(
            "Codeforces",
            "Credentials saved",
            "Codeforces accepted the signed API request. The key and secret are stored in the operating system credential store.",
        ))
    }
    #[cfg(not(feature = "codeforces"))]
    {
        let _ = progress;
        Err(Error::CodeforcesDisabled)
    }
}

pub(crate) async fn codeforces_status(
    progress: impl FnMut(usize, Option<u64>),
) -> Result<ToolAction> {
    #[cfg(feature = "codeforces")]
    {
        let Some((credentials, source)) = codeforces_credentials()? else {
            return Ok(action(
                "Codeforces",
                "Not configured",
                "Run `cp-cli --platform codeforces auth login`; it accepts an issued API key and hides the secret while you paste it. Automation can set CODEFORCES_API_KEY and CODEFORCES_API_SECRET.",
            ));
        };
        match platform_codeforces::Client::new()?
            .authenticated_friends(&credentials, progress)
            .await
        {
            Ok(_) => Ok(action(
                "Codeforces",
                "Ready",
                &format!("Codeforces accepted the {source} API credentials."),
            )),
            Err(platform_codeforces::Error::ApiFailure) => Ok(action(
                "Codeforces",
                "Rejected",
                &format!("Codeforces rejected the {source} API credentials."),
            )),
            Err(error) => Err(error.into()),
        }
    }
    #[cfg(not(feature = "codeforces"))]
    {
        let _ = progress;
        Err(Error::CodeforcesDisabled)
    }
}

pub(crate) fn codeforces_logout() -> Result<ToolAction> {
    #[cfg(feature = "codeforces")]
    {
        let removed = match codeforces_keyring_entry()?.delete_credential() {
            Ok(()) => true,
            Err(keyring::Error::NoEntry) => false,
            Err(error) => return Err(Error::CredentialStore(error)),
        };
        let environment_present = codeforces_environment_present();
        let detail = match (removed, environment_present) {
            (true, true) => {
                "Saved credentials were removed. CODEFORCES_API_KEY and CODEFORCES_API_SECRET remain active in this shell."
            }
            (true, false) => {
                "Saved credentials were removed from the operating system credential store."
            }
            (false, true) => {
                "No saved credentials were found. CODEFORCES_API_KEY and CODEFORCES_API_SECRET remain active in this shell."
            }
            (false, false) => "No saved Codeforces credentials were found.",
        };
        Ok(action(
            "Codeforces",
            if removed {
                "Credentials removed"
            } else {
                "Nothing to remove"
            },
            detail,
        ))
    }
    #[cfg(not(feature = "codeforces"))]
    {
        Err(Error::CodeforcesDisabled)
    }
}

pub(crate) fn exercism_login() -> Result<ToolAction> {
    require_terminal()?;
    let terminal = Term::stderr();
    terminal.write_line("Enter the API token issued for your Exercism account.")?;
    terminal.write_str("Exercism API token: ")?;
    let token = terminal.read_secure_line()?;
    crate::exercism_cli::Client::new().configure_token(token.trim())?;
    Ok(action(
        "Exercism",
        "Token configured",
        "The official Exercism CLI saved the token in its own configuration.",
    ))
}

pub(crate) fn exercism_status() -> Result<ToolAction> {
    crate::exercism_cli::Client::new().show_configuration()?;
    Ok(action(
        "Exercism",
        "Configuration readable",
        "The official Exercism CLI can read its configuration. Exercism validates the token on the next authenticated command.",
    ))
}

fn require_terminal() -> Result<()> {
    if std::io::stderr().is_terminal() {
        Ok(())
    } else {
        Err(Error::InteractiveLoginRequired)
    }
}

fn action(platform: &str, action: &str, detail: &str) -> ToolAction {
    ToolAction {
        platform: platform.into(),
        action: action.into(),
        detail: detail.into(),
    }
}

#[cfg(feature = "codeforces")]
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct StoredCodeforcesCredentials {
    version: u8,
    key: String,
    secret: String,
}

#[cfg(feature = "codeforces")]
fn codeforces_keyring_entry() -> Result<keyring::Entry> {
    keyring::Entry::new(KEYRING_SERVICE, CODEFORCES_KEYRING_USER).map_err(Error::CredentialStore)
}

#[cfg(feature = "codeforces")]
fn save_codeforces_credentials(key: &str, secret: &str) -> Result<()> {
    let encoded = serde_json::to_vec(&StoredCodeforcesCredentials {
        version: CODEFORCES_CREDENTIAL_VERSION,
        key: key.to_owned(),
        secret: secret.to_owned(),
    })
    .map_err(|_| Error::InvalidCodeforcesStoredCredentials)?;
    codeforces_keyring_entry()?
        .set_secret(&encoded)
        .map_err(Error::CredentialStore)
}

#[cfg(feature = "codeforces")]
fn codeforces_credentials() -> Result<Option<(platform_codeforces::ApiCredentials, &'static str)>> {
    if let Some(credentials) = codeforces_environment_credentials()? {
        return Ok(Some((credentials, "environment")));
    }
    let encoded = match codeforces_keyring_entry()?.get_secret() {
        Ok(encoded) => encoded,
        Err(keyring::Error::NoEntry) => return Ok(None),
        Err(error) => return Err(Error::CredentialStore(error)),
    };
    decode_codeforces_credentials(&encoded).map(|credentials| Some((credentials, "saved")))
}

#[cfg(feature = "codeforces")]
fn codeforces_environment_credentials() -> Result<Option<platform_codeforces::ApiCredentials>> {
    let key = std::env::var_os("CODEFORCES_API_KEY");
    let secret = std::env::var_os("CODEFORCES_API_SECRET");
    match (key, secret) {
        (None, None) => Ok(None),
        (Some(key), Some(secret)) => {
            let key = key
                .into_string()
                .map_err(|_| Error::InvalidCodeforcesAuthenticationEnvironment)?;
            let secret = secret
                .into_string()
                .map_err(|_| Error::InvalidCodeforcesAuthenticationEnvironment)?;
            platform_codeforces::ApiCredentials::new(key, secret)
                .map(Some)
                .map_err(Into::into)
        }
        _ => Err(Error::InvalidCodeforcesAuthenticationEnvironment),
    }
}

#[cfg(feature = "codeforces")]
fn codeforces_environment_present() -> bool {
    ["CODEFORCES_API_KEY", "CODEFORCES_API_SECRET"]
        .iter()
        .any(|name| std::env::var_os(name).is_some())
}

#[cfg(feature = "codeforces")]
fn decode_codeforces_credentials(encoded: &[u8]) -> Result<platform_codeforces::ApiCredentials> {
    let stored: StoredCodeforcesCredentials =
        serde_json::from_slice(encoded).map_err(|_| Error::InvalidCodeforcesStoredCredentials)?;
    if stored.version != CODEFORCES_CREDENTIAL_VERSION {
        return Err(Error::InvalidCodeforcesStoredCredentials);
    }
    platform_codeforces::ApiCredentials::new(stored.key, stored.secret)
        .map_err(|_| Error::InvalidCodeforcesStoredCredentials)
}

#[cfg(all(test, feature = "codeforces"))]
mod tests {
    use super::*;

    #[test]
    fn stored_codeforces_credentials_are_versioned_and_validated() -> Result<()> {
        let encoded = serde_json::to_vec(&StoredCodeforcesCredentials {
            version: CODEFORCES_CREDENTIAL_VERSION,
            key: "api-key".to_owned(),
            secret: "api-secret".to_owned(),
        })?;
        assert!(decode_codeforces_credentials(&encoded).is_ok());
        assert!(
            decode_codeforces_credentials(
                br#"{"version":2,"key":"api-key","secret":"api-secret"}"#
            )
            .is_err()
        );
        Ok(())
    }
}
