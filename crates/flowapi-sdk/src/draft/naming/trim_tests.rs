//! A method's name without its resource's words, on names the corpus has: GitHub's, Stripe's,
//! Cloudflare's, Twilio's; and the runs that stay.

use super::trim;
use crate::words::words;

fn trimmed(name: &str, group: &str) -> String {
    trim(&words(name), &words(group)).join("_")
}

#[test]
fn a_run_at_the_front_or_in_the_middle_goes() {
    let cases = [
        (
            "actions/get-actions-cache-retention-limit-for-enterprise",
            "actions",
            "get_cache_retention_limit_for_enterprise",
        ),
        ("repos/list-for-org", "repos", "list_for_org"),
        ("PostCustomersCustomerCardsId", "customers", "post_cards_id"),
        (
            "email-security-list-sending-domain-restrictions",
            "email_security_settings",
            "list_sending_domain_restrictions",
        ),
        ("account-user-group-list", "account_user_groups", "list"),
    ];
    for (name, group, want) in cases {
        assert_eq!(trimmed(name, group), want, "{name} in {group}");
    }
}

#[test]
fn a_run_at_the_end_goes_after_the_verb_or_a_connector() {
    let cases = [
        ("ListFlow", "studio_v2_flows", "list"),
        ("list-abuse-reports", "abuse_reports_other", "list"),
        ("users/list-followers-for-user", "users", "list_followers"),
        (
            "create-a-virtual-network",
            "tunnel_virtual_networks",
            "create",
        ),
        ("get-policies", "policy", "get"),
    ];
    for (name, group, want) in cases {
        assert_eq!(trimmed(name, group), want, "{name} in {group}");
    }
}

#[test]
fn a_run_a_word_qualifies_stays() {
    let cases = [
        (
            "users/list-followers-for-authenticated-user",
            "users",
            "list_followers_for_authenticated_user",
        ),
        ("get-fonts-setting", "zone_settings", "get_fonts_setting"),
        (
            "DeleteIncomingPhoneNumberAssignedAddOn",
            "api20100401_assigned_add_on",
            "delete_incoming_phone_number_assigned_add_on",
        ),
    ];
    for (name, group, want) in cases {
        assert_eq!(trimmed(name, group), want, "{name} in {group}");
    }
}

#[test]
fn connectors_alone_are_no_resource_and_a_plural_is_one_noun() {
    assert_eq!(
        trimmed("list-scripts-for-namespace", "workers_for_platforms"),
        "list_scripts_for_namespace"
    );
    // Half of a pair stays: `list or subtitles` would say nothing.
    assert_eq!(
        trimmed("list-captions-or-subtitles", "stream_subtitles_captions"),
        "list_captions_or_subtitles"
    );
    // `address` is no plural of `addres`, and `status` keeps its `s` against `statu`.
    assert_eq!(trimmed("get-address", "addresses"), "get_address");
    assert_eq!(trimmed("get-status", "status"), "get");
    // A name that is its resource alone leaves nothing: the caller keeps a longer name.
    assert!(trim(&words("pets"), &words("pets")).is_empty());
}
