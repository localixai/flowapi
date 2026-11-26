//! A title's words: a piece in mixed case is one word of the package's name and two or more of
//! the client's (as the client is read everywhere), an acronym a word, a version a word, `api`
//! put first before a digit; and the package's name is one npm takes (lower case, URL-safe).

use super::title;
use crate::words::{join, pascal};

/// The package's name and the client's class drafted from a title.
fn named(text: &str) -> (String, String) {
    let (prose, client) = title(text);
    (join(&prose, "-"), pascal(&client))
}

#[test]
fn a_word_in_mixed_case_stays_one_word_of_the_package() {
    let cases = [
        (
            "GitHub v3 REST API",
            "github-v3-rest-api",
            "GitHubV3RestApi",
        ),
        ("DigitalOcean API", "digitalocean-api", "DigitalOceanApi"),
        ("OpenAI API", "openai-api", "OpenAiApi"),
        ("PayPal Payments", "paypal-payments", "PayPalPayments"),
        (
            "YouTube Data API v3",
            "youtube-data-api-v3",
            "YouTubeDataApiV3",
        ),
        (
            "Twilio - Ip_messaging",
            "twilio-ip-messaging",
            "TwilioIpMessaging",
        ),
        ("Slack Web API", "slack-web-api", "SlackWebApi"),
        ("HTTPBin", "httpbin", "HttpBin"),
        ("2nd Hand Market", "api-2nd-hand-market", "Api2ndHandMarket"),
        ("", "api", "Api"),
    ];
    for (text, package, client) in cases {
        assert_eq!(
            named(text),
            (package.to_owned(), client.to_owned()),
            "{text}"
        );
    }
}

#[test]
fn the_package_name_is_one_npm_takes() {
    for text in ["Café & Co. API", "  The  API  ", "x/y:z", "Ünïcode API"] {
        let (package, _) = named(text);
        assert!(
            package
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
            "{text}: {package}"
        );
        assert!(
            !package.starts_with('-') && !package.ends_with('-'),
            "{text}: {package}"
        );
    }
}
