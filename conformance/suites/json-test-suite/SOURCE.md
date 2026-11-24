# Where these files come from

Copied unchanged from `github.com/nst/JSONTestSuite` (`test_parsing/`), at commit
`1ef36fa01286573e846ac449e8683f8833c5b26a` (2024-11-22), MIT (`LICENSE` beside this file). Nothing here is ours: change nothing in
place, so an update stays a plain copy.

Each file is one JSON text: `y_` must be accepted, `n_` refused, `i_` is the parser's choice.
`crates/flowapi/tests/json.rs` holds flowapi's reader to them (oracle O4); what it does with each
`i_` case is written in that test.

To update: clone the repository, read what changed since the commit above, copy `test_parsing/`
and `LICENSE` over these, and put the new commit here.

    git clone --depth 1 https://github.com/nst/JSONTestSuite
    git -C JSONTestSuite log -1 --format='%H %cs'
