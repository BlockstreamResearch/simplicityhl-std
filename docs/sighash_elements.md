# Elements sighash modes

> **Warning: unaudited.** This module and this specification have not been audited. The
> signatures they define guard funds, and the message format is permanent for any coins locked
> with it. Use them with caution, read [What a signature does not cover](#what-a-signature-does-not-cover)
> before designing a contract, and verify signatures against the test vectors before relying on
> them.

This document specifies the messages signed by the `sighash_elements` module
(`simf/lib/sighash_elements.simf`), byte for byte, so that a wallet or signer can produce
signatures for it without running the contract.

A contract that calls `bip_0340_verify_with_mode(pk, sig, mode)` accepts `sig` if it is a valid
[BIP-340](https://github.com/bitcoin/bips/blob/master/bip-0340.mediawiki) signature by `pk` over
the 32-byte message `sighash(mode)` described here. The message is signed as is, with no further
hashing or tagging. The signature is always 64 bytes: the mode is a separate value that the
contract takes from its witness, not a byte appended to the signature as in BIP-341.

ALL is not new: it is Simplicity's standard signature message, `jet::sig_all_hash`, unchanged.
A wallet that already signs Simplicity spends (the simplex SDK signer, for example) produces ALL
signatures for this module as it is. This document defines the other five modes, which are
specific to this module.

## Notation

- `SHA256(x)` is a single SHA-256 of the byte string `x`. Every hash below is a single SHA-256,
  except Elements' own derivation of issued asset and token IDs.
- `‖` is concatenation.
- `u32be(n)` and `u64be(n)` are big-endian encodings of 4 and 8 bytes.
- Txids, asset IDs, block hashes, contract hashes, issuance entropies and blinding nonces are used
  in their internal byte order: the order they have in a serialized transaction or block header,
  which is the reverse of their usual hex display.
- CMRs, x-only public keys and the tags below are used in the order they are usually displayed.
- `H(b)` is `SHA256(b)` of a byte string `b`, with no length prefix. For an absent value `b` is
  empty, so `H(empty) = e3b0c442…b855`.

The signed message and each mode hash preimage have fixed sizes. The component hashes do not,
since explicit and confidential values have different lengths.

## Mode byte

The mode is a BIP-341 `hash_type` byte.

| Byte | Mode | Message |
|---|---|---|
| `0x01` | ALL | `jet::sig_all_hash` (see [ALL](#all)) |
| `0x02` | NONE | [tagged](#signed-message) |
| `0x03` | SINGLE | tagged |
| `0x81` | ALL\|ANYONECANPAY | tagged |
| `0x82` | NONE\|ANYONECANPAY | tagged |
| `0x83` | SINGLE\|ANYONECANPAY | tagged |

Any other byte, including `0x00`, makes the spend invalid. `0x00` is rejected so that each
signature has exactly one valid mode byte.

## ALL

ALL is `jet::sig_all_hash`, defined by Simplicity, not by this module. It is restated here for
reference; Simplicity's definition is authoritative.

```
SHA256(genesis ‖ genesis ‖ tx_hash ‖ tap_env ‖ u32be(index))
```

where `tx_hash` is
`SHA256(u32be(version) ‖ u32be(lock_time) ‖ inputs_hash ‖ outputs_hash ‖ issuances_hash ‖ output_surjection_proofs_hash ‖ input_utxos_hash)`.
The components are defined [below](#components).

## Signed message

For every other mode the signed message is

```
SHA256(SIGHASH_TAG ‖ genesis ‖ mode_hash ‖ tap_env ‖ u32be(index))    NONE, SINGLE           (132 bytes)
SHA256(SIGHASH_TAG ‖ genesis ‖ mode_hash ‖ tap_env)                   ANYONECANPAY modes     (128 bytes)
```

- `SIGHASH_TAG = SHA256("simplicityhl-std\x1fSighash")`
  `= c8b0db060d9bc330f234d3f17438816834d7b0b3819ba2e9e99b54a5e062a12f`.
  `\x1f` is the ASCII unit separator. `SIGHASH_TAG ‖ genesis` is exactly one 64-byte SHA-256
  block, so a signer can precompute that midstate once per chain.
- `genesis` is the genesis block hash of the chain the spend is for.
- `mode_hash` is defined in [Mode hashes](#mode-hashes).
- `tap_env` is defined in [Tapleaf environment](#tapleaf-environment). It identifies the contract
  being spent.
- `index` is the position of the input being signed. The ANYONECANPAY modes leave it out, so the
  input may end up at any position.

## Mode hashes

Each mode hash is a tagged hash, in the BIP-340 style, of the transaction version, lock time,
and the parts of the transaction the mode signs:

```
mode_hash = SHA256(TAG ‖ TAG ‖ u32be(version) ‖ u32be(lock_time) ‖ fields)
```

`version` and `lock_time` are the transaction's raw `nVersion` and `nLockTime` fields.

| Mode | `TAG` = SHA256 of | `fields` | Size |
|---|---|---|---|
| NONE | `simplicityhl-std\x1fSighash\x1fnone` | `inputs_hash ‖ input_utxos_hash ‖ issuances_hash` | 168 |
| SINGLE | `simplicityhl-std\x1fSighash\x1fsingle` | `inputs_hash ‖ input_utxos_hash ‖ issuances_hash ‖ output_hash(index) ‖ output_surjection_proof(index)` | 232 |
| ALL\|ANYONECANPAY | `simplicityhl-std\x1fSighash\x1fall_anyonecanpay` | `input_hash(index) ‖ input_utxo_hash(index) ‖ issuance_hash(index) ‖ outputs_hash` | 200 |
| NONE\|ANYONECANPAY | `simplicityhl-std\x1fSighash\x1fnone_anyonecanpay` | `input_hash(index) ‖ input_utxo_hash(index) ‖ issuance_hash(index)` | 168 |
| SINGLE\|ANYONECANPAY | `simplicityhl-std\x1fSighash\x1fsingle_anyonecanpay` | `input_hash(index) ‖ input_utxo_hash(index) ‖ issuance_hash(index) ‖ output_hash(index)` | 200 |

The tag values are:

| Mode | `TAG` |
|---|---|
| NONE | `901263cfeb7f58d0db0814d66e05911495cefae6cd3fe4d5300abac5dc75ceff` |
| SINGLE | `6598255e3bf6290815c342018df4159dead0e953fcf39df15b4ec85bd82edfba` |
| ALL\|ANYONECANPAY | `c58ee484f801263d0eb40aa98a131125908368fde9e323b200c9a650700b41ac` |
| NONE\|ANYONECANPAY | `899df532df6d769b5e0bfb155ac805ef84011fbf6f7aa3d3cbe1119055f0d6bc` |
| SINGLE\|ANYONECANPAY | `cfe3e884cd9186cbc582383e1fd65e577dd6d32ba58d544150d794522332d7bf` |

SINGLE and SINGLE|ANYONECANPAY are invalid when the transaction has no output at `index`.

Surjection proofs are signed only by SINGLE (and by ALL). A surjection proof depends on the whole
input set, so in the ANYONECANPAY modes adding an input would mean regenerating it.

Under SINGLE|ANYONECANPAY the input may move, but the output it signs must be at the input's new
index.

## Components

These are the values of the Simplicity jets of the same names.

### Value encodings

Assets, amounts and nonces are encoded exactly as in a serialized Elements transaction:

| Value | Explicit | Confidential | Null |
|---|---|---|---|
| `asset(a)` | `0x01 ‖ asset ID` (33) | `0x0a` or `0x0b` `‖ x` (33) | `0x00` (1) |
| `amount(v)` | `0x01 ‖ u64be(v)` (9) | `0x08` or `0x09` `‖ x` (33) | as explicit 0: `0x01 ‖ u64be(0)` (9) |
| `nonce(n)` | `0x01 ‖ n` (33) | `0x02` or `0x03` `‖ x` (33) | `0x00` (1) |

For a confidential value the encoding is its serialized 33 bytes, prefix included. A null amount
is never encoded as `0x00`: it is treated as an explicit 0.

### Current input

For the input at position `i`:

```
input_hash(i)      = SHA256(pegin ‖ txid ‖ u32be(vout) ‖ u32be(sequence) ‖ annex)
input_utxo_hash(i) = SHA256(asset(utxo asset) ‖ amount(utxo amount) ‖ H(utxo scriptPubKey))
issuance_hash(i)   = SHA256(issued_asset ‖ issued_tokens ‖ H(asset range proof) ‖ H(token range proof) ‖ entropy_data)
```

- `pegin` is `0x00`, or for a peg-in `0x01 ‖ parent chain genesis hash` (33 bytes). The genesis
  hash is the 32 bytes of the peg-in witness's third item (`pegin_witness[2]`), as they appear
  there.
- `txid` and `vout` are the input's previous outpoint. `vout` is the output index with Elements'
  issuance and peg-in flag bits (`1 << 31` and `1 << 30`) cleared.
- `annex` is `0x00` if the input has no annex, otherwise `0x01 ‖ H(annex)`, where `annex` is the
  last witness item *without* its leading `0x50` byte.

  An input has an annex if and only if its script witness has at least two items and the last
  item is non-empty and starts with `0x50`. This applies to every input, whatever its script
  type. Note that this hash differs from BIP-341's `sha_annex`, which includes the `0x50` byte
  and a length prefix.
- `utxo` is the output being spent: its asset, amount and scriptPubKey. For a peg-in it is the
  output Elements derives from the peg-in witness: explicit asset `pegin_witness[1]`, explicit
  amount `pegin_witness[0]` (stored little-endian in the witness, encoded here as `u64be`), and
  the claim script `pegin_witness[3]` as scriptPubKey.

`issuance_hash(i)` for an input with no issuance is

```
SHA256(0x00 ‖ 0x00 ‖ 0x00 ‖ 0x00 ‖ H(empty) ‖ H(empty) ‖ 0x00)
```

and for an input with an issuance:

| Part | New issuance | Reissuance |
|---|---|---|
| `issued_asset` | `asset(explicit asset ID) ‖ amount(issued amount)` | same |
| `issued_tokens` | `asset(explicit token ID) ‖ amount(issued token amount)` | `asset(explicit token ID) ‖ amount(explicit 0)` |
| asset range proof | the issuance amount's range proof if the amount is confidential, otherwise empty | same |
| token range proof | the token amount's range proof if the amount is confidential, otherwise empty | empty |
| `entropy_data` | `0x01 ‖ 32 zero bytes ‖ contract hash` | `0x01 ‖ blinding nonce ‖ entropy` |

An input has an issuance if its issued amount or its token amount is non-null. An issuance is a
reissuance if its blinding nonce is non-zero; for a reissuance, `entropy` is the issuance's
entropy field. An absent issued amount or token amount counts as explicit 0.

The asset and token IDs are Elements' usual issuance IDs, derived from the issuance entropy as in
Elements' `issuance.cpp`. The token ID depends on whether the *issued asset amount* is
confidential, for reissuances as well as new issuances.

### All inputs

These hash the same data for every input, in input order, grouped by field.

```
inputs_hash      = SHA256(outpoints ‖ sequences ‖ annexes)
input_utxos_hash = SHA256(utxo_amounts ‖ utxo_scripts)
issuances_hash   = SHA256(issuance_amounts ‖ issuance_tokens ‖ issuance_range_proofs ‖ issuance_entropies)
```

Each item on the right is `SHA256` of the concatenation, over all inputs, of:

| Item | Per input |
|---|---|
| `outpoints` | `pegin ‖ txid ‖ u32be(vout)` |
| `sequences` | `u32be(sequence)` |
| `annexes` | `annex` |
| `utxo_amounts` | `asset(utxo asset) ‖ amount(utxo amount)` |
| `utxo_scripts` | `H(utxo scriptPubKey)` |
| `issuance_amounts` | `0x00 ‖ 0x00` if no issuance, otherwise `issued_asset` |
| `issuance_tokens` | `0x00 ‖ 0x00` if no issuance, otherwise `issued_tokens` |
| `issuance_range_proofs` | `H(asset range proof) ‖ H(token range proof)` |
| `issuance_entropies` | `0x00` if no issuance, otherwise `entropy_data` |

`pegin`, `annex`, `issued_asset`, `issued_tokens`, `entropy_data` and the range proofs are as in
[Current input](#current-input).

### Outputs

For the output at position `i`:

```
output_hash(i)             = SHA256(asset(asset) ‖ amount(amount) ‖ nonce(nonce) ‖ H(scriptPubKey) ‖ H(range proof))
output_surjection_proof(i) = H(surjection proof)
```

The range proof counts only if the amount is confidential, and the surjection proof only if the
asset is confidential. Otherwise each is taken as empty, whatever the output carries.

For all outputs, in output order:

```
outputs_hash = SHA256(output_amounts ‖ output_nonces ‖ output_scripts ‖ output_range_proofs)
output_surjection_proofs_hash = SHA256(H(surjection proof) of every output)
```

where each item is `SHA256` of, over all outputs, `asset(asset) ‖ amount(amount)`, `nonce(nonce)`,
`H(scriptPubKey)` and `H(range proof)` respectively. Fee outputs are ordinary outputs here.

### Tapleaf environment

`tap_env` identifies the contract and the taproot output being spent:

```
tap_env  = SHA256(tapleaf ‖ tappath ‖ internal_key)
tapleaf  = SHA256(LEAF_TAG ‖ LEAF_TAG ‖ leaf_version ‖ 0x20 ‖ CMR)
tappath  = SHA256(control block path)
```

- `LEAF_TAG = SHA256("TapLeaf/elements")`.
- `leaf_version` is the first byte of the control block with its lowest bit cleared. For a
  Simplicity leaf it is always `0xbe`.
- `CMR` is the 32-byte commitment Merkle root of the Simplicity program.
- `control block path` is the concatenation of the 32-byte hashes that follow the internal key in
  the control block (empty for a single-leaf tree).
- `internal_key` is the 32-byte x-only internal key from the control block.

## Differences from Elements taproot signatures

These modes reuse BIP-341's names and `hash_type` bytes, but not its message format. A standard
PSBT or taproot signer cannot produce them.

- The mode byte is not part of the message. The five custom modes are told apart by their tags,
  and ALL by its different structure.
- `0x00` (SIGHASH_DEFAULT) is not accepted.
- Elements' ALL|ANYONECANPAY and SINGLE|ANYONECANPAY sign the outputs' surjection proofs; these
  modes do not.
- ALL, NONE and SINGLE sign every input's annex. BIP-341 signs only the current input's annex,
  and hashes it differently (see [Current input](#current-input)).
- The message commits to `tap_env` (leaf, path and internal key) rather than to the leaf hash and
  key version. There is no epoch byte, `spend_type`, or `codesep_pos`.
- SINGLE without an output at `index` is invalid, as in BIP-341.

## What a signature does not cover

A signature covers the transaction data listed above and `tap_env`, which fixes the contract (its
CMR), the tapleaf and the internal key. It does not cover the Simplicity witness of the input
being spent: neither the witness values nor the pruned program, and so not which branch of the
contract runs. `sig_all_hash` has the same limit.

So a signature authorises the spend through *any* branch of the contract that accepts this key's
signature in that mode, and whoever completes the transaction chooses the branch and the other
witness values.

With ALL this rarely matters, because the whole transaction is fixed whichever branch runs. With
the narrower modes it can. NONE already lets anyone holding the signature choose the outputs: it
binds the inputs, version and lock time, and nothing about where the value goes. A signer may
think a NONE signature is safe because one branch of the contract constrains the outputs itself,
but if another branch accepts the same key without that constraint, the signature can be used
there instead, with any outputs. NONE|ANYONECANPAY binds only the signed input, so it is the most
permissive of all.

What contract authors can do today:

- Put branches that make different assumptions about signatures in different tapleaves.
  `tap_env` commits to the leaf, so a signature made for one leaf is invalid in another.
- Use a different key in each branch.
- Accept only the modes a branch is safe with, by calling the per-mode functions (for example
  `sighash_single`) instead of `bip_0340_verify_with_mode`.

This module cannot cover the branch itself, because no jet exposes the spending input's own
witness. In Elements the witness of a Simplicity spend is `[witness data, program, CMR, control
block]`. A jet returning the hash of the program item would let a signature commit to the pruned
program, and so to the branch. The witness data item cannot be covered the same way, since it
contains the signature, which would then have to sign itself. Even covering the program needs
care: the pruned program is only known after the program has run, and that run needs the
signature, so a signer would have to prune it assuming the signature check succeeds.

## Relation to Simplicity's `check_sig_verify`

Simplicity has its own design for choosing a sighash at spend time
([SimplicityHL#66](https://github.com/BlockstreamResearch/SimplicityHL/issues/66)). The
`check_sig_verify` jet checks a BIP-340 signature over

```
SHA256(T ‖ T ‖ cmr ‖ output),    T = SHA256("Simplicity\x1fSignature")
```

where `cmr` is the commitment Merkle root of a *hash mode*, any Simplicity expression returning 32
bytes and chosen by the signer at spend time, and `output` is its result. SimplicityHL disables
`check_sig_verify` for now, because it cannot yet enforce that structure.

The two designs can be used side by side:

- **They cannot be confused.** A `check_sig_verify` message is a hash whose first 64-byte block is
  `T ‖ T`. This module's messages start with `SIGHASH_TAG ‖ genesis`, and ALL with
  `genesis ‖ genesis`. Different first blocks mean different messages unless SHA-256 collides, so
  a signature made for one is never valid for the other, even with the same key.
- **Each contract uses one.** The choice is fixed in the contract's CMR, so coins locked with this
  module always use these messages.

They differ in what they offer:

| | This module | `check_sig_verify` |
|---|---|---|
| Modes | Six, fixed | Any expression returning 32 bytes |
| The signature commits to | The mode, through its tag | The hash mode's CMR |
| Computing the message | From this spec | By evaluating the hash mode |
| ALL | Plain `sig_all_hash` | `sig_all_hash` inside the tagged message, so not the same signature |
| Available in SimplicityHL | Now | Not yet |

A wallet has to know which design a contract uses. In particular, an ALL signature for this module
is not valid for a contract that uses `check_sig_verify` with `sig_all_hash` as its hash mode.

## Known implementation issue

rust-simplicity (0.8 and current master) detects an annex whenever the last witness item starts
with `0x50`, without requiring at least two witness items. Elements does require two, and
Elements is consensus. If another input in the transaction is a taproot key-path spend whose
signature happens to start with `0x50` (1 in 256), a signer that computes the message with
rust-simplicity gets a different `inputs_hash` from the network, and its ALL, NONE and SINGLE
signatures are rejected. The ANYONECANPAY modes are unaffected, since they hash only the current
input.

This is fixed by
[rust-simplicity#384](https://github.com/BlockstreamResearch/rust-simplicity/pull/384).

## Test vectors

[`sighash_elements_vectors.json`](sighash_elements_vectors.json) holds the expected messages for
a set of transactions, in all six modes. Each vector gives:

- `tx`: the transaction, consensus-serialized and hex-encoded.
- `utxos`: the outputs its inputs spend, in input order, each consensus-serialized.
- `index`: the position of the input being signed.
- `control_block`, `cmr` and `genesis`: the rest of the context (`genesis` in internal byte
  order).
- `tap_env`: the [tapleaf environment](#tapleaf-environment), as an intermediate value.
- `results`: for each mode, `mode_hash` (null for ALL) and the signed message `sighash` (null
  where the spend is invalid).

| Vector | What it covers |
|---|---|
| `explicit` | Two inputs and two outputs, all explicit. |
| `second_input` | The signed input at index 1; version 3, a height lock time, non-final sequences. |
| `issuances` | A new issuance on one input and a reissuance on the signed input. |
| `annexes` | An annex on both inputs. |
| `one_item_witness_starting_0x50` | A one-item witness starting with `0x50`, which is not an annex. Every message matches `explicit`. |
| `confidential_output` | An output with a confidential asset and amount, a nonce, a range proof and a surjection proof. |
| `no_paired_output` | No output at the signed input's index, so SINGLE and SINGLE\|ANYONECANPAY are invalid. |

The vectors come from an implementation of this document written independently of the module
(`tests/common/sighash_spec.rs`). The tests in `tests/sighash_elements_test.rs` check that:

- every vector can be recomputed from the file alone;
- the file matches what the implementation produces now;
- the implementation and the contract produce the same message for every vector and mode.

The one exception is `one_item_witness_starting_0x50` in ALL, NONE and SINGLE, where the
contract runs on rust-simplicity and disagrees until the fix for the
[known implementation issue](#known-implementation-issue) is released. The test expects that
difference, so it will flag the release.
