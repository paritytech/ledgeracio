# WARNING: This is alpha quality software and not suitable for production. It is incomplete and will have bugs.

# Ledgeracio CLI

Ledgeracio is a command-line tool and Ledger app designed for staking operations
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

Ledgeracio only support Unix-like systems, and has only tested mostly on Linux.
That said, it works on macOS and other Unix-like systems that provide the
necessary support for userspace USB drivers.

## Conventions

- An *index* is an integer, at least 1, specified in decimal.  Indexes are used
  to determine which [BIP44](https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki) derivation path to use.
- Subcommands that take a single argument take it directly.  Subcommands that
  take multiple arguments use keyword arguments, which are passed as
  `--key value` or `--key=value`.  This avoids needing to memorize the order of
  arguments.

## Top-level commands

### Allowlist handling: `ledgeracio allowlist`

The Ledgeracio app enforces a list of allowed stash accounts.  This is
managed using the `ledgeracio allowlist` command.

Some subcommands involve the generation or use of secret keys, which are stored
on disk without encryption.  These subcommands MUST NOT be used on untrusted
machines.  Ideally, they should be run on a machine that is reserved for
provisioning of Ledgeracio apps, and which has no access to the Internet.

#### Key generation: `ledgeracio allowlist gen-key`

This command takes one argument: the basename (filename without extension) of
the keys to generate.  The public key will be given the extension `.pub` and the
secret key the extension `.sec`.  The files will be generated with 0400
permissions, which means that they can only be read by the current user and the
system administrator, and they cannot be written to except by the administrator.
This is to prevent accidental overwrites.

The public key is not sensitive, and is required by anyone who wishes to verify
signed allowlists and operate on the allowed accounts.  It will be uploaded
to the Ledger device by `ledgeracio allowlist set-key`.  The secret key allows
generating signatures, and therefore must be kept secret.  It should never leave
the (preferably air gapped) machine it is generated on.

#### Uploading an allowlist signing key to a device: `ledgeracio allowlist set-key`

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

#### Retrieving the uploaded key: `ledgeracio allowlist get-key`

This command takes no arguments.  The public key that has been uploaded will be
retrieved and printed to stdout.  If no public key has been uploaded, or if the
app is not the Ledgeracio app, an error will be returned.

#### Signing an allowlist: `ledgeracio allowlist sign`

This command takes the following arguments.  All of them are mandatory.

- `--file <file>`: the textual allowlist file to sign.  See
  [FORMATS.md](FORMATS.md) for its format.
- `--nonce <nonce>`: The nonce to sign the file with.  The nonce must be greater
  than the previous nonce, or the Ledgeracio app will reject the allowlist.
- `--output <output>`: The name of the output file to write.
- `--secret <secret>`: The name of the secret key file.

#### Inspecting a signed allowlist: `ledgeracio allowlist inspect`

This command takes two arguments.  Both of them are mandatory.

- `--file <file>`: The name of the signed allowlist to inspect.
- `--public <public>`: The name of the public key file that signed the
  allowlist.  This command will fail if the signature cannot be verified.

#### Uploading an allowlist: `ledgeracio allowlist upload`

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
