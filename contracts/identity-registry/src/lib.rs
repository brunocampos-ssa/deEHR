#![no_std]

//! # deEHR Identity / DID Registry
//!
//! Klever KVM smart contract — the on-chain authority for `did:klever` DID
//! Documents, per [ADR-0004][adr-0004] (the `did:klever` method) and the
//! Identity registry slot in [ADR-0002 §3][adr-0002].
//!
//! MVP scope (issue #27): a single classical verification method, **Ed25519
//! `#klv-1` only** — post-quantum verification methods are deferred per
//! ADR-0004. No PHI is ever stored on-chain (ADR-0002 §2): only the SHA-256
//! hash of the off-chain DID Document, the Ed25519 verification-method public
//! key, key-rotation history, timestamps, a replay nonce, and a deactivation
//! flag.
//!
//! The DID's method-specific identifier **is the Klever account address**
//! (ADR-0004 §1); the account is its own controller. State-changing endpoints
//! therefore key off `get_caller()`, and `update`/`deactivate` additionally
//! require an Ed25519 signature by the *current* primary key over a
//! domain-separated, nonce-bound message (proof of key control + replay
//! protection).
//!
//! [adr-0004]: ../../../docs/architecture/adr-0004-did-klever-method.md
//! [adr-0002]: ../../../docs/architecture/adr-0002-on-chain-registry-design.md

use klever_sc::derive_imports::*;
use klever_sc::imports::*;

/// Domain-separation tags for signed operations — bind a signature to one
/// operation so a signature for one can never be replayed as another.
const SIG_DOMAIN_REGISTER: u8 = 0x00;
const SIG_DOMAIN_UPDATE: u8 = 0x01;
const SIG_DOMAIN_DEACTIVATE: u8 = 0x02;

/// On-chain DID record. Contains no PHI — only commitments and key state.
#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone)]
pub struct DidRecord<M: ManagedTypeApi> {
    /// SHA-256 hash of the off-chain DID Document.
    pub doc_hash: ManagedByteArray<M, 32>,
    /// Ed25519 public key of the `#klv-1` verification method.
    pub primary_key: ManagedByteArray<M, 32>,
    pub created_at: u64,
    pub updated_at: u64,
    /// Monotonic counter; bound into every signed message for replay protection.
    pub nonce: u64,
    pub deactivated: bool,
}

/// One entry in a DID's key-rotation history (ADR-0002 §3).
#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone)]
pub struct KeyRotation<M: ManagedTypeApi> {
    pub previous_key: ManagedByteArray<M, 32>,
    pub rotated_at: u64,
}

#[klever_sc::contract]
pub trait IdentityRegistry {
    #[init]
    fn init(&self) {}

    /// Upgrade entrypoint (owner-only at the VM level; deploy with
    /// `UPGRADEABLE` set — ADR-0002 §9). No storage migration in the MVP.
    #[upgrade]
    fn upgrade(&self) {}

    // ---- storage ----

    #[storage_mapper("did")]
    fn did_record(&self, did: &ManagedAddress) -> SingleValueMapper<DidRecord<Self::Api>>;

    #[view(keyHistory)]
    #[storage_mapper("keyHistory")]
    fn key_history(&self, did: &ManagedAddress) -> VecMapper<KeyRotation<Self::Api>>;

    // ---- events (ADR-0002 §7: <= 4 topics, raw 32-byte addresses) ----

    #[event("didRegistered")]
    fn did_registered_event(
        &self,
        #[indexed] did: &ManagedAddress,
        #[indexed] doc_hash: &ManagedByteArray<Self::Api, 32>,
    );

    #[event("didUpdated")]
    fn did_updated_event(
        &self,
        #[indexed] did: &ManagedAddress,
        #[indexed] doc_hash: &ManagedByteArray<Self::Api, 32>,
        #[indexed] nonce: u64,
    );

    #[event("didDeactivated")]
    fn did_deactivated_event(&self, #[indexed] did: &ManagedAddress, #[indexed] nonce: u64);

    // ---- endpoints ----

    /// Register a new `did:klever` DID Document for the caller's account.
    /// The DID's method-specific id is the caller address (ADR-0004 §1).
    ///
    /// `signature` is a **proof-of-possession**: the caller must control the
    /// `#klv-1` private key it is registering. It must be `primary_key`'s Ed25519
    /// signature over `0x00 || sc_address || did || doc_hash || primary_key ||
    /// 0u64(BE)` (the record's initial nonce). This stops a DID being registered
    /// with a key the holder cannot sign with, which would permanently freeze the
    /// DID's `update`/`deactivate` (both verify against the stored key).
    #[endpoint(registerDid)]
    fn register_did(
        &self,
        doc_hash: ManagedByteArray<Self::Api, 32>,
        primary_key: ManagedByteArray<Self::Api, 32>,
        signature: ManagedByteArray<Self::Api, 64>,
    ) {
        let did = self.blockchain().get_caller();
        require!(self.did_record(&did).is_empty(), "DID already registered");

        // Proof of possession of the #klv-1 key (verified against the *supplied*
        // key — there is no stored record yet). Panics/reverts on failure.
        let message = self.signed_message(
            SIG_DOMAIN_REGISTER,
            &did,
            0,
            Some((&doc_hash, &primary_key)),
        );
        self.crypto().verify_ed25519(
            primary_key.as_managed_buffer(),
            &message,
            signature.as_managed_buffer(),
        );

        let now = self.blockchain().get_block_timestamp();
        self.did_record(&did).set(DidRecord {
            doc_hash: doc_hash.clone(),
            primary_key,
            created_at: now,
            updated_at: now,
            nonce: 0,
            deactivated: false,
        });
        self.did_registered_event(&did, &doc_hash);
    }

    /// Resolve a DID to its current record; panics for an unknown DID.
    /// A deactivated DID resolves to its tombstone record (`deactivated == true`)
    /// — per W3C DID Core, deactivation is surfaced in DID-document metadata and
    /// the record is retained (never deleted) so the DID stays resolvable.
    #[view(resolveDid)]
    fn resolve_did(&self, did: ManagedAddress) -> DidRecord<Self::Api> {
        require!(!self.did_record(&did).is_empty(), "unknown DID");
        self.did_record(&did).get()
    }

    /// Patch the DID-Document hash and rotate the `#klv-1` key, under proof of
    /// control of the *current* primary key. `signature` must be that key's
    /// Ed25519 signature over
    /// `0x01 || sc_address || did || new_doc_hash || new_primary_key || nonce(BE u64)`,
    /// where `nonce` is the record's current nonce.
    #[endpoint(updateDid)]
    fn update_did(
        &self,
        new_doc_hash: ManagedByteArray<Self::Api, 32>,
        new_primary_key: ManagedByteArray<Self::Api, 32>,
        signature: ManagedByteArray<Self::Api, 64>,
    ) {
        let did = self.blockchain().get_caller();
        require!(!self.did_record(&did).is_empty(), "unknown DID");
        let mut record = self.did_record(&did).get();
        require!(!record.deactivated, "DID deactivated");

        let message = self.signed_message(
            SIG_DOMAIN_UPDATE,
            &did,
            record.nonce,
            Some((&new_doc_hash, &new_primary_key)),
        );
        // Panics (reverts the tx) if the signature is not valid for the current
        // primary key — proof of `#klv-1` key control.
        self.crypto().verify_ed25519(
            record.primary_key.as_managed_buffer(),
            &message,
            signature.as_managed_buffer(),
        );

        let now = self.blockchain().get_block_timestamp();
        self.key_history(&did).push(&KeyRotation {
            previous_key: record.primary_key,
            rotated_at: now,
        });
        record.doc_hash = new_doc_hash.clone();
        record.primary_key = new_primary_key;
        record.updated_at = now;
        require!(record.nonce < u64::MAX, "nonce overflow");
        record.nonce += 1;
        self.did_record(&did).set(&record);
        self.did_updated_event(&did, &new_doc_hash, record.nonce);
    }

    /// Deactivate (tombstone) the caller's DID, under proof of control of the
    /// current primary key. `signature` must sign
    /// `0x02 || sc_address || did || nonce(BE u64)`.
    #[endpoint(deactivateDid)]
    fn deactivate_did(&self, signature: ManagedByteArray<Self::Api, 64>) {
        let did = self.blockchain().get_caller();
        require!(!self.did_record(&did).is_empty(), "unknown DID");
        let mut record = self.did_record(&did).get();
        require!(!record.deactivated, "DID already deactivated");

        let message = self.signed_message(SIG_DOMAIN_DEACTIVATE, &did, record.nonce, None);
        self.crypto().verify_ed25519(
            record.primary_key.as_managed_buffer(),
            &message,
            signature.as_managed_buffer(),
        );

        record.deactivated = true;
        record.updated_at = self.blockchain().get_block_timestamp();
        require!(record.nonce < u64::MAX, "nonce overflow");
        record.nonce += 1;
        self.did_record(&did).set(&record);
        self.did_deactivated_event(&did, record.nonce);
    }

    // ---- helpers ----

    /// Build the canonical, domain-separated message that a state-changing
    /// operation must be signed over. Layout:
    /// `domain(1) || sc_address(32) || did(32) [|| doc_hash(32) || primary_key(32)] || nonce(BE 8)`.
    ///
    /// The contract's own address is bound in so a signature is valid only at
    /// the deployment it was produced for — a signature captured on one instance
    /// (e.g. testnet) cannot be replayed on another (e.g. mainnet) even if the
    /// same account, key and nonce exist there.
    fn signed_message(
        &self,
        domain: u8,
        did: &ManagedAddress,
        nonce: u64,
        doc_and_key: Option<(
            &ManagedByteArray<Self::Api, 32>,
            &ManagedByteArray<Self::Api, 32>,
        )>,
    ) -> ManagedBuffer {
        let mut msg = ManagedBuffer::new();
        msg.append_bytes(&[domain]);
        let sc_address = self.blockchain().get_sc_address();
        msg.append(sc_address.as_managed_buffer());
        msg.append(did.as_managed_buffer());
        if let Some((doc_hash, primary_key)) = doc_and_key {
            msg.append(doc_hash.as_managed_buffer());
            msg.append(primary_key.as_managed_buffer());
        }
        msg.append_bytes(&nonce.to_be_bytes());
        msg
    }
}
