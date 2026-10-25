import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { beforeAll, describe, expect, it } from "vitest";
import * as mw from "../wasm/mw-crypto/mw_crypto.js";
import type { SmimeVerification } from "../contracts/crypto.ts";

const fixture = (name: string): Uint8Array =>
  new Uint8Array(
    readFileSync(
      resolve(process.cwd(), "../../fixtures/crypto/smime/e41", name),
    ),
  );
const pem = (name: string): string => new TextDecoder().decode(fixture(name));

beforeAll(() => {
  mw.initSync({
    module: readFileSync(
      resolve(process.cwd(), "src/wasm/mw-crypto/mw_crypto_bg.wasm"),
    ),
  });
  mw.__init();
});

describe("S/MIME committed browser guest", () => {
  it("verifies external detached content and refuses tampering", () => {
    const signature = Buffer.from(fixture("alice-detached.p7s")).toString(
      "base64",
    );
    const data = pem("content.txt");
    expect(mw.verify({ kind: "smime", signature, data }).status).toBe(
      "verified",
    );
    expect(
      mw.verify({ kind: "smime", signature, data: `${data}!` }).status,
    ).toBe("invalid");
  });

  it("returns trust, sender, chain and revocation evidence", () => {
    const args = {
      cms: fixture("alice-detached.p7s"),
      content: fixture("content.txt"),
      adminTrust: [pem("root.pem")],
      mozilla: false,
      sender: "alice@example.com",
    };
    const result = mw.verifySmime(args) as SmimeVerification;
    expect(result.verification.state).toBe("revocation-unchecked");
    expect(result.anchor).toBe("admin");
    expect(result.chain).toHaveLength(3);
    expect(result.revocationQueries).toHaveLength(2);
    expect(
      mw.verifySmime({ ...args, sender: "wrong@example.com" }).verification
        .state,
    ).toBe("address-mismatch");
    expect(
      mw.verifySmime({ ...args, hardFail: true }).verification.signature.status,
    ).toBe("unverified-key");
    expect(mw.smimeRootSetInfo({}).count).toBeGreaterThan(50);
  });

  it("signs detached with ECDSA and Ed25519", () => {
    for (const [name, algorithm] of [
      ["ecc", "ecdsa-p256"],
      ["p384", "ecdsa-p384"],
      ["ed", "ed25519"],
    ]) {
      const signed = mw.sign({
        kind: "smime",
        data: "body",
        certPem: pem(`${name}.pem`),
        encryptedPrivateBundle: pem(`${name}.key`),
        passphrase: "",
        detached: true,
      });
      const result = mw.verify({
        kind: "smime",
        signature: signed.signatureArmored,
        data: "body",
      });
      expect(result.status).toBe("verified");
      expect(result.algorithm).toBe(algorithm);
      expect(
        mw.verify({
          kind: "smime",
          signature: signed.signatureArmored,
          data: "changed",
        }).status,
      ).toBe("invalid");
    }
  });

  it("uses GCM for capable recipients and CBC for unknown recipients", () => {
    for (const recipientGcm of [[true], [null]]) {
      const encrypted = mw.encrypt({
        kind: "smime",
        plaintext: "private body",
        recipientPublicKeys: [pem("ecc.pem")],
        recipientGcm,
      });
      expect(encrypted.smimeType).toBe(
        recipientGcm[0] ? "authEnveloped-data" : "enveloped-data",
      );
      const result = mw.decrypt({
        kind: "smime",
        ciphertext: encrypted.armoredCiphertext,
        encryptedPrivateBundle: pem("ecc.key"),
        passphrase: "",
      });
      expect(result.plaintextText).toBe("private body");
    }
  });

  it("signs then encrypts and verifies the inner signature on decrypt", () => {
    const { keyRef } = mw.unlockKey({
      kind: "smime",
      encryptedPrivateBundle: pem("ed.key"),
      passphrase: "",
    });
    try {
      const encrypted = mw.encrypt({
        kind: "smime",
        plaintext: "signed private body",
        recipientPublicKeys: [pem("ecc.pem")],
        recipientGcm: [true],
        signWithKeyRef: keyRef,
        signWithCertPem: pem("ed.pem"),
      });
      const result = mw.decrypt({
        kind: "smime",
        ciphertext: encrypted.armoredCiphertext,
        encryptedPrivateBundle: pem("ecc.key"),
        passphrase: "",
      });
      expect(result.plaintextText).toBe("signed private body");
      expect(result.signature.status).toBe("unverified-key");
      expect(result.verification.state).toBe("untrusted");
    } finally {
      mw.lockKey({ keyRef });
    }
  });

  it("generates an Ed25519 certificate through the public worker API", () => {
    const result = mw.generateKey({
      kind: "smime",
      keyType: "ed25519",
      userId: "Browser <browser@example.com>",
      passphrase: "test password",
    });
    expect(result.algorithm).toBe("ed25519");
    expect(result.encryptedPrivateBundle).toContain("ENCRYPTED PRIVATE KEY");
    expect(result.addresses).toContain("browser@example.com");
  }, 60_000);
});
