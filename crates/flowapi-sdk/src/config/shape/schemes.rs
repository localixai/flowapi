//! The pagination schemes of the format, each `type` with its members: what each request
//! parameter and each path in the response is for. Part of `shape.rs`, by which the checker, its
//! reader and the JSON Schema all go.

use super::{ANY, Field, START, Shape, TEXT, def, f, req};

pub(crate) const CURSOR_REQUEST: &[Field] = &[
    req(
        "cursor",
        &TEXT,
        "The parameter the next page's token is sent in",
    ),
    f("limit", &TEXT, "The parameter of the page's size"),
];
pub(crate) const OFFSET_REQUEST: &[Field] = &[
    req("offset", &TEXT, "The parameter of the first item's offset"),
    f("limit", &TEXT, "The parameter of the page's size"),
];
pub(crate) const PAGE_REQUEST: &[Field] = &[
    req("page", &TEXT, "The parameter of the page's number"),
    f("per_page", &TEXT, "The parameter of the page's size"),
];
pub(crate) const LIMIT_REQUEST: &[Field] = &[f("limit", &TEXT, "The parameter of the page's size")];

const ITEMS: Field = f(
    "items",
    &TEXT,
    "The path of the items' array in the body (`data.items`); the body itself when not given",
);
const HAS_MORE: Field = f(
    "has_more",
    &TEXT,
    "The path of a boolean that says whether another page follows",
);

pub(crate) const CURSOR_RESPONSE: &[Field] = &[
    ITEMS,
    req("next", &TEXT, "The path of the next page's token"),
    HAS_MORE,
];
pub(crate) const LAST_ID_RESPONSE: &[Field] = &[
    ITEMS,
    def(
        "id",
        &TEXT,
        "\"id\"",
        "The member of an item whose value the next page starts after",
    ),
    HAS_MORE,
];
pub(crate) const OFFSET_RESPONSE: &[Field] = &[
    ITEMS,
    f(
        "total",
        &TEXT,
        "The path of the number of items in every page",
    ),
];
pub(crate) const PAGE_RESPONSE: &[Field] = &[
    ITEMS,
    f("total_pages", &TEXT, "The path of the number of pages"),
    HAS_MORE,
];
pub(crate) const LINK_RESPONSE: &[Field] = &[
    ITEMS,
    f("next", &TEXT, "The path of the next page's URL in the body"),
    f(
        "header",
        &TEXT,
        "The response header whose `rel=\"next\"` link is the next page (`Link`)",
    ),
];
pub(crate) const SINGLE_RESPONSE: &[Field] = &[ITEMS];

/// A pagination scheme's `type`, its tag.
macro_rules! kind {
    ($name:literal) => {
        req("type", &ANY, $name)
    };
}

/// A scheme's request parameters, by name.
macro_rules! request {
    ($fields:expr) => {
        req(
            "request",
            &Shape::Obj($fields),
            "Request parameters, by name",
        )
    };
}

/// Where a scheme's response says what the next page is.
macro_rules! response {
    ($fields:expr) => {
        req(
            "response",
            &Shape::Obj($fields),
            "Where the body says what the next page is",
        )
    };
}

pub(crate) static SCHEME: Shape = Shape::Named("pagination", &Shape::Tagged(SCHEMES));

/// The pagination schemes by `type`, each with its members.
pub(crate) const SCHEMES: &[(&str, &[Field])] = &[
    (
        "cursor",
        &[
            kind!("cursor"),
            request!(CURSOR_REQUEST),
            response!(CURSOR_RESPONSE),
        ],
    ),
    (
        "last_item_id",
        &[
            kind!("last_item_id"),
            request!(CURSOR_REQUEST),
            response!(LAST_ID_RESPONSE),
        ],
    ),
    (
        "offset",
        &[
            kind!("offset"),
            request!(OFFSET_REQUEST),
            response!(OFFSET_RESPONSE),
        ],
    ),
    (
        "page",
        &[
            kind!("page"),
            request!(PAGE_REQUEST),
            response!(PAGE_RESPONSE),
            def("start", &START, "1", "The first page's number"),
        ],
    ),
    (
        "link",
        &[
            kind!("link"),
            f(
                "request",
                &Shape::Obj(LIMIT_REQUEST),
                "Request parameters, by name",
            ),
            response!(LINK_RESPONSE),
        ],
    ),
    (
        "single",
        &[
            kind!("single"),
            f(
                "response",
                &Shape::Obj(SINGLE_RESPONSE),
                "Where the items are",
            ),
        ],
    ),
];
