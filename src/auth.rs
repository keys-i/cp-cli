use crate::{
    domain::{AuthLogout, AuthStatus},
    error::{Error, Result},
};

#[cfg(feature = "leetcode")]
use crate::domain::AuthSource;

#[cfg(feature = "leetcode")]
const KEYRING_SERVICE: &str = "cp-cli";
#[cfg(feature = "leetcode")]
const KEYRING_USER: &str = "leetcode";
#[cfg(feature = "leetcode")]
const STORED_CREDENTIAL_VERSION: u8 = 1;

pub(crate) async fn status() -> Result<AuthStatus> {
    #[cfg(feature = "leetcode")]
    {
        let Some((credentials, source)) = credentials()? else {
            return Ok(AuthStatus {
                configured: false,
                authenticated: false,
                source: None,
            });
        };
        let authenticated = match platform_leetcode::Client::new()?
            .auth_status(&credentials)
            .await
        {
            Ok(authenticated) => authenticated,
            Err(platform_leetcode::Error::Authentication) => false,
            Err(error) => return Err(error.into()),
        };
        Ok(AuthStatus {
            configured: true,
            authenticated,
            source: Some(source),
        })
    }
    #[cfg(not(feature = "leetcode"))]
    {
        Err(Error::LeetCodeDisabled)
    }
}

pub(crate) fn logout() -> Result<AuthLogout> {
    #[cfg(feature = "leetcode")]
    {
        let entry = keyring_entry()?;
        let removed = match entry.delete_credential() {
            Ok(()) => true,
            Err(keyring::Error::NoEntry) => false,
            Err(error) => return Err(Error::CredentialStore(error)),
        };
        Ok(AuthLogout {
            removed,
            environment_present: authentication_environment_present(),
        })
    }
    #[cfg(not(feature = "leetcode"))]
    {
        Err(Error::LeetCodeDisabled)
    }
}

#[cfg(feature = "leetcode")]
pub(crate) fn required_credentials() -> Result<platform_leetcode::Credentials> {
    credentials()?
        .map(|(credentials, _)| credentials)
        .ok_or(Error::AuthenticationRequired)
}

#[cfg(feature = "leetcode")]
fn credentials() -> Result<Option<(platform_leetcode::Credentials, AuthSource)>> {
    if let Some(credentials) = environment_credentials()? {
        return Ok(Some((credentials, AuthSource::Environment)));
    }
    saved_credentials()
        .map(|credentials| credentials.map(|credentials| (credentials, AuthSource::SecureStore)))
}

#[cfg(feature = "leetcode")]
fn environment_credentials() -> Result<Option<platform_leetcode::Credentials>> {
    let session = std::env::var_os("LEETCODE_SESSION");
    let csrf =
        std::env::var_os("LEETCODE_CSRFTOKEN").or_else(|| std::env::var_os("LEETCODE_CSRF_TOKEN"));
    match (session, csrf) {
        (None, None) => Ok(None),
        (Some(session), Some(csrf)) => {
            let session = session
                .into_string()
                .map_err(|_| Error::InvalidAuthenticationEnvironment)?;
            let csrf = csrf
                .into_string()
                .map_err(|_| Error::InvalidAuthenticationEnvironment)?;
            Ok(Some(platform_leetcode::Credentials::new(&session, &csrf)?))
        }
        _ => Err(Error::InvalidAuthenticationEnvironment),
    }
}

#[cfg(feature = "leetcode")]
fn authentication_environment_present() -> bool {
    [
        "LEETCODE_SESSION",
        "LEETCODE_CSRFTOKEN",
        "LEETCODE_CSRF_TOKEN",
    ]
    .iter()
    .any(|name| std::env::var_os(name).is_some())
}

#[cfg(feature = "leetcode")]
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredCredentials {
    version: u8,
    session: String,
    csrf: String,
}

#[cfg(feature = "leetcode")]
fn keyring_entry() -> Result<keyring::Entry> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER).map_err(Error::CredentialStore)
}

#[cfg(feature = "leetcode")]
fn saved_credentials() -> Result<Option<platform_leetcode::Credentials>> {
    let encoded = match keyring_entry()?.get_secret() {
        Ok(encoded) => encoded,
        Err(keyring::Error::NoEntry) => return Ok(None),
        Err(error) => return Err(Error::CredentialStore(error)),
    };
    decode_credentials(&encoded).map(Some)
}

#[cfg(feature = "leetcode")]
fn decode_credentials(encoded: &[u8]) -> Result<platform_leetcode::Credentials> {
    let stored: StoredCredentials =
        serde_json::from_slice(encoded).map_err(|_| Error::InvalidStoredCredentials)?;
    if stored.version != STORED_CREDENTIAL_VERSION {
        return Err(Error::InvalidStoredCredentials);
    }
    platform_leetcode::Credentials::new(&stored.session, &stored.csrf)
        .map_err(|_| Error::InvalidStoredCredentials)
}

#[cfg(all(test, feature = "leetcode"))]
mod tests {
    use super::*;

    #[test]
    fn stored_credential_contract() {
        assert!(
            decode_credentials(br#"{"version":1,"session":"session-value","csrf":"csrf-value"}"#)
                .is_ok()
        );
        assert!(matches!(
            decode_credentials(br#"{"version":2,"session":"session","csrf":"csrf"}"#),
            Err(Error::InvalidStoredCredentials)
        ));
        assert!(matches!(
            decode_credentials(br#"{"version":1,"session":"bad value","csrf":"csrf"}"#),
            Err(Error::InvalidStoredCredentials)
        ));
    }
}
