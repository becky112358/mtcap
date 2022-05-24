use super::*;

use std::str::FromStr;

#[test]
fn key_from_str() {
    assert!(Key::from_str("00").is_err());
    assert!(Key::from_str("------------------------------------").is_err());

    assert_eq!(
        Key::from_str("00000000000000000000000000000000")
            .unwrap()
            .digits,
        [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00
        ]
    );
    assert_eq!(
        Key::from_str("01010101010101010101010101010101")
            .unwrap()
            .digits,
        [
            0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
            0x01, 0x01
        ]
    );
    assert_eq!(
        Key::from_str("01-23-45-67-89-ab-cd-ef-01-23-45-67-89-ab-cd-ef")
            .unwrap()
            .digits,
        [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab,
            0xcd, 0xef
        ]
    );
}
