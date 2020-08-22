# Ledgeracio File Formats

Ledgeracio uses several different formats for files.  There is a separate format
for public and secret keys, and a different format for textual allowlists.  The
binary allowlist format is described elsewhere

## Allowlist Signing Keys

Ledgeracio uses bespoke formats for its allowlist signing keys.  The public key
is a text file designed to be easily transmitted between machines, while the
secret key uses a bespoke binary format optimized for error detection and
constant-time parsing.  Both have magic numbers that are recognizable by tools
like [file(1)](man:file(1)), and are versioned in case future changes are
needed.

### Secret Keys

Ledgeracio secret keys are 88 bytes long.  They should use the `.sec` file
extension, although this is only enforced by `ledgeracio allowlist gen-key`.

A Ledgeracio secret key is described by the following C struct, little-endian
encoded:

```c
struct LedgeracioSecretKey {
    uint8_t magic[21];
    uint8_t version;
    uint8_t reserved;
    uint8_t network;
    unsigned char secret[32];
    unsigned char public[32];
}
```

A Ledgeracio secret key always begins with the magic sequence
`Ledgeracio Secret Key`, case-sensitive.  `version` is a version number, and is
1 for keys following this specification.  `reserved` MUST be set to 0.
`network` indicates the network this key should be used with.  `secret` is an
ed25519 secret key, and `public` is the corresponding public key.

Tools MUST reject a secret key if:

- The magic number is wrong
- The version is unknown
- Fields designated as “reserved” are not 0
- The secret and public keys do not match each other

Secret keys SHOULD be generated on the machine they will be used on and SHOULD
NOT ever leave that machine.  It is expected that they will be generated on a
trusted computer that is only used for provisioning Ledger devices and has no
access to the Internet.

### Allowlist Public Keys

Ledgeracio public keys use a textual format designed for easy transmission.  The
parser is currently very strict.

The following regular expression defines the public key format:

```
^Ledgeracio version ([1-9][0-9]*) public key for network ([[:alpha:]]+)
([[:alnum:]/+]{43}=)
$
```

Line endings MUST consist of a single line feed.  Excess whitespace, including
at the end of lines, is not permitted.

The first capture group is the version; it is 1 for keys conforming to this
specification.  The second capture group is the human-readable name of the
network, ASCII case-insensitive.  The final capture group is the base64-encoded
ed25519 public key.

Tools MUST reject a public key if it is syntactically incorrect, the network or
version is unknown, or the public key is not valid.
