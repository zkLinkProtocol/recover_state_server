use zklink_crypto::franklin_crypto::{
    eddsa::{PrivateKey, PublicKey},
    jubjub::FixedGenerators,
};
use zklink_crypto::params::{MAX_ACCOUNT_ID, JUBJUB_PARAMS, MAX_REAL_TOKEN_ID};
use zklink_crypto::public_key_from_private;
use zklink_crypto::rand::{Rng, SeedableRng, XorShiftRng};

use num::{BigUint, ToPrimitive};

use super::*;
use crate::{
    helpers::{pack_fee_amount, pack_token_amount},
    AccountId, Engine, Nonce, TokenId,
};

fn gen_pk_and_msg() -> (PrivateKey<Engine>, Vec<Vec<u8>>) {
    let mut rng = XorShiftRng::from_seed([1, 2, 3, 4]);

    let pk = PrivateKey(rng.gen());

    let mut messages = Vec::new();
    messages.push(Vec::<u8>::new());
    messages.push(b"hello world".to_vec());

    (pk, messages)
}

fn gen_account_id<T: Rng>(rng: &mut T) -> AccountId {
    AccountId(rng.gen::<u32>().min(*MAX_ACCOUNT_ID))
}

fn gen_token_id<T: Rng>(rng: &mut T) -> TokenId {
    TokenId(rng.gen::<u32>().min(*MAX_REAL_TOKEN_ID))
}

#[test]
fn test_print_transfer_for_protocol() {
    let mut rng = XorShiftRng::from_seed([1, 2, 3, 4]);
    let key = gen_pk_and_msg().0;
    let transfer = Transfer::new_signed(
        gen_account_id(&mut rng),
        Default::default(),
        Default::default(),
        Default::default(),
        gen_token_id(&mut rng),
        BigUint::from(12_340_000_000_000u64),
        BigUint::from(56_700_000_000u64),
        Nonce(rng.gen()),
        &key,
        Default::default(),
    )
    .expect("failed to sign transfer");

    println!(
        "User representation:\n{}\n",
        serde_json::to_string_pretty(&transfer).expect("json serialize")
    );

    println!("Signer:");
    println!("Private key: {}", key.0.to_string());
    let (pk_x, pk_y) = public_key_from_private(&key).0.into_xy();
    println!("Public key: x: {}, y: {}\n", pk_x, pk_y);

    let signed_fields = vec![
        ("type", vec![Transfer::TX_TYPE]),
        ("accountId", transfer.account_id.to_be_bytes().to_vec()),
        ("to", transfer.to.to_fixed_bytes().to_vec()),
        ("token", transfer.token.to_be_bytes().to_vec()),
        ("amount", pack_token_amount(&transfer.amount)),
        ("fee", pack_fee_amount(&transfer.fee)),
        ("nonce", transfer.nonce.to_be_bytes().to_vec()),
    ];
    println!("Signed transaction fields:");
    let mut field_concat = Vec::new();
    for (field, value) in signed_fields.into_iter() {
        println!("{}: 0x{}", field, hex::encode(&value));
        field_concat.extend(value.into_iter());
    }
    println!("Signed bytes: 0x{}", hex::encode(&field_concat));
    assert_eq!(
        field_concat,
        transfer.get_bytes(),
        "Protocol serialization mismatch"
    );
}

#[test]
fn test_print_withdraw_for_protocol() {
    let mut rng = XorShiftRng::from_seed([2, 2, 3, 4]);
    let key = gen_pk_and_msg().0;
    let withdraw = Withdraw::new_signed(
        gen_account_id(&mut rng),
        Default::default(),
        Default::default(),
        Default::default(),
        gen_token_id(&mut rng),
        Default::default(),
        BigUint::from(12_340_000_000_000u64),
        BigUint::from(56_700_000_000u64),
        Nonce(rng.gen()),
        Default::default(),
        Default::default(),
        &key,
        Default::default(),
    )
    .expect("failed to sign withdraw");

    println!(
        "User representation:\n{}\n",
        serde_json::to_string_pretty(&withdraw).expect("json serialize")
    );

    println!("Signer:");
    println!("Private key: {}", key.0.to_string());
    let (pk_x, pk_y) = public_key_from_private(&key).0.into_xy();
    println!("Public key: x: {}, y: {}\n", pk_x, pk_y);

    let signed_fields = vec![
        ("type", vec![Withdraw::TX_TYPE]),
        ("accountId", withdraw.account_id.to_be_bytes().to_vec()),
        ("to", withdraw.to.to_fixed_bytes().to_vec()),
        ("token", withdraw.l2_source_token.to_be_bytes().to_vec()),
        (
            "fullAmount",
            withdraw.amount.to_u128().unwrap().to_be_bytes().to_vec(),
        ),
        ("fee", pack_fee_amount(&withdraw.fee)),
        ("nonce", withdraw.nonce.to_be_bytes().to_vec()),
    ];
    println!("Signed transaction fields:");
    let mut field_concat = Vec::new();
    for (field, value) in signed_fields.into_iter() {
        println!("{}: 0x{}", field, hex::encode(&value));
        field_concat.extend(value.into_iter());
    }
    println!("Signed bytes: 0x{}", hex::encode(&field_concat));
    assert_eq!(
        field_concat,
        withdraw.get_bytes(),
        "Protocol serialization mismatch1"
    );
}

#[test]
fn test_musig_rescue_signing_verification() {
    let (pk, messages) = gen_pk_and_msg();

    for msg in &messages {
        let signature = TxSignature::sign_musig_rescue(&pk, msg);

        if let Some(sign_pub_key) = signature.verify_musig_rescue(msg) {
            let pub_key =
                PublicKey::from_private(&pk, FixedGenerators::SpendingKeyGenerator, &JUBJUB_PARAMS);
            assert!(
                sign_pub_key.0.eq(&pub_key.0),
                "Signature pub key is wrong, msg: {}",
                hex::encode(&msg)
            );
        } else {
            panic!("Signature is incorrect, msg: {}", hex::encode(&msg));
        }
    }
}

#[test]
fn test_check_signature() {
    let (pk, msg) = gen_pk_and_msg();
    let signature = TxSignature::sign_musig(&pk, &msg[1])
        .signature
        .serialize_packed()
        .unwrap();

    assert_eq!(hex::encode(signature), "4e3298ac8cc13868dbbc94ad6fb41085ffe05b3c2eee22f88b05e69b7a5126aea723d7a3e7282ef5a32d9479c9c8dde52b3e3c462dd445dcd8158ebb6edb6000");
}
