//! A title's words: a piece in mixed case is one word of the package's name and two or more of
//! the client's (as the client is read everywhere), an acronym a word, a version a word, `api`
//! put first before a digit; and the package's name is one npm takes (lower case, URL-safe).
//! Methods whose names meet both take their longer names.

use super::{methods, title};
use crate::words::{join, pascal, words};

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

fn w(names: &[&str]) -> Vec<Vec<String>> {
    names.iter().map(|n| words(n)).collect()
}

#[test]
fn two_names_that_meet_both_take_their_longer_names() {
    // `get_cache` twice: both are raised, and the third, which met nobody, keeps its short one.
    let names = [
        w(&[
            "get_cache",
            "get_actions_cache",
            "actions_get_actions_cache",
        ]),
        w(&["get_cache", "get_cache_usage", "actions_get_cache_usage"]),
        w(&["list_for_org", "repos_list_for_org"]),
    ];
    assert_eq!(
        methods(&names),
        ["get_actions_cache", "get_cache_usage", "list_for_org"]
    );
}

#[test]
fn names_are_raised_a_kind_at_a_time() {
    // GitHub's `GET /users` and `GET /user` are both `list` by the verb; the first's trimmed name
    // is `list` again, the second's `get_authenticated`: neither needs its full `operationId`.
    let names = [
        w(&["list", "list", "list", "users_list"]),
        w(&[
            "list",
            "get_authenticated",
            "get_authenticated",
            "users_get_authenticated",
        ]),
    ];
    assert_eq!(methods(&names), ["list", "get_authenticated"]);
}

#[test]
fn a_name_raised_into_another_raises_that_one_too_and_numbers_end_it() {
    let names = [
        w(&["list", "list_runs"]),
        w(&["list", "get_runs"]),
        w(&["list_runs", "list_runs_for_repo"]),
        w(&["delete"]),
        w(&["delete"]),
    ];
    assert_eq!(
        methods(&names),
        [
            "list_runs",
            "get_runs",
            "list_runs_for_repo",
            "delete",
            "delete_2"
        ]
    );
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
