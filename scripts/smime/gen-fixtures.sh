#!/usr/bin/env bash
# Regenerate the OpenSSL fixtures for mw-crypto's S/MIME tests (t32-e41) into
# fixtures/crypto/smime/e41/. Needs OpenSSL 3.4+ on PATH (for x509 -not_before /
# -not_after); the S/MIME interop job runs it in its container. Re-running makes
# new keys, so the tests read only what is committed. Every private key is
# written unencrypted (test material only); the PKCS#12 files use password `test`.
#
#   root.pem             RSA-3072 root CA ("Fixture Root")
#   inter.pem            RSA-2048 intermediate (pathlen 0, EKU emailProtection),
#                        signed by the root; also the OCSP responder and CRL signer
#   alice.*              RSA-2048 leaf for alice@example.com, AIA OCSP + CRL DP
#   ecc.*, p384.*        ECDSA/ECDH P-256 and P-384 leaves (ecc@, p384@example.com)
#   ed.*                 Ed25519 leaf (ed@example.com), signing only
#   x25519.*             X25519 leaf (x25519@example.com), key agreement only
#   old.*                RSA leaf valid 2020-01-01 .. 2021-01-01 (expired)
#   revoked.*            RSA leaf, revoked in crl.der and in ocsp-revoked.der
#   carol.*              RSA leaf for carol@example.com (address-mismatch cases)
#   noeku.*              RSA leaf with EKU serverAuth only (not for email)
#   content.txt          the signed/encrypted content (CRLF)
#   *-detached.p7s       detached SignedData over content.txt, DER
#   alice-opaque.p7m     opaque SignedData (content inside), DER
#   alice-pss.p7s        detached, RSASSA-PSS
#   alice-signed.eml     a complete multipart/signed message (openssl cms -text)
#   to-alice-cbc.p7m     EnvelopedData, AES-256-CBC, RSA PKCS#1 v1.5
#   to-alice-oaep.p7m    EnvelopedData, AES-256-CBC, RSA-OAEP
#   to-alice-gcm.p7m     AuthEnvelopedData, AES-256-GCM
#   to-ecc.p7m, to-p384.p7m   key agreement (ECDH + AES-KW). OpenSSL 3.5 does not
#                        encrypt to X25519 certificates, so that case is a
#                        Mailwoman round trip only.
#   crl.der              CRL by the intermediate listing revoked.*
#   ocsp-good.der, ocsp-revoked.der          OCSP responses by the intermediate
set -euo pipefail
# Git for Windows rewrites arguments that start with "/" (the -subj names) into
# Windows paths; switch that off. No effect elsewhere.
export MSYS_NO_PATHCONV=1

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT="${ROOT}/fixtures/crypto/smime/e41"
mkdir -p "${OUT}"
W="$(mktemp -d)"
trap 'rm -rf "${W}"' EXIT
cd "${W}"

AIA="authorityInfoAccess=OCSP;URI:http://127.0.0.1:40080/ocsp"
CDP="crlDistributionPoints=URI:http://127.0.0.1:40080/crl.der"

# Root and intermediate.
openssl req -x509 -newkey rsa:3072 -nodes -keyout root.key -out root.pem -days 3650 \
  -subj "/CN=Fixture Root" \
  -addext "basicConstraints=critical,CA:TRUE" -addext "keyUsage=critical,keyCertSign,cRLSign" 2>/dev/null
openssl req -new -newkey rsa:2048 -nodes -keyout inter.key -out inter.csr -subj "/CN=Fixture Intermediate" 2>/dev/null
printf '%s\n' "basicConstraints=critical,CA:TRUE,pathlen:0" "keyUsage=critical,keyCertSign,cRLSign,digitalSignature" \
  "extendedKeyUsage=emailProtection,OCSPSigning" > inter.ext
openssl x509 -req -in inter.csr -CA root.pem -CAkey root.key -CAcreateserial -out inter.pem -days 3000 -extfile inter.ext 2>/dev/null

# leaf NAME EMAIL KEYARGS KU [EXTRA-EXT...]
leaf() {
  local name=$1 email=$2 keyargs=$3 ku=$4
  shift 4
  # shellcheck disable=SC2086
  openssl genpkey $keyargs -out "${name}.key" 2>/dev/null
  printf '%s\n' "basicConstraints=critical,CA:FALSE" "keyUsage=critical,${ku}" \
    "subjectAltName=email:${email}" "$@" > "${name}.ext"
  if grep -q '^extendedKeyUsage' "${name}.ext"; then :; else echo "extendedKeyUsage=emailProtection" >> "${name}.ext"; fi
  openssl req -new -key "${name}.key" -out "${name}.csr" -subj "/CN=${name}/emailAddress=${email}" 2>/dev/null || true
}

sign_leaf() {
  local name=$1
  shift
  openssl x509 -req -in "${name}.csr" -CA inter.pem -CAkey inter.key -CAcreateserial \
    -out "${name}.pem" -extfile "${name}.ext" "$@" 2>/dev/null
}

leaf alice alice@example.com "-algorithm RSA -pkeyopt rsa_keygen_bits:2048" "digitalSignature,keyEncipherment" "${AIA}" "${CDP}"
sign_leaf alice -days 1000
leaf ecc ecc@example.com "-algorithm EC -pkeyopt ec_paramgen_curve:P-256" "digitalSignature,keyAgreement"
sign_leaf ecc -days 1000
leaf p384 p384@example.com "-algorithm EC -pkeyopt ec_paramgen_curve:P-384" "digitalSignature,keyAgreement"
sign_leaf p384 -days 1000
leaf ed ed@example.com "-algorithm ED25519" "digitalSignature"
sign_leaf ed -days 1000
leaf old old@example.com "-algorithm RSA -pkeyopt rsa_keygen_bits:2048" "digitalSignature,keyEncipherment"
sign_leaf old -not_before 20200101000000Z -not_after 20210101000000Z
leaf revoked revoked@example.com "-algorithm RSA -pkeyopt rsa_keygen_bits:2048" "digitalSignature,keyEncipherment" "${AIA}" "${CDP}"
sign_leaf revoked -days 1000
leaf carol carol@example.com "-algorithm RSA -pkeyopt rsa_keygen_bits:2048" "digitalSignature,keyEncipherment"
sign_leaf carol -days 1000
leaf noeku noeku@example.com "-algorithm RSA -pkeyopt rsa_keygen_bits:2048" "digitalSignature,keyEncipherment" "extendedKeyUsage=serverAuth"
sign_leaf noeku -days 1000

# X25519 cannot sign its own request: the certificate is issued with -force_pubkey
# over a throwaway request.
openssl genpkey -algorithm X25519 -out x25519.key 2>/dev/null
openssl pkey -in x25519.key -pubout -out x25519.pub 2>/dev/null
printf '%s\n' "basicConstraints=critical,CA:FALSE" "keyUsage=critical,keyAgreement" \
  "subjectAltName=email:x25519@example.com" "extendedKeyUsage=emailProtection" > x25519.ext
openssl req -new -newkey rsa:2048 -nodes -keyout throwaway.key -out x25519.csr \
  -subj "/CN=x25519/emailAddress=x25519@example.com" 2>/dev/null
openssl x509 -req -in x25519.csr -CA inter.pem -CAkey inter.key -CAcreateserial -days 1000 \
  -force_pubkey x25519.pub -extfile x25519.ext -out x25519.pem 2>/dev/null

printf 'Content-Type: text/plain; charset=utf-8\r\n\r\nSigned and encrypted by OpenSSL for mw-crypto.\r\n' > content.txt

# Signatures.
for n in alice ecc p384 ed carol old revoked noeku; do
  # OpenSSL has no default digest for Ed25519 in CMS; RFC 8419 names SHA-512.
  md=""
  if [ "${n}" = ed ]; then md="-md sha512"; fi
  # shellcheck disable=SC2086
  openssl cms -sign -binary -in content.txt -signer "${n}.pem" -inkey "${n}.key" -certfile inter.pem \
    ${md} -outform DER -out "${n}-detached.p7s" 2>/dev/null
done
openssl cms -sign -binary -nodetach -in content.txt -signer alice.pem -inkey alice.key -certfile inter.pem \
  -outform DER -out alice-opaque.p7m
openssl cms -sign -binary -in content.txt -signer alice.pem -inkey alice.key -certfile inter.pem \
  -keyopt rsa_padding_mode:pss -outform DER -out alice-pss.p7s
printf 'Mailwoman fixture body.\r\n' > body.txt
{
  printf 'From: alice@example.com\r\nTo: bob@example.com\r\nSubject: OpenSSL signed\r\n'
  openssl cms -sign -text -in body.txt -signer alice.pem -inkey alice.key -certfile inter.pem 2>/dev/null
} > alice-signed.eml

# Encryption.
openssl cms -encrypt -binary -aes-256-cbc -in content.txt -recip alice.pem -outform DER -out to-alice-cbc.p7m
openssl cms -encrypt -binary -aes-256-cbc -in content.txt -recip alice.pem -keyopt rsa_padding_mode:oaep \
  -outform DER -out to-alice-oaep.p7m
openssl cms -encrypt -binary -aes-256-gcm -in content.txt -recip alice.pem -outform DER -out to-alice-gcm.p7m
for n in ecc p384; do
  openssl cms -encrypt -binary -aes-256-cbc -in content.txt -recip "${n}.pem" -outform DER -out "to-${n}.p7m"
done

# Revocation: a CA database listing revoked.* as revoked.
mkdir -p ca && : > ca/index.txt && echo 1000 > ca/crlnumber
cat > ca.cnf <<EOF
[ ca ]
default_ca = fixture
[ fixture ]
database = ca/index.txt
crlnumber = ca/crlnumber
certificate = inter.pem
private_key = inter.key
default_md = sha256
default_crl_days = 3650
EOF
openssl ca -config ca.cnf -valid alice.pem -batch 2>/dev/null
openssl ca -config ca.cnf -valid revoked.pem -batch 2>/dev/null
openssl ca -config ca.cnf -revoke revoked.pem -crl_reason keyCompromise -batch 2>/dev/null
openssl ca -config ca.cnf -gencrl -out crl.pem -batch 2>/dev/null
openssl crl -in crl.pem -outform DER -out crl.der
for n in alice revoked; do
  openssl ocsp -issuer inter.pem -cert "${n}.pem" -no_nonce -reqout "ocsp-req-${n}.der" 2>/dev/null
  openssl ocsp -index ca/index.txt -rsigner inter.pem -rkey inter.key -CA inter.pem \
    -reqin "ocsp-req-${n}.der" -respout "ocsp-${n}.der" -ndays 3650 2>/dev/null
done
mv ocsp-alice.der ocsp-good.der

openssl pkcs12 -export -inkey ecc.key -in ecc.pem -out ecc.p12 -passout pass:test 2>/dev/null
openssl pkcs12 -export -inkey ed.key -in ed.pem -out ed.p12 -passout pass:test 2>/dev/null

for f in root.pem inter.pem content.txt crl.der ocsp-good.der ocsp-revoked.der alice-signed.eml \
         alice-opaque.p7m alice-pss.p7s to-alice-cbc.p7m to-alice-oaep.p7m to-alice-gcm.p7m \
         to-ecc.p7m to-p384.p7m ecc.p12 ed.p12; do
  cp "${f}" "${OUT}/"
done
for n in alice ecc p384 ed old revoked carol noeku x25519; do
  cp "${n}.pem" "${n}.key" "${OUT}/"
done
for n in alice ecc p384 ed carol old revoked noeku; do
  cp "${n}-detached.p7s" "${OUT}/"
done
echo "fixtures written to ${OUT}"
