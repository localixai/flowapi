//! The digest against FIPS 180-4's examples (the empty message, one block, two blocks, a million
//! `a`), the lengths where padding takes another block, and a 1 MiB input whose digest is the
//! system's `sha256sum`.

use super::sha256_hex;

#[test]
fn fips_180_4_vectors() {
    let cases: [(&[u8], &str); 4] = [
        (
            b"",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ),
        (
            b"abc",
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
        (
            b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
        ),
        (
            b"abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
            "cf5b16a778af8380036ce59e7b0492370b249b11e8f07a51afac45037afee9d1",
        ),
    ];
    for (input, want) in cases {
        assert_eq!(
            sha256_hex(input),
            want,
            "{}",
            String::from_utf8_lossy(input)
        );
    }
    assert_eq!(
        sha256_hex(&vec![b'a'; 1_000_000]),
        "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
    );
}

#[test]
fn padding_at_the_block_edges() {
    // 55 bytes fit the length in the same block; 56 need another; 64 are a whole block.
    let cases = [
        (
            55,
            "d5e285683cd4efc02d021a5c62014694958901005d6f71e89e0989fac77e4072",
        ),
        (
            56,
            "04c26261370ee7541549d16dee320c723e3fd14671e66a099afe0a377c16888e",
        ),
        (
            64,
            "7ce100971f64e7001e8fe5a51973ecdfe1ced42befe7ee8d5fd6219506b5393c",
        ),
    ];
    for (n, want) in cases {
        assert_eq!(sha256_hex(&vec![b'x'; n]), want, "{n} bytes");
    }
}

#[test]
fn one_mebibyte() {
    let bytes: Vec<u8> = (0..1u32 << 20)
        .map(|i| u8::try_from(i % 251).unwrap_or(0))
        .collect();
    assert_eq!(
        sha256_hex(&bytes),
        "631b84027d6b9e52b539c4e8373622d23032dfadc64d60af87339c9037e4f769"
    );
}
