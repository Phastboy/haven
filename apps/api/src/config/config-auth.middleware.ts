/**
 * Ed25519 request-signing middleware for /api/config.
 *
 * How it works
 * ─────────────
 * 1. The web client picks the current UTC time as an ISO-8601 string (the "payload").
 * 2. It signs the payload with its Ed25519 private key (CONFIG_PRIVATE_KEY, never sent to the browser).
 * 3. It sends the request with two headers:
 *      X-Config-Timestamp: <ISO-8601>
 *      X-Config-Signature: <base64url-encoded signature>
 * 4. This middleware:
 *      a. Rejects the request if the timestamp is >30 s old or in the future (replay protection).
 *      b. Verifies the signature against CONFIG_PUBLIC_KEY (the only key the API stores).
 *
 * Key generation (one-time setup):
 *   node -e "
 *     const { generateKeyPairSync } = require('crypto');
 *     const { privateKey, publicKey } = generateKeyPairSync('ed25519', {
 *       publicKeyEncoding:  { type: 'spki',  format: 'pem' },
 *       privateKeyEncoding: { type: 'pkcs8', format: 'pem' },
 *     });
 *     console.log('PRIVATE:\n', privateKey);
 *     console.log('PUBLIC:\n', publicKey);
 *   "
 *   Put the PUBLIC key in apps/api/.env  → CONFIG_PUBLIC_KEY
 *   Put the PRIVATE key in apps/web/.env → CONFIG_PRIVATE_KEY   (never shared with the API)
 */

import { createPublicKey, createVerify } from "crypto";
import type { Context } from "elysia";

/** Maximum age of an accepted timestamp, in milliseconds. */
const MAX_AGE_MS = 30_000;

/** Memoised key object — parsed once, reused on every request. */
let cachedPublicKey: ReturnType<typeof createPublicKey> | null = null;

function getPublicKey(): ReturnType<typeof createPublicKey> {
  if (cachedPublicKey) return cachedPublicKey;

  const raw = process.env["CONFIG_PUBLIC_KEY"];
  if (!raw) {
    throw new Error("CONFIG_PUBLIC_KEY environment variable is not set.");
  }

  // The env var may have literal \n instead of real newlines (common in CI secrets).
  const pem = raw.replace(/\\n/g, "\n");

  cachedPublicKey = createPublicKey({ key: pem, format: "pem" });
  return cachedPublicKey;
}

export function verifyConfigSignature({
  headers,
  set,
}: Pick<Context, "headers" | "set">): { error: string } | undefined {
  const timestamp = headers["x-config-timestamp"];
  const signature = headers["x-config-signature"];

  if (!timestamp || !signature) {
    set.status = 401;
    return { error: "Missing config authentication headers." };
  }

  // ── 1. Freshness check ────────────────────────────────────────────────────
  const requestTime = Date.parse(timestamp);
  if (Number.isNaN(requestTime)) {
    set.status = 400;
    return { error: "Invalid X-Config-Timestamp format." };
  }
  const age = Math.abs(Date.now() - requestTime);
  if (age > MAX_AGE_MS) {
    set.status = 401;
    return { error: "Config request timestamp is expired or too far in the future." };
  }

  // ── 2. Signature verification ─────────────────────────────────────────────
  try {
    const publicKey = getPublicKey();
    const verifier = createVerify("SHA512");
    verifier.update(timestamp);

    // Signature arrives as base64url; Node's verify() accepts it directly.
    const valid = verifier.verify(publicKey, signature, "base64");
    if (!valid) {
      set.status = 401;
      return { error: "Invalid config request signature." };
    }
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : "Signature verification failed.";
    set.status = 500;
    return { error: msg };
  }

  // Signature is valid — return undefined to allow the request through.
  return undefined;
}
