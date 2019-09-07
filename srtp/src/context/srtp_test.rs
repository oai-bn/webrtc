use super::*;

use util::Error;

//TODO: BenchmarkEncryptRTP
//TODO: BenchmarkEncryptRTPInPlace
//TODO: BenchmarkDecryptRTP

fn build_test_context() -> Result<Context, Error> {
    let master_key = vec![
        0x0d, 0xcd, 0x21, 0x3e, 0x4c, 0xbc, 0xf2, 0x8f, 0x01, 0x7f, 0x69, 0x94, 0x40, 0x1e, 0x28,
        0x89,
    ];
    let master_salt = vec![
        0x62, 0x77, 0x60, 0x38, 0xc0, 0x6d, 0xc9, 0x41, 0x9f, 0x6d, 0xd9, 0x43, 0x3e, 0x7c,
    ];

    Context::new(
        master_key,
        master_salt,
        PROTECTION_PROFILE_AES128CM_HMAC_SHA1_80,
    )
}

struct RTPTestCase {
    sequence_number: u16,
    encrypted: Vec<u8>,
}

lazy_static! {
    static ref rtpTestCaseDecrypted: Vec<u8> = vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05];
    static ref rtpTestCases: Vec<RTPTestCase> = vec![
        RTPTestCase {
            sequence_number: 5000,
            encrypted: vec![
                0x6d, 0xd3, 0x7e, 0xd5, 0x99, 0xb7, 0x2d, 0x28, 0xb1, 0xf3, 0xa1, 0xf0, 0xc, 0xfb,
                0xfd, 0x8
            ],
        },
        RTPTestCase {
            sequence_number: 5001,
            encrypted: vec![
                0xda, 0x47, 0xb, 0x2a, 0x74, 0x53, 0x65, 0xbd, 0x2f, 0xeb, 0xdc, 0x4b, 0x6d, 0x23,
                0xf3, 0xde
            ],
        },
        RTPTestCase {
            sequence_number: 5002,
            encrypted: vec![
                0x6e, 0xa7, 0x69, 0x8d, 0x24, 0x6d, 0xdc, 0xbf, 0xec, 0x2, 0x1c, 0xd1, 0x60, 0x76,
                0xc1, 0xe
            ],
        },
        RTPTestCase {
            sequence_number: 5003,
            encrypted: vec![
                0x24, 0x7e, 0x96, 0xc8, 0x7d, 0x33, 0xa2, 0x92, 0x8d, 0x13, 0x8d, 0xe0, 0x76, 0x9f,
                0x8, 0xdc
            ],
        },
        RTPTestCase {
            sequence_number: 5004,
            encrypted: vec![
                0x75, 0x43, 0x28, 0xe4, 0x3a, 0x77, 0x59, 0x9b, 0x2e, 0xdf, 0x7b, 0x12, 0x68, 0xb,
                0x57, 0x49
            ],
        },
    ];
}

#[test]
fn test_rtp_invalid_auth() -> Result<(), Error> {
    let master_key = vec![
        0x0d, 0xcd, 0x21, 0x3e, 0x4c, 0xbc, 0xf2, 0x8f, 0x01, 0x7f, 0x69, 0x94, 0x40, 0x1e, 0x28,
        0x89,
    ];
    let invalid_salt = vec![
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    let mut encrypt_context = build_test_context()?;
    let mut invalid_context = Context::new(
        master_key,
        invalid_salt,
        PROTECTION_PROFILE_AES128CM_HMAC_SHA1_80,
    )?;

    for testCase in rtpTestCases.iter() {
        let pkt = rtp::packet::Packet {
            header: rtp::packet::Header {
                sequence_number: testCase.sequence_number,
                ..Default::default()
            },
            payload: rtpTestCaseDecrypted.clone(),
        };
        let mut pkt_raw: Vec<u8> = vec![];
        {
            let mut writer = BufWriter::<&mut Vec<u8>>::new(pkt_raw.as_mut());
            pkt.marshal(&mut writer)?;
        }

        let out = encrypt_context.encrypt_rtp(&pkt_raw)?;

        let result = invalid_context.decrypt_rtp(&out);
        assert!(
            result.is_err(),
            "Managed to decrypt with incorrect salt for packet with SeqNum: {}",
            testCase.sequence_number
        );
    }

    Ok(())
}

#[test]
fn test_rtp_lifecyle() -> Result<(), Error> {
    let mut encrypt_context = build_test_context()?;
    let mut decrypt_context = build_test_context()?;

    for testCase in rtpTestCases.iter() {
        let decrypted_pkt = rtp::packet::Packet {
            header: rtp::packet::Header {
                sequence_number: testCase.sequence_number,
                ..Default::default()
            },
            payload: rtpTestCaseDecrypted.clone(),
        };

        let mut decrypted_raw: Vec<u8> = vec![];
        {
            let mut writer = BufWriter::<&mut Vec<u8>>::new(decrypted_raw.as_mut());
            decrypted_pkt.marshal(&mut writer)?;
        }

        let encrypted_pkt = rtp::packet::Packet {
            header: rtp::packet::Header {
                sequence_number: testCase.sequence_number,
                ..Default::default()
            },
            payload: testCase.encrypted.clone(),
        };
        let mut encrypted_raw: Vec<u8> = vec![];
        {
            let mut writer = BufWriter::<&mut Vec<u8>>::new(encrypted_raw.as_mut());
            encrypted_pkt.marshal(&mut writer)?;
        }

        let actual_encrypted = encrypt_context.encrypt_rtp(&decrypted_raw)?;
        assert_eq!(
            actual_encrypted, encrypted_raw,
            "RTP packet with SeqNum invalid encryption: {}",
            testCase.sequence_number
        );

        let actual_decrypted = decrypt_context.decrypt_rtp(&encrypted_raw)?;
        assert_ne!(
            encrypted_raw[..encrypted_raw.len() - AUTH_TAG_SIZE].to_vec(),
            actual_decrypted,
            "DecryptRTP improperly encrypted in place"
        );

        assert_eq!(
            actual_decrypted, decrypted_raw,
            "RTP packet with SeqNum invalid decryption: {}",
            testCase.sequence_number,
        )
    }

    Ok(())
}