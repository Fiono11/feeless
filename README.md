# ⋰·⋰ Feeless

## What is Feeless?

**Feeless** is a **Nano** cryptocurrency node, wallet, tools, and Rust crate. This is not the official project for Nano,
only an implementation written in Rust. The official Nano node
implementation [lives here](https://github.com/nanocurrency/nano-node).

⚠ This is a work in progress. It's not ready to use as a real node! ⚠

I decided to start this project as a personal adventure of understanding Nano. I give no promises about my future
motivation to complete this project 🤐.

## What is Nano?

**Nano** is digital money that significantly improves on **Bitcoin** and other cryptocurrencies.

The main features of **Nano** are:

* No transaction fees.
* Extremely fast to send money--less than 1 second for 100% confirmation.

  <sup>Bitcoin takes 10 minutes on average for ~80%<sup>1</sup> confirmation.</sup>
* Highly decentralized

  <sup>(IIUC) Using the Nakamoto coeffieint measurement, it is more decentralized than Bitcoin<sup>2 3</sup>.
* No inflation.
* Green--Massively less energy use than Bitcoin.

For more information on what Nano is, see the Nano documentation: https://docs.nano.org/what-is-nano/overview/

Nano is also known as: Nano cryptocurrency, Nano coin, Rai Blocks.

<sup>
1. The Bitcoin white paper, under section 11 "Calculations" explains there's a ~80% chance for an attacker with 10% mining power to overtake the longest chain. https://bitcoin.org/bitcoin.pdf
2. Measuring Decentralization in Bitcoin and Ethereum using Multiple Metrics and Granularities https://arxiv.org/pdf/2101.10699.pdf
3. List of representative nodes showing a Nakamoto coefficient of 8 at the time of writing (2021-02) https://nanocharts.info/

</sup>

## Goals

### General

* Correctness before performance.

### Rust crate

* A complete library that a Rust developer can use to handle wallets, keys, blocks, signing, proof of work, etc.

### Tools

* A command line tool for particular actions, e.g. generating seeds, conversions between keys, addresses, etc.
* A command line client for the official Nano RPC server.

### Nano node

* A functional Nano node with business logic from the official C++ implementation.
* Correct rebroadcasting rules
* Representative voting
* Bootstrapping
* It has to perform well enough to help the network. I don't want Nano to slow down if people start using this! 🤦‍♀️

## Non-goals

* Only support protocol version 18+
* No UDP support
* No user interface

## Task list

A medium term task list:

- [x] Seeds
  - [x] Mnemonic (word list) seed generation/parsing (BIP39)
  - [x] Derive keys from mnemonic (BIP33)
  - [x] Hex seeds
- [x] Keys (ed25519/blake2b)
  - [x] Private keys
  - [x] Public keys
  - [x] Nano addresses
    - [x] Validation
    - [x] Parsing
    - [x] Conversion to/from public keys
- [x] Nano amount conversions
  - [x] raw
  - [x] nano
  - [x] Mnano/NANO
- [x] Proof of work (core)
  - [x] Verification against a threshold
  - [x] Generation
  - [x] Dynamic threshold
- [ ] Blocks
  - [x] Hashing
  - [ ] Work
  - [x] State blocks
  - [ ] <v18 blocks?
- [ ] Packet dissector
  - [x] Parse hex dump from Wireshark
  - [ ] Parse pcap
  - [x] Dump some message types to console
- [ ] Node
  - [ ] Configuration
    - [x] Initial command line interface
    - [ ] Network
    - [ ] Database
    - [ ] ...
  - [ ] Networks
    - [x] Live (Don't worry, I'm only connecting to my own node at the moment!)
    - [ ] Test
    - [ ] Beta
  - [ ] Bootstrap peer connection (peering.nano.org)
  - [x] Validate given peer network
  - [ ] Validate given peer versions
  - [ ] Multiple peer connectivity (currently only connects to one peer)
    - [ ] Configurable maximum peer limit
  - [x] Header parsing
    - [x] Network
    - [x] Versions
    - [x] Extensions
      - [x] Handshake query/response flags
      - [x] Count
      - [x] Block type
      - [ ] Telemetry size
      - [ ] Extended params present
  - [ ] Logic
    - [ ] Rebroadcasting
    - [ ] Representatives
    - [ ] Publish retries (difficulty changes)
    - [ ] ...
  - [ ] Messages
    - [ ] Node ID Handshake
      - [x] Serialize (TODO: needs small refactor)
      - [x] Deserialize
      - [x] Send cookie
      - [ ] Cookie/peer store and logic
      - [x] Validate response
    - [ ] Confirm Req
      - [ ] Serialize
      - [ ] Deserialize
        - [x] Hash pairs
        - [ ] Block selector
      - [ ] Handle response
    - [ ] Confirm Ack
      - [ ] Serialize
      - [ ] Deserialize
        - [x] Vote by hash
        - [ ] Block
    - [ ] Keepalive
      - [ ] Serialize
      - [x] Deserialize
    - [ ] Publish
      - [ ] Serialize
      - [x] Deserialize
        - [x] State blocks
        - [ ] Other blocks
    - [ ] Bulk pull
    - [ ] Bulk pull account
    - [ ] Bulk pull blocks
    - [ ] Bulk push
    - [ ] Telemetry Req
      - [ ] Serialize
      - [x] Deserialize
      - [ ] Collect telemetry
      - [ ] Handle response
    - [ ] Telemetry Ack
    - [ ] Frontier Req
  - [ ] Storage
    - [x] Basic KV store to file
    - [x] Basic cookie/peer storage
    - [ ] Peers
    - [ ] Blocks
    - [ ] ...
  - [ ] RPC
- [ ] Rust
  - [ ] Ask around for a code review
  - [ ] Use either `zerocopy` or make all core types zero-copy with storing `[u8]` and methods as accessors. `zerocopy`
    did work for most things when I tried but had problems with enums. Might revisit.
  - [ ] Use `thiserror` instead of `anyhow`
  - [ ] Github actions CI (including `cargo clippy`)
- [ ] Future things
  - [ ] Performance
    - [ ] Automated comparison
  - [ ] Proof of work
    - [ ] Server
    - [ ] GPU
  - [ ] WASM

## Credits and references

* Thanks to the hard work from the Nano Foundation.
* https://github.com/nanocurrency/nano-node
  * The actual Nano implementation as a source of truth.
* https://forum.nano.org/, https://old.reddit.com/r/nanocurrency/, Nano Discord: https://chat.nano.org/
  * A very friendly community helping out others and myself.
* https://docs.nano.org/
  * General useful information.
* https://nanoo.tools/
  * Helped me understand technical details on state blocks and hashing, and also with validating conversions between
    things.
* https://github.com/nanocurrency/protocol/blob/master/reference
  * Node protocol specification.
* https://iancoleman.io/bip39/
  * Helped me test out my BIP 39/BIP 44 implementations.

## Licence

This project is licenced under both MIT and Apache 2.0.