# WARNING: This is alpha quality software and not suitable for production. It is incomplete and will have bugs.

# Ledgeracio LEAF

Ledgeracio is a command-line tool and Ledger app designed for staking operations on Substrate-based networks.  It allows performing the following operations:

- `ledgeracio stash`: Stash operations
    - `ledgeracio stash nominate`: Nominate a new validator set.
    - `ledgeracio stash set-payee`: Set the payment target for validation rewards.

- `ledgeracio validator`: Validator operations
    - `ledgeracio validator announce`: Announce intention to validate.
    - `ledgeracio validator replace-key`: Rotate a session key.

Running `ledgeracio --help` will provide top-level usage instructions.

Ledgeracio LEAF is intended to work with a special Ledgeracio Ledger app, but can work with stock Kusama or Polkadot apps as well.  This is less secure, however, as these apps do not enforce the same restrictions that the Ledgeracio app does.

Ledgeracio can also use a software keystore.  To use it, pass `--secret-file FILENAME` as a command-line argument.  This is less secure and should not be used in production.  However, it is quite useful for development and testing.
