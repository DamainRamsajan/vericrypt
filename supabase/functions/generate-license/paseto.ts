// PASETO v4.public (Ed25519) token signing, written to the PASETO specification.
// Token = "v4.public." + base64url(message || signature) [ + "." + base64url(footer) ]
// Signature is over PAE("v4.public.", message, footer, implicitAssertion).
// (The previous hand-rolled code produced "v4.public.<header>.<payload>.<sig>.<footer>", a JWT-like layout that no
// PASETO verifier accepts, and imported the private key with format "raw", which WebCrypto does not allow.)

const enc = new TextEncoder();

function concat(parts: Uint8Array[]): Uint8Array {
  const out = new Uint8Array(parts.reduce((n, p) => n + p.length, 0));
  let o = 0;
  for (const p of parts) { out.set(p, o); o += p.length; }
  return out;
}

function le64(n: number): Uint8Array {
  const b = new Uint8Array(8);
  let v = BigInt(n);
  for (let i = 0; i < 8; i++) { b[i] = Number(v & 0xffn); v >>= 8n; }
  b[7] &= 0x7f; // PASETO clears the most significant bit
  return b;
}

export function pae(pieces: Uint8Array[]): Uint8Array {
  const parts: Uint8Array[] = [le64(pieces.length)];
  for (const p of pieces) { parts.push(le64(p.length), p); }
  return concat(parts);
}

function b64url(bytes: Uint8Array): string {
  let s = "";
  for (const b of bytes) s += String.fromCharCode(b);
  return btoa(s).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

function hexToBytes(h: string): Uint8Array {
  const clean = h.trim();
  if (!/^[0-9a-fA-F]*$/.test(clean) || clean.length % 2 !== 0) throw new Error("private key is not valid hex");
  const m = clean.match(/.{1,2}/g) ?? [];
  return new Uint8Array(m.map((b) => parseInt(b, 16)));
}

// RFC 8410 PKCS#8 wrapper for a raw 32-byte Ed25519 seed (WebCrypto imports private keys as pkcs8 or jwk, never raw).
const PKCS8_ED25519_PREFIX = new Uint8Array([
  0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22, 0x04, 0x20,
]);

/** privateKeyHex: 32-byte seed, or the 64-byte seed||public form that VeriCrypt's tools write. */
export async function signPasetoV4Public(
  privateKeyHex: string,
  claims: Record<string, unknown>,
  footer = "",
): Promise<string> {
  const keyBytes = hexToBytes(privateKeyHex);
  if (keyBytes.length !== 32 && keyBytes.length !== 64) {
    throw new Error("private key must be 32 or 64 bytes (64 or 128 hex characters)");
  }
  const key = await crypto.subtle.importKey(
    "pkcs8",
    concat([PKCS8_ED25519_PREFIX, keyBytes.slice(0, 32)]),
    { name: "Ed25519" },
    false,
    ["sign"],
  );
  const header = enc.encode("v4.public.");
  const message = enc.encode(JSON.stringify(claims));
  const footerBytes = enc.encode(footer);
  const signature = new Uint8Array(
    await crypto.subtle.sign("Ed25519", key, pae([header, message, footerBytes, new Uint8Array(0)])),
  );
  let token = "v4.public." + b64url(concat([message, signature]));
  if (footer) token += "." + b64url(footerBytes);
  return token;
}
