Test-only certificates and keys, generated with OpenSSL for the engine tests. They protect nothing.
Certificates were issued between 21 Oct and 1 Nov 2026 and the tests pin `as_of` to 2026-11-02 so results are deterministic.
`mid.pem` is a non-TLS RSA-2048 certificate valid until April 2032: it exercises the "deprecated after 2030" rule of the final NIST IR 8547.
`web.key` and `edkey.pem` are deliberately unencrypted private keys (the tests check that plaintext keys are flagged).
