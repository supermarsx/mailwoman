#!/usr/bin/env bash
# Grant RFC 3062 (LDAP Password Modify) SELF-SERVICE on the seeded tree.
#
# WHY THIS FILE EXISTS
# --------------------
# The `mailwoman` password-change backend `Ldap3062` can be configured to bind
# as the END USER and change that user's own password (`.without_user_identity()`
# — the exop targets the bound identity). `crates/mw-server/tests/v7_e2e.rs`
# exercises exactly that in `passwd_ldap3062_change_live`.
#
# Until 26.20 that test had NEVER run: it failed earlier, on a stale `BOB_DN`
# that did not exist in the seed (`rc=32 noSuchObject`). Once the DN was fixed
# the exop reached the directory for the first time and was refused with
# **`rc=50 insufficientAccessRights`**.
#
# The reason is that `olcDatabase={2}mdb,cn=config` carries **no `olcAccess` at
# all** in this image, so slapd falls back to its built-in default, which is
# effectively read-only for everyone but `rootdn`. (The image does write an
# `olcAccess` when `LDAP_ALLOW_ANON_BINDING=no`, but onto `{1}monitor` — a
# different database — so nothing here conflicts with it.) That is a property of
# the CI fixture, not of the product: the server correctly surfaced the
# directory's refusal.
#
# WHY A SCRIPT AND NOT AN LDIF
# ----------------------------
# `olcAccess` lives in `cn=config`. The image's `LDAP_CUSTOM_LDIF_DIR` (our
# `ldifs/`) is applied with `ldapadd -D "$LDAP_ADMIN_DN"`, i.e. as the *data*
# rootdn, which cannot write `cn=config`; and it can only ADD entries, whereas
# this has to MODIFY the existing database entry. `/docker-entrypoint-initdb.d`
# is the image's supported hook, but it runs with **slapd stopped**, so this uses
# the offline `slapmodify` against the slapd.d directory rather than `ldapmodify`.
#
# THE RULE IS DELIBERATELY NARROW
# -------------------------------
# Rule {0} scopes to the `userPassword` attribute ONLY:
#   * `by self write`     — the one grant RFC 3062 self-service actually needs.
#   * `by anonymous auth` — required for simple bind to work at all; `auth` permits
#                           comparing a password, never reading it.
#   * `by * none`         — nobody else may even read the hashes. This is STRICTER
#                           than the built-in default it replaces, which exposed
#                           them to any authenticated reader.
# It is not a blanket `by self write` on the entry: bob can change his password
# and nothing else about himself.
#
# Rule {1} restates the read access the built-in default was providing, because
# once ANY `olcAccess` is present on a database the built-in default no longer
# applies. Without it the GAL/directory legs (`directory-vs-openldap`, the v7
# GAL search, cert and photo lookups) would all start failing.
#
# `cn=admin` (rootdn) bypasses ACLs entirely, so the admin-bind path the same
# test uses to normalise bob's password beforehand is unaffected.
#
# CI/DEV ONLY, like the rest of this directory. Never point it at a real tree.
set -euo pipefail

conf_dir="${LDAP_ONLINE_CONF_DIR:-/bitnami/openldap/slapd.d}"
suffix="${LDAP_ROOT:-dc=example,dc=com}"

# Resolve the data backend by its suffix rather than assuming `{2}mdb`, so an
# image that numbers its databases differently does not silently get the ACL
# attached to the wrong one (or to none).
db_dn="$(
  slapcat -F "$conf_dir" -n 0 2>/dev/null \
    | awk -v s="$suffix" '
        /^dn: olcDatabase=/ { dn = substr($0, 5); next }
        /^olcSuffix: / { if (substr($0, 12) == s) { print dn; exit } }
      '
)"

if [ -z "$db_dn" ]; then
  echo "[mw-acl] FATAL: no database in $conf_dir has olcSuffix: $suffix." >&2
  echo "[mw-acl] Refusing to continue — without the ACL, RFC 3062 self-service" >&2
  echo "[mw-acl] password change fails with rc=50 and v7_e2e would go red." >&2
  exit 1
fi

echo "[mw-acl] granting self-service userPassword write on $db_dn (suffix $suffix)"

slapmodify -F "$conf_dir" -n 0 <<LDIF
dn: $db_dn
changetype: modify
add: olcAccess
olcAccess: {0}to attrs=userPassword by self write by anonymous auth by * none
olcAccess: {1}to * by * read
LDIF

echo "[mw-acl] done"
