# WARNING: This is alpha quality software and not suitable for production. It is incomplete and will have bugs.

# Ledgeracio CLI

Ledgeracio is a command-line tool and a Ledger app designed for staking operations
on Substrate-based networks.

Running `ledgeracio --help` will provide top-level usage instructions.

Ledgeracio CLI is intended to work with a special Ledgeracio Ledger app, but
most of its commands will work with stock Kusama or Polkadot Ledger apps as well.
This is less secure, however, as these apps do not enforce the same restrictions
that the Ledgeracio app does.  Using a stock app in production is not
recommended.

The Polkadot app can be found [here](https://github.com/zondax/ledger-polkadot)
and the Kusama app can be found [here](https://github.com/zondax/ledger-kusama).
Other Substrate-based chains are currently not supported, but local devnets
should work as long as their RPC API matches Kusama/Polkadot's.

Ledgeracio only supports Unix-like systems, and has mostly been tested on Linux.
That said, it works on macOS and other Unix-like systems that provide the
necessary support for userspace USB drivers.

## What is Ledgeracio?

Ledgeracio is a CLI app to perform various tasks common to staking on Kusama and
Polkadot, aka staking-ops.  Ledgeracio is designed to reduce the risk of user
error by way of an allowlist of validators that is set up and signed once and
stored on the Ledger device. Furthermore, Ledgeracio can speed up the workflow
considerably when compared to alternatives using Parity Signer + Polkadot{.js}.

This repository only contains the CLI.  To submit transactions with Ledgeracio,
you will also need the companion Ledger app that you can install from the Ledger app store for [Polkadot](https://support.ledger.com/hc/en-us/articles/360016289919) and [Kusama](https://support.ledger.com/hc/en-us/articles/360016289979-Kusama-KSM-).  Development versions of the apps are available at [Zondax/ledger-polkadot](https://github.com/Zondax/ledger-polkadot)
and [Zondax/ledger-kusama](https://github.com/Zondax/ledger-kusama).  Please do
not use the unaudited versions in production.  For instruction on how to setup and use your Ledger device with Polkadot/Kusama, see the [Polkadot wiki](https://wiki.polkadot.network/docs/en/learn-ledger).

The Ledgeracio CLI contains two binaries.  The first, simply called
`ledgeracio`, is used to submit transactions.  The second, called
`ledgeracio-allowlist`, is used to manage the Ledgeracio Ledger app’s list of
allowed stash accounts.  Generally, one will use `ledgeracio` for normal
operations, and only use `ledgeracio-allowlist` when the list of allowed stash
accounts must be changed.  `ledgeracio` does not handle sensitive data, so it
can safely be used on virtually any machine on which it will run.
Some subcommands of `ledgeracio-allowlist`, however, generate and use secret
keys, which are stored unencrypted on disk.  Therefore, they MUST NOT be used
except on trusted and secured machines.  Ideally, these subcommands should be
run on a machine that is reserved for provisioning of Ledger devices with the Ledgeracio app, and which has no network connectivity.

The allowlist serves to prevent one from accidentally nominating the wrong
validator, which could result in a slash.  It does NOT protect against malicious
use of the device.  Anyone with both the device and its PIN can uninstall the
Ledgeracio app and install the standard Polkadot or Kusama app, which uses the
same derivation path and thus can perform the same transactions.

## Conventions

- An *index* is an integer, at least 1, specified in decimal.  Indexes are used
  to determine which [BIP44](https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki)
  derivation path to use.
- Subcommands that take a single argument take it directly.  Subcommands that
  take multiple arguments use keyword arguments, which are passed as
  `--key value` or `--key=value`.  This avoids needing to memorize the order of
  arguments.
- All commands require that a network name be passed as the first argument.  You
  might want to make a shell alias for this, such as

  ```sh
  alias 'ledgeracio-polkadot=ledgeracio --network polkadot'
  alias 'ledgeracio-kusama=ledgeracio --network kusama'
  ```

## Getting Started

### Allowlist signing

Provisioning the Ledgeracio Ledger app requires a trusted computer.  This
computer will store the secret key used to sign allowlists.  This computer does
not need network access, and generally should not have it.
`ledgeracio-allowlist` does not encrypt the secret key, so operations that
involve secret keys should only be done on machines that use encrypted storage.

Only devices used for nomination need to be provisioned.  However, if you only
intend to use the app for validator management, you should set an empty
allowlist, which blocks all nominator operations.

First, `ledgeracio-allowlist gen-key <file>` is used to generate a secret key.
The public part will be placed in `<file>.pub` and the secret part in
`<file>.sec`.  Both will be created with 0400 permissions, so that they are not
accidentally overwritten or exposed.  This operation requires a trusted
computer.  The public key file can be freely redistributed, while the secret key
file should never leave the machine it was generated on.

You can now sign a textual allowlist file with `ledgeracio-allowlist sign`.  A
textual allowlist file has one SS58 address per line.  Leading and trailing
whitespace is stripped.  If the first non-whitespace character on a line is `#`
or `;`, or if the line is empty or consists entirely of whitespace, it is
considered to be a comment and ignored.

`ledgeracio-allowlist sign` is invoked as follows:

```
ledgeracio-allowlist --network <network> sign --file <file> --nonce <nonce> --output <output> --secret <secret>
```

`<file>` is the allowlist file.  `<nonce>` is the nonce, which is incorporated
into the signed allowlist file named `<output>`.  Ledgeracio apps keep track of
the nonce of the most recent allowlist uploaded, and reject new uploads unless
the new allowlist has a nonce higher than the old one.  Nonces do not need to be
contiguous, so skipping a nonce is okay.  Signed allowlists are stored in a
binary format.

### Device provisioning

`ledgeracio-allowlist` is also used for device provisioning.  To set the
allowlist, use `ledgeracio-allowlist set-key`.  This command will only
succeed once.  If an allowlist has already been uploaded, it will fail.  The
only way to change the allowlist signing key is to reinstall the Ledgeracio app,
which does not result in any funds being lost.

`ledgeracio-allowlist upload` is used to upload an allowlist.  The uploaded
allowlist must have a nonce that is greater than the nonce of the previous
allowlist.  If there was no previous allowlist, any nonce is allowed.

To verify the signature of a binary allowlist file, use
`ledgeracio-allowlist inspect`.  This also displays the allowlist on stdout.

### Ledgeracio Use

`ledgeracio` is used for staking operations.  Before accounts on a Ledger device
can be used for staking, they must be chosen as a controller account.  You can
obtain the address by running `ledgeracio <validator|nominator> address`.  The
address can be directly pasted into a GUI tool, such as Polkadot{.js}.

`ledgeracio nominator nominate` is used to nominate an approved validator,
and `ledgeracio validator announce` is used to announce intention to validate.
`ledgeracio [nominator|validator] set-payee` is used to set the payment target.
`ledgeracio [nominator|validator] chill` is used to stop staking, while
`ledgeracio [nominator|validator] show` and
`ledgeracio [nominator|validator] show-address` are used to display staking
status.  The first takes an index, while the second takes an address.
`show-address` does not require a Ledger device.
`ledgeracio validator replace-key` is used to set a validator’s session key.

## Subcommand Reference

### Allowlist handling: `ledgeracio-allowlist`

The Ledgeracio app enforces a list of allowed stash accounts.  This is
managed using the `ledgeracio-allowlist` command.

Some subcommands involve the generation or use of secret keys, which are stored
on disk without encryption.  These subcommands MUST NOT be used on untrusted
machines.  Ideally, they should be run on a machine that is reserved for
provisioning of Ledgeracio apps, and which has no access to the Internet.

#### Key generation: `ledgeracio-allowlist gen-key`

This command takes one argument: the basename (filename without extension) of
the keys to generate.  The public key will be given the extension `.pub` and the
secret key the extension `.sec`.  The files will be generated with 0400
permissions, which means that they can only be read by the current user and the
system administrator, and they cannot be written to except by the administrator.
This is to prevent accidental overwrites.

The public key is not sensitive, and is required by anyone who wishes to verify
signed allowlists and operate on the allowed accounts.  It will be uploaded
to the Ledger device by `ledgeracio-allowlist set-key`.  The secret key allows
generating signatures, and therefore must be kept secret.  It should never leave
the (preferably air gapped) machine it is generated on.

#### Uploading an allowlist signing key to a device: `ledgeracio-allowlist set-key`

This command takes one argument, the name of the public key file (including
extension).  The key will be parsed and uploaded to the Ledgeracio app running
on the attached Ledger device.  If it is not able to do so, Ledgeracio will
print an error message and exit with a non-zero status.

If a key has already been uploaded, uploading a new key will fail.  The only
workaround is to reinstall the Ledgeracio app.  This *does not* forfeit any
funds stored on the device. We strongly recommend users to use separate Ledger
devices for ledgeracio and cold storage.

The user will be required to confirm the upload via the Ledger UI.  This allows
the user to check that the correct key has been uploaded, instead of a key
chosen by an attacker who has compromised the user’s machine.

#### Retrieving the uploaded key: `ledgeracio-allowlist get-key`

This command takes no arguments.  The public key that has been uploaded will be
retrieved and printed to stdout.  If no public key has been uploaded, or if the
app is not the Ledgeracio app, an error will be returned.

#### Signing an allowlist: `ledgeracio-allowlist sign`

This command takes the following arguments.  All of them are mandatory.

- `--file <file>`: the textual allowlist file to sign.  See
  [FORMATS.md](FORMATS.md) for its format.
- `--nonce <nonce>`: The nonce to sign the file with.  The nonce must be greater
  than the previous nonce, or the Ledgeracio app will reject the allowlist.
- `--output <output>`: The name of the output file to write.
- `--secret <secret>`: The name of the secret key file.

#### Inspecting a signed allowlist: `ledgeracio-allowlist inspect`

This command takes two arguments.  Both of them are mandatory.

- `--file <file>`: The name of the signed allowlist to inspect.
- `--public <public>`: The name of the public key file that signed the
  allowlist.  This command will fail if the signature cannot be verified.

#### Uploading an allowlist: `ledgeracio-allowlist upload`

This command takes one argument: the filename of the signed binary allowlist to
upload.  The command will fail if any of the following occurs:

- There is no Ledger device connected.
- The attached device is not running the Ledgeracio app.
- The Ledgeracio app refuses the operation.

The Ledgeracio app will refuse the operation if:

- No signing key has been uploaded.
- The allowlist has not been signed by the public key stored in the app.
- The nonce is not greater than that of the previously uploaded allowlist.  If
  no allowlist has been previously uploaded, any nonce is allowed.
- The user refuses the operation.

### Metadata inspection: `ledgeracio metadata`

This command takes no arguments.  It pretty-prints the chain metadata to stdout.
It is primarily intended for debugging.  Requires a network connection.

### Properties inspection: `ledgeracio properties`

This command takes no arguments.  It pretty-prints the chain properties to
stdout.  It is primarily intended for debugging.  Requires a network connection.

### Nominator operations: `ledgeracio nominator`

This command performs operations using nominator keys ― that is, keys on a
nominator derivation path.  Requires a network connection.  The following
subcommands are available:

#### Displaying the address at an index: `ledgeracio nominator address`

This command takes an index as a parameter.  The address on the device
corresponding to that index is displayed on stdout.

#### Showing a nominator controller: `ledgeracio nominator show`

This command takes an index as parameter, and displays information about the
corresponding nominator controller account.

#### Showing a nominator controller address: `ledgeracio nominator show-address`

This command takes an SS58-formatted address as parameter, and displays
information about the corresponding nominator controller account.  It does not
require a Ledger device.

#### Nominating a new validator set: `ledgeracio nominator nominate`

This command takes a index followed by a list of SS58-formatted addresses.
It uses the account at the provided index to nominate the provided validator
stash accounts.

The user must confirm this action on the Ledger device.  For security reasons,
users ***MUST*** confirm that the addresses displayed on the device are the
intended ones.  A compromised host machine can send a set of accounts that is
not the ones the user intended.  If any of the addresses sent to the device are
not on the allowlist, the transaction will not be signed.

#### Stopping nomination: `ledgeracio nominator chill`

This command stops the account at the provided index from nominating.

The user must confirm this action on the Ledger device.

#### Setting a payment target: `ledgeracio nominator set-payee`

This command takes an index as argument, and sets the payment target.  The
target must be one of `Stash`, `Staked`, or `Controller` (case-insensitive).

### Validator operations: `ledgeracio validator`

This command handles validator operations.  It requires a network connection, and
has the following subcommands:

#### Displaying a validator address: `ledgeracio validator address <index>`

This command displays the address of the validator controller account at the
given index.

#### Announcing an intention to validate: `ledgeracio validator announce <index> [commission]`

This command announces that the controller account at `<index>` intends to
validate.  An optional commission may also be provided.  If none is supplied, it
defaults to 100%.
FIXME: document the format of the comission. Is it "13" for 13% commission? Or "0.13"?

#### Cease validation: `ledgeracio validator chill`

This command stops validation.

The user must confirm this action on the Ledger device.

#### Setting the payment target: `ledgeracio validator set-payee`

This command is the validator version of `ledgeracio nominator set-payee`.  See
its documentation for details.

#### Displaying information on a given validator: `ledgeracio validator show`

This command is the validator version of `ledgeracio nominator show`.  See
its documentation for details.

#### Displaying information on a given validator address: `ledgeracio validator show-address`

This command is the validator version of `ledgeracio nominator show-address`.
See its documentation for details.

#### Rotating a session key: `ledgeracio validator replace-key <index> <keys>`

This command sets the session keys of the validator controlled by the account at
`<index>`.  The keys must be in hexidecimal, as returned by the key rotation RPC
call.
