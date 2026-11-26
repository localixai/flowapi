//! CommonMark made plain: the markup descriptions write (GitHub's links and notes, Stripe's code,
//! Twilio's HTML), and the first sentence a package's description takes.

use super::{inline, paragraphs, sentence};

#[test]
fn inline_markup_is_taken_out_and_its_words_kept() {
    let cases = [
        (
            "Lists the [repositories](https://docs.github.com/rest) of `org`.",
            "Lists the repositories of org.",
        ),
        ("**Note:** a _new_ *field*", "Note: a new field"),
        (
            "keep snake_case_words and 2 * 3",
            "keep snake_case_words and 2 * 3",
        ),
        ("A <b>bold</b> <a href=\"x\">link</a>", "A bold link"),
        ("See <https://example.com>.", "See https://example.com."),
        ("a < b &amp; c &gt; d", "a < b & c > d"),
        ("an ![image](x.png) here", "an image here"),
        ("escaped \\*stars\\* stay", "escaped *stars* stay"),
        ("List the pets */ one page", "List the pets */ one page"),
        ("[unclosed and (more", "[unclosed and (more"),
    ];
    for (text, want) in cases {
        assert_eq!(inline(text), want, "{text}");
    }
}

#[test]
fn blocks_are_paragraphs_and_code_is_left_out() {
    let text = "# Heading\n\nFirst line\nwraps on.\n\n```js\ncode()\n```\n\n- one\n- two\n\n> quoted\n> text\n\n    indented code\n";
    assert_eq!(
        paragraphs(text),
        [
            "Heading",
            "First line wraps on.",
            "- one",
            "- two",
            "quoted text"
        ]
    );
}

#[test]
fn the_first_sentence_is_a_short_one_of_the_first_paragraph() {
    assert_eq!(
        sentence("# Shop\n\nSells pets. And more, e.g. food.").as_deref(),
        Some("Sells pets.")
    );
    assert_eq!(
        sentence("Version 2.0 of the API, for e.g. tests").as_deref(),
        Some("Version 2.0 of the API, for e.g. tests")
    );
    assert_eq!(sentence(&"long ".repeat(60)), None);
    assert_eq!(sentence("- only a list"), None);
    assert_eq!(sentence(""), None);
}
