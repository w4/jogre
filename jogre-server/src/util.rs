use hmac::{digest::FixedOutput, Hmac, Mac};
use sha3::Sha3_256;
use tower_cookies::{
    cookie::{time::Duration, CookieBuilder, SameSite},
    Cookies,
};
use tracing::warn;

use crate::context::DerivedKeys;

type HmacSha3 = Hmac<Sha3_256>;

const CSRF_TOKEN_COOKIE_NAME: &str = "csrf_token";

#[derive(Copy, Clone)]
pub struct CsrfToken {
    signed: [u8; 32],
    unsigned: u128,
}

impl CsrfToken {
    pub fn new(derived_keys: &DerivedKeys) -> Self {
        let unsigned = rand::random::<u128>();

        let mut hmac = HmacSha3::new_from_slice(&derived_keys.csrf_hmac_key).unwrap();
        hmac.update(&unsigned.to_be_bytes());
        let signed = hmac.finalize_fixed().into();

        Self { signed, unsigned }
    }

    pub fn write_cookie(&self, cookies: &Cookies) {
        cookies.add(
            CookieBuilder::new(CSRF_TOKEN_COOKIE_NAME, hex::encode(self.signed))
                .http_only(true)
                .max_age(Duration::hours(24))
                .same_site(SameSite::Strict)
                // .secure(true) // TODO
                .finish(),
        );
    }

    #[must_use]
    pub fn verify(derived_keys: &DerivedKeys, cookies: &Cookies, form_value: &str) -> bool {
        let Some(cookie) = cookies.get(CSRF_TOKEN_COOKIE_NAME) else {
            warn!("Missing CSRF token");
            return false;
        };

        let form_value = match hex::decode(form_value) {
            Ok(v) => v,
            Err(error) => {
                warn!(?error, "Invalid form CSRF token");
                return false;
            }
        };

        let cookie_token = match hex::decode(cookie.value()) {
            Ok(v) => v,
            Err(error) => {
                warn!(?error, "Invalid cookie CSRF token");
                return false;
            }
        };

        let mut hmac = HmacSha3::new_from_slice(&derived_keys.csrf_hmac_key).unwrap();
        hmac.update(&form_value);

        match hmac.verify_slice(&cookie_token) {
            Ok(()) => true,
            Err(error) => {
                warn!(?error, "CSRF form value and cookie mismatch");
                false
            }
        }
    }

    pub fn form_value(&self) -> String {
        hex::encode(self.unsigned.to_be_bytes())
    }
}
