//! Whitebox tests for the deEHR Identity / DID Registry.
//!
//! Covers the full lifecycle (register / resolve / update / deactivate) plus the
//! negative cases called out in issue #27: replay, wrong signature, wrong key,
//! unknown DID, double-register, and update-after-deactivate.
//!
//! All data here is synthetic — no PHI (the contract stores none anyway).
//!
//! Requires the built artifact: run `ksc all build` (produces
//! `output/deehr-identity-registry.kleversc.json`) before `cargo test`.

use deehr_identity_registry::*;
use ed25519_dalek::{Signer, SigningKey};
use klever_sc_scenario::imports::*;

const CODE_PATH: &str = "kleversc:output/deehr-identity-registry.kleversc.json";
const SC_ADDR: &str = "sc:identity";
const SC: TestSCAddress = TestSCAddress::new("identity");
const OWNER_EXPR: &str = "address:owner";
const OWNER: TestAddress = TestAddress::new("owner");

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();
    blockchain.register_contract(CODE_PATH, deehr_identity_registry::ContractBuilder);
    blockchain
}

/// Deploy the contract from `owner` and return the ready world.
fn deploy() -> ScenarioWorld {
    let mut world = world();
    let contract = WhiteboxContract::new(SC_ADDR, deehr_identity_registry::contract_obj);
    let code = world.code_expression(CODE_PATH);
    world.set_state_step(
        SetStateStep::new()
            .put_account(OWNER_EXPR, Account::new().nonce(1))
            .new_address(OWNER_EXPR, 2, SC_ADDR),
    );
    world.whitebox_deploy(
        &contract,
        ScDeployStep::new().from(OWNER_EXPR).code(code),
        |sc| sc.init(),
    );
    world
}

// ---- signed-message builders (mirror the contract's `signed_message`) ----

fn register_message(sc: &[u8; 32], did: &[u8; 32], doc_hash: &[u8; 32], key: &[u8; 32]) -> Vec<u8> {
    let mut m = Vec::with_capacity(1 + 32 + 32 + 32 + 32 + 8);
    m.push(0x00);
    m.extend_from_slice(sc);
    m.extend_from_slice(did);
    m.extend_from_slice(doc_hash);
    m.extend_from_slice(key);
    m.extend_from_slice(&0u64.to_be_bytes());
    m
}

fn update_message(
    sc: &[u8; 32],
    did: &[u8; 32],
    doc_hash: &[u8; 32],
    key: &[u8; 32],
    nonce: u64,
) -> Vec<u8> {
    let mut m = Vec::with_capacity(1 + 32 + 32 + 32 + 32 + 8);
    m.push(0x01);
    m.extend_from_slice(sc);
    m.extend_from_slice(did);
    m.extend_from_slice(doc_hash);
    m.extend_from_slice(key);
    m.extend_from_slice(&nonce.to_be_bytes());
    m
}

fn deactivate_message(sc: &[u8; 32], did: &[u8; 32], nonce: u64) -> Vec<u8> {
    let mut m = Vec::with_capacity(1 + 32 + 32 + 8);
    m.push(0x02);
    m.extend_from_slice(sc);
    m.extend_from_slice(did);
    m.extend_from_slice(&nonce.to_be_bytes());
    m
}

fn mba<const N: usize>(bytes: &[u8; N]) -> ManagedByteArray<DebugApi, N> {
    ManagedByteArray::new_from_bytes(bytes)
}

fn owner_did() -> ManagedAddress<DebugApi> {
    ManagedAddress::from(&OWNER.eval_to_array())
}

/// Assert the call succeeded and emitted a klever_sc event whose identifier is
/// `identifier` (the event name is encoded as the log's first topic), and return
/// that event's topics so the caller can check the indexed values too.
fn expect_event(result: TxResult, identifier: &[u8]) -> Vec<Vec<u8>> {
    result.assert_ok();
    result
        .result_logs
        .into_iter()
        .find(|log| log.topics.first().map(Vec::as_slice) == Some(identifier))
        .unwrap_or_else(|| {
            panic!(
                "expected event {:?} to be emitted",
                core::str::from_utf8(identifier).unwrap_or("<binary>")
            )
        })
        .topics
}

// ---------------------------------------------------------------------------
// Happy-path lifecycle
// ---------------------------------------------------------------------------

#[test]
fn full_lifecycle_register_update_deactivate() {
    let mut world = deploy();
    let contract = WhiteboxContract::new(SC_ADDR, deehr_identity_registry::contract_obj);
    let did_bytes = OWNER.eval_to_array();
    let sc_bytes = SC.eval_to_array();

    // Key for the #klv-1 verification method.
    let sk1 = SigningKey::from_bytes(&[11u8; 32]);
    let pk1 = sk1.verifying_key().to_bytes();
    let doc1 = [0xA1u8; 32];
    let reg_sig = sk1
        .sign(&register_message(&sc_bytes, &did_bytes, &doc1, &pk1))
        .to_bytes();

    // register (with proof-of-possession of the #klv-1 key)
    world.whitebox_call_check(
        &contract,
        ScCallStep::new().from(OWNER_EXPR),
        |sc| {
            sc.register_did(mba(&doc1), mba(&pk1), mba(&reg_sig));
        },
        |tx| {
            let topics = expect_event(tx, b"didRegistered");
            assert_eq!(topics[1], did_bytes.to_vec(), "didRegistered: did topic");
            assert_eq!(topics[2], doc1.to_vec(), "didRegistered: doc_hash topic");
        },
    );

    world.whitebox_query(&contract, |sc| {
        let rec = sc.resolve_did(owner_did());
        assert!(!rec.deactivated);
        assert_eq!(rec.nonce, 0);
        assert_eq!(rec.doc_hash.to_byte_array(), doc1);
        assert_eq!(rec.primary_key.to_byte_array(), pk1);
    });

    // update: rotate to key2 + new doc hash, signed by the CURRENT key (sk1) over nonce 0
    let sk2 = SigningKey::from_bytes(&[22u8; 32]);
    let pk2 = sk2.verifying_key().to_bytes();
    let doc2 = [0xB2u8; 32];
    let sig = sk1
        .sign(&update_message(&sc_bytes, &did_bytes, &doc2, &pk2, 0))
        .to_bytes();
    world.whitebox_call_check(
        &contract,
        ScCallStep::new().from(OWNER_EXPR),
        |sc| {
            sc.update_did(mba(&doc2), mba(&pk2), mba(&sig));
        },
        |tx| {
            let topics = expect_event(tx, b"didUpdated");
            assert_eq!(topics[1], did_bytes.to_vec(), "didUpdated: did topic");
            assert_eq!(topics[2], doc2.to_vec(), "didUpdated: doc_hash topic");
        },
    );

    world.whitebox_query(&contract, |sc| {
        let rec = sc.resolve_did(owner_did());
        assert_eq!(rec.nonce, 1);
        assert_eq!(rec.doc_hash.to_byte_array(), doc2);
        assert_eq!(rec.primary_key.to_byte_array(), pk2);
        // rotation history keeps the previous key
        let history = sc.key_history(&owner_did());
        assert_eq!(history.len(), 1);
        assert_eq!(history.get(1).previous_key.to_byte_array(), pk1);
    });

    // deactivate: signed by the now-current key (sk2) over nonce 1
    let sig = sk2
        .sign(&deactivate_message(&sc_bytes, &did_bytes, 1))
        .to_bytes();
    world.whitebox_call_check(
        &contract,
        ScCallStep::new().from(OWNER_EXPR),
        |sc| {
            sc.deactivate_did(mba(&sig));
        },
        |tx| {
            let topics = expect_event(tx, b"didDeactivated");
            assert_eq!(topics[1], did_bytes.to_vec(), "didDeactivated: did topic");
        },
    );

    world.whitebox_query(&contract, |sc| {
        let rec = sc.resolve_did(owner_did());
        assert!(rec.deactivated);
        assert_eq!(rec.nonce, 2);
    });
}

// ---------------------------------------------------------------------------
// Negative cases
// ---------------------------------------------------------------------------

#[test]
fn register_twice_fails() {
    let mut world = deploy();
    let contract = WhiteboxContract::new(SC_ADDR, deehr_identity_registry::contract_obj);
    let sc_bytes = SC.eval_to_array();
    let did_bytes = OWNER.eval_to_array();
    let sk = SigningKey::from_bytes(&[1u8; 32]);
    let pk = sk.verifying_key().to_bytes();
    let doc = [0x01u8; 32];
    let reg_sig = sk
        .sign(&register_message(&sc_bytes, &did_bytes, &doc, &pk))
        .to_bytes();
    world.whitebox_call(&contract, ScCallStep::new().from(OWNER_EXPR), |sc| {
        sc.register_did(mba(&doc), mba(&pk), mba(&reg_sig));
    });
    // Second register reverts on the is_empty gate (before the PoP check), so the
    // reused signature is irrelevant.
    world.whitebox_call_check(
        &contract,
        ScCallStep::new()
            .from(OWNER_EXPR)
            .expect(TxExpect::user_error("str:DID already registered")),
        |sc| sc.register_did(mba(&doc), mba(&pk), mba(&reg_sig)),
        |_| {},
    );
}

#[test]
fn update_unknown_did_fails() {
    let mut world = deploy();
    let contract = WhiteboxContract::new(SC_ADDR, deehr_identity_registry::contract_obj);
    let sig = [0u8; 64];
    world.whitebox_call_check(
        &contract,
        ScCallStep::new()
            .from(OWNER_EXPR)
            .expect(TxExpect::user_error("str:unknown DID")),
        |sc| sc.update_did(mba(&[0u8; 32]), mba(&[0u8; 32]), mba(&sig)),
        |_| {},
    );
}

#[test]
fn update_wrong_signature_fails() {
    let mut world = deploy();
    let contract = WhiteboxContract::new(SC_ADDR, deehr_identity_registry::contract_obj);
    let did_bytes = OWNER.eval_to_array();
    let sc_bytes = SC.eval_to_array();
    let sk1 = SigningKey::from_bytes(&[11u8; 32]);
    let pk1 = sk1.verifying_key().to_bytes();
    let reg_sig = sk1
        .sign(&register_message(
            &sc_bytes,
            &did_bytes,
            &[0xA1u8; 32],
            &pk1,
        ))
        .to_bytes();
    world.whitebox_call(&contract, ScCallStep::new().from(OWNER_EXPR), |sc| {
        sc.register_did(mba(&[0xA1u8; 32]), mba(&pk1), mba(&reg_sig));
    });

    // Sign with the WRONG key (sk_other), not the registered primary key.
    let sk_other = SigningKey::from_bytes(&[99u8; 32]);
    let doc2 = [0xB2u8; 32];
    let pk2 = SigningKey::from_bytes(&[22u8; 32])
        .verifying_key()
        .to_bytes();
    let bad_sig = sk_other
        .sign(&update_message(&sc_bytes, &did_bytes, &doc2, &pk2, 0))
        .to_bytes();
    world.whitebox_call_check(
        &contract,
        ScCallStep::new()
            .from(OWNER_EXPR)
            .expect(TxExpect::err("62", "str:invalid signature")),
        |sc| sc.update_did(mba(&doc2), mba(&pk2), mba(&bad_sig)),
        |r| {
            assert_ne!(
                r.result_status, 0,
                "expected signature verification to fail"
            )
        },
    );
}

#[test]
fn replay_of_old_update_signature_fails() {
    let mut world = deploy();
    let contract = WhiteboxContract::new(SC_ADDR, deehr_identity_registry::contract_obj);
    let did_bytes = OWNER.eval_to_array();
    let sc_bytes = SC.eval_to_array();
    let sk1 = SigningKey::from_bytes(&[11u8; 32]);
    let pk1 = sk1.verifying_key().to_bytes();
    let reg_sig = sk1
        .sign(&register_message(
            &sc_bytes,
            &did_bytes,
            &[0xA1u8; 32],
            &pk1,
        ))
        .to_bytes();
    world.whitebox_call(&contract, ScCallStep::new().from(OWNER_EXPR), |sc| {
        sc.register_did(mba(&[0xA1u8; 32]), mba(&pk1), mba(&reg_sig));
    });

    // First update (nonce 0) — valid, rotates key to pk2 and advances nonce to 1.
    let sk2 = SigningKey::from_bytes(&[22u8; 32]);
    let pk2 = sk2.verifying_key().to_bytes();
    let doc2 = [0xB2u8; 32];
    let sig0 = sk1
        .sign(&update_message(&sc_bytes, &did_bytes, &doc2, &pk2, 0))
        .to_bytes();
    world.whitebox_call(&contract, ScCallStep::new().from(OWNER_EXPR), |sc| {
        sc.update_did(mba(&doc2), mba(&pk2), mba(&sig0));
    });

    // Intervening real update (nonce 1, signed by the now-current key sk2) so the
    // record moves to (doc3, pk3). Without this, replaying sig0's exact (doc2,
    // pk2) args would simply be a no-op and get rejected by the no-op guard
    // before the signature check — here we exercise genuine nonce-replay defense.
    let sk3 = SigningKey::from_bytes(&[33u8; 32]);
    let pk3 = sk3.verifying_key().to_bytes();
    let doc3 = [0xC3u8; 32];
    let sig1 = sk2
        .sign(&update_message(&sc_bytes, &did_bytes, &doc3, &pk3, 1))
        .to_bytes();
    world.whitebox_call(&contract, ScCallStep::new().from(OWNER_EXPR), |sc| {
        sc.update_did(mba(&doc3), mba(&pk3), mba(&sig1));
    });

    // Replay the SAME (nonce-0) signature — its args (doc2, pk2) now differ from
    // the current record (doc3, pk3), so it clears the no-op guard, but the
    // record nonce is now 2, so the signed message no longer matches and
    // verification must fail.
    world.whitebox_call_check(
        &contract,
        ScCallStep::new()
            .from(OWNER_EXPR)
            .expect(TxExpect::err("62", "str:invalid signature")),
        |sc| sc.update_did(mba(&doc2), mba(&pk2), mba(&sig0)),
        |r| assert_ne!(r.result_status, 0, "expected replayed signature to fail"),
    );
}

#[test]
fn update_after_deactivate_fails() {
    let mut world = deploy();
    let contract = WhiteboxContract::new(SC_ADDR, deehr_identity_registry::contract_obj);
    let did_bytes = OWNER.eval_to_array();
    let sc_bytes = SC.eval_to_array();
    let sk1 = SigningKey::from_bytes(&[11u8; 32]);
    let pk1 = sk1.verifying_key().to_bytes();
    let reg_sig = sk1
        .sign(&register_message(
            &sc_bytes,
            &did_bytes,
            &[0xA1u8; 32],
            &pk1,
        ))
        .to_bytes();
    world.whitebox_call(&contract, ScCallStep::new().from(OWNER_EXPR), |sc| {
        sc.register_did(mba(&[0xA1u8; 32]), mba(&pk1), mba(&reg_sig));
    });

    let sig = sk1
        .sign(&deactivate_message(&sc_bytes, &did_bytes, 0))
        .to_bytes();
    world.whitebox_call(&contract, ScCallStep::new().from(OWNER_EXPR), |sc| {
        sc.deactivate_did(mba(&sig));
    });

    // Any update after deactivation is rejected before signature checks.
    let sig2 = sk1
        .sign(&update_message(
            &sc_bytes,
            &did_bytes,
            &[0xC3u8; 32],
            &pk1,
            1,
        ))
        .to_bytes();
    world.whitebox_call_check(
        &contract,
        ScCallStep::new()
            .from(OWNER_EXPR)
            .expect(TxExpect::user_error("str:DID deactivated")),
        |sc| sc.update_did(mba(&[0xC3u8; 32]), mba(&pk1), mba(&sig2)),
        |_| {},
    );
}

#[test]
fn register_wrong_pop_signature_fails() {
    let mut world = deploy();
    let contract = WhiteboxContract::new(SC_ADDR, deehr_identity_registry::contract_obj);
    let sc_bytes = SC.eval_to_array();
    let did_bytes = OWNER.eval_to_array();
    let pk = SigningKey::from_bytes(&[5u8; 32])
        .verifying_key()
        .to_bytes();
    let doc = [0xD1u8; 32];

    // Proof-of-possession signed by a DIFFERENT key than `pk` — i.e. the caller
    // does not control the key it is trying to register. Registration must revert
    // (audit finding 4.1: no key can be bound without proving possession).
    let sk_other = SigningKey::from_bytes(&[77u8; 32]);
    let bad_pop = sk_other
        .sign(&register_message(&sc_bytes, &did_bytes, &doc, &pk))
        .to_bytes();
    world.whitebox_call_check(
        &contract,
        ScCallStep::new()
            .from(OWNER_EXPR)
            .expect(TxExpect::err("62", "str:invalid signature")),
        |sc| sc.register_did(mba(&doc), mba(&pk), mba(&bad_pop)),
        |_| {},
    );
}

// ---------------------------------------------------------------------------
// No-op guard (updateDid)
// ---------------------------------------------------------------------------

/// A doc-only patch (key unchanged) is allowed — updateDid is the only way to
/// rotate the document hash — and must NOT append a keyHistory entry.
#[test]
fn update_doc_only_keeps_key_and_skips_history() {
    let mut world = deploy();
    let contract = WhiteboxContract::new(SC_ADDR, deehr_identity_registry::contract_obj);
    let did_bytes = OWNER.eval_to_array();
    let sc_bytes = SC.eval_to_array();

    let sk1 = SigningKey::from_bytes(&[11u8; 32]);
    let pk1 = sk1.verifying_key().to_bytes();
    let doc1 = [0xA1u8; 32];
    let reg_sig = sk1
        .sign(&register_message(&sc_bytes, &did_bytes, &doc1, &pk1))
        .to_bytes();
    world.whitebox_call(&contract, ScCallStep::new().from(OWNER_EXPR), |sc| {
        sc.register_did(mba(&doc1), mba(&pk1), mba(&reg_sig));
    });

    // Patch the doc hash only — same key (pk1), signed by the current key.
    let doc2 = [0xB2u8; 32];
    let sig = sk1
        .sign(&update_message(&sc_bytes, &did_bytes, &doc2, &pk1, 0))
        .to_bytes();
    world.whitebox_call(&contract, ScCallStep::new().from(OWNER_EXPR), |sc| {
        sc.update_did(mba(&doc2), mba(&pk1), mba(&sig));
    });

    world.whitebox_query(&contract, |sc| {
        let rec = sc.resolve_did(owner_did());
        assert_eq!(rec.nonce, 1);
        assert_eq!(rec.doc_hash.to_byte_array(), doc2);
        assert_eq!(rec.primary_key.to_byte_array(), pk1);
        // key unchanged -> no rotation recorded
        assert_eq!(sc.key_history(&owner_did()).len(), 0);
    });
}

/// An update that changes neither the doc hash nor the key is rejected before
/// the signature check (cheap guard first), so it cannot consume a nonce.
#[test]
fn update_true_noop_is_rejected() {
    let mut world = deploy();
    let contract = WhiteboxContract::new(SC_ADDR, deehr_identity_registry::contract_obj);
    let did_bytes = OWNER.eval_to_array();
    let sc_bytes = SC.eval_to_array();

    let sk1 = SigningKey::from_bytes(&[11u8; 32]);
    let pk1 = sk1.verifying_key().to_bytes();
    let doc1 = [0xA1u8; 32];
    let reg_sig = sk1
        .sign(&register_message(&sc_bytes, &did_bytes, &doc1, &pk1))
        .to_bytes();
    world.whitebox_call(&contract, ScCallStep::new().from(OWNER_EXPR), |sc| {
        sc.register_did(mba(&doc1), mba(&pk1), mba(&reg_sig));
    });

    // Re-submit the exact current state (doc1, pk1) — a true no-op.
    let sig = sk1
        .sign(&update_message(&sc_bytes, &did_bytes, &doc1, &pk1, 0))
        .to_bytes();
    world.whitebox_call_check(
        &contract,
        ScCallStep::new()
            .from(OWNER_EXPR)
            .expect(TxExpect::user_error("str:no-op update")),
        |sc| sc.update_did(mba(&doc1), mba(&pk1), mba(&sig)),
        |_| {},
    );

    // Nonce must be untouched.
    world.whitebox_query(&contract, |sc| {
        assert_eq!(sc.resolve_did(owner_did()).nonce, 0);
    });
}

// ---------------------------------------------------------------------------
// Cross-deployment replay (instance binding — audit M-1)
// ---------------------------------------------------------------------------

const SC_B_ADDR: &str = "sc:identity-b";
const SC_B: TestSCAddress = TestSCAddress::new("identity-b");
const DEPLOYER_B_EXPR: &str = "address:deployer-b";

/// A signature produced for one deployment must be rejected on another. The
/// contract binds its own address into every signed message (audit M-1), so the
/// same PoP signature cannot be replayed across two contract instances even with
/// the same account, key and nonce — e.g. a signature captured on testnet cannot
/// be replayed on mainnet.
#[test]
fn replay_across_deployments_fails() {
    let mut world = world();
    let contract_a = WhiteboxContract::new(SC_ADDR, deehr_identity_registry::contract_obj);
    let contract_b = WhiteboxContract::new(SC_B_ADDR, deehr_identity_registry::contract_obj);
    let code = world.code_expression(CODE_PATH);

    // Two independent deployments at distinct addresses (each its deployer's
    // first deploy — mirrors the proven single-deploy nonce/new_address pattern).
    world.set_state_step(
        SetStateStep::new()
            .put_account(OWNER_EXPR, Account::new().nonce(1))
            .put_account(DEPLOYER_B_EXPR, Account::new().nonce(1))
            .new_address(OWNER_EXPR, 2, SC_ADDR)
            .new_address(DEPLOYER_B_EXPR, 2, SC_B_ADDR),
    );
    world.whitebox_deploy(
        &contract_a,
        ScDeployStep::new().from(OWNER_EXPR).code(code.clone()),
        |sc| sc.init(),
    );
    world.whitebox_deploy(
        &contract_b,
        ScDeployStep::new().from(DEPLOYER_B_EXPR).code(code),
        |sc| sc.init(),
    );

    let did_bytes = OWNER.eval_to_array();
    let sc_a_bytes = SC.eval_to_array();
    let sc_b_bytes = SC_B.eval_to_array();
    assert_ne!(sc_a_bytes, sc_b_bytes, "deployments must have distinct addresses");

    let sk = SigningKey::from_bytes(&[11u8; 32]);
    let pk = sk.verifying_key().to_bytes();
    let doc = [0xA1u8; 32];

    // Proof-of-possession signature bound to SC-A's address.
    let sig_for_a = sk
        .sign(&register_message(&sc_a_bytes, &did_bytes, &doc, &pk))
        .to_bytes();

    // Sanity check: it is accepted on SC-A.
    world.whitebox_call(&contract_a, ScCallStep::new().from(OWNER_EXPR), |sc| {
        sc.register_did(mba(&doc), mba(&pk), mba(&sig_for_a));
    });

    // The SAME signature, replayed verbatim on SC-B (a different address), must be
    // rejected: SC-B rebuilds the signed message with its own address, so the
    // signature no longer matches.
    world.whitebox_call_check(
        &contract_b,
        ScCallStep::new()
            .from(OWNER_EXPR)
            .expect(TxExpect::err("62", "str:invalid signature")),
        |sc| sc.register_did(mba(&doc), mba(&pk), mba(&sig_for_a)),
        |r| assert_ne!(r.result_status, 0, "cross-deployment replay must fail"),
    );
}
