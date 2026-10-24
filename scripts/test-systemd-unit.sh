#!/usr/bin/env bash
# Start docs/deploy/mailwoman.service under a real systemd and check that it works.
#
# Usage: scripts/test-systemd-unit.sh [--host] <image>
#   <image>  an image built with `docker build --target runtime-static`; its static mailwoman
#            and mw-render binaries and its plugins directory are copied out of it.
#   --host   use this machine's systemd (a GitHub runner); needs root or sudo, and port 8080 on
#            127.0.0.1 free. It installs to /usr/local/bin, /usr/lib/mailwoman/plugins,
#            /etc/mailwoman and /etc/systemd/system, creates the `mailwoman` user, and removes all
#            of it on exit. Without --host the same steps run inside a privileged container that
#            boots systemd (needs Docker).
#
# Environment:
#   MW_SYSTEMD_UNIT       unit file to install (default docs/deploy/mailwoman.service)
#   MW_SYSTEMD_IMAGE      base image for the container (default: ubuntu:24.04, pinned below)
#   MW_SYSTEMD_CONTAINER  container name (default mw-systemd-unit-test-<pid>)
#
# The unit file is installed byte for byte. A drop-in adds only an EnvironmentFile (mode 0600)
# with a generated MW_SERVER_KEY and the settings the probes need: MW_MODE=engine (plugins load
# only in engine mode), an admin user, MW_CSRF_STRICT=true, and trusted-header sign-in from
# 127.0.0.1 so a mailbox session exists without a mail server. Header sign-in is for this test
# only; it is not a production setting.
#
# Checks, in order (the first failure stops the run with FAIL and the journal):
#   1. `systemd-analyze verify` on the unit (output printed);
#   2. the service is active 10 s after start and has not been restarted;
#   3. GET /healthz answers 200;
#   4. every directive below, read back with `systemctl show`, equals the unit file, and the
#      running process has NoNewPrivs 1 and a seccomp filter (Seccomp 2) in /proc/<pid>/status;
#   5. a first-party plugin (spam-rspamd) is compiled by wasmtime and answers a classifier probe
#      (this is what decides whether MemoryDenyWriteExecute=true is compatible with the build);
#   6. POST /api/sanitize returns 200 with the script removed. The render jail is required by
#      default on Linux, so a 200 means the mw-render child ran under the unit's restrictions;
#   7. `systemd-analyze security` output is printed (informational).
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
# ubuntu:24.04 (systemd 255, the same major as GitHub's ubuntu-24.04 runner), index digest.
default_base='ubuntu:24.04@sha256:68ee22004a9109b44973313b02ff7b1de250f1de0f5ecb628d73f579aaff33ac'

host=false
if [[ "${1:-}" == "--host" ]]; then
  host=true
  shift
fi
if (($# != 1)); then
  sed -n '2,13p' "$0" >&2
  exit 2
fi
image="$1"
unit="${MW_SYSTEMD_UNIT:-$root/docs/deploy/mailwoman.service}"
base="${MW_SYSTEMD_IMAGE:-$default_base}"
ctr="${MW_SYSTEMD_CONTAINER:-mw-systemd-unit-test-$$}"
work="$(mktemp -d)"

log() { printf '[systemd-unit] %s\n' "$*"; }
die() {
  printf '[systemd-unit] FAIL: %s\n' "$*" >&2
  exit 1
}

[[ -r "$unit" ]] || die "cannot read unit $unit"

# A path the docker CLI can open: on Git Bash for Windows it does not read /tmp/... paths.
native_path() {
  if command -v cygpath >/dev/null 2>&1; then cygpath -m "$1"; else printf '%s
' "$1"; fi
}

as_root() {
  if [[ $(id -u) -eq 0 ]]; then "$@"; else sudo "$@"; fi
}

# Run a shell function (with the helpers below) as root inside the target: the container, or
# this machine with --host.
in_target() {
  local script
  script="$(declare -f t_fail t_property t_setup t_probe t_cleanup); $*"
  if $host; then
    as_root env -u BASH_ENV bash -c "$script"
  else
    MSYS_NO_PATHCONV=1 docker exec -i "$ctr" bash -c "$script"
  fi
}

# Copy a local file or directory to a path in the target.
to_target() {
  if $host; then
    as_root mkdir -p "$(dirname "$2")"
    as_root cp -R "$1" "$2"
  else
    MSYS_NO_PATHCONV=1 docker cp "$(native_path "$1")" "$ctr:$2" >/dev/null
  fi
}

cleanup() {
  local status=$?
  if [[ -n "${cid:-}" ]]; then docker rm "$cid" >/dev/null 2>&1 || true; fi
  if $host; then
    in_target t_cleanup || true
  elif [[ -n "${started_ctr:-}" ]]; then
    docker rm -f "$ctr" >/dev/null 2>&1 || true
  fi
  rm -rf "$work"
  exit "$status"
}
trap cleanup EXIT

# ----------------------------------------------------------------------------------------------
# Functions that run inside the target (as root). They are shipped there with `declare -f`, so
# they use only bash, coreutils, curl, sed, grep and systemd tools.

t_fail() {
  printf '[systemd-unit] FAIL: %s\n' "$*" >&2
  journalctl -u mailwoman --no-pager -n 60 >&2 || true
  exit 1
}

# t_property NAME EXPECTED: compare `systemctl show` with the value written in the unit file.
t_property() {
  local name="$1" want="$2" got
  got="$(systemctl show -p "$name" --value mailwoman)"
  case "$want" in
    true) want=yes ;;
    false) want=no ;;
  esac
  case "$name" in
    RestrictAddressFamilies)
      want="$(tr ' ' '\n' <<<"$want" | sort | tr '\n' ' ')"
      got="$(tr ' ' '\n' <<<"$got" | sort | tr '\n' ' ')"
      ;;
    SystemCallErrorNumber)
      # systemctl show prints the errno number.
      case "$want" in
        EPERM) want=1 ;;
        EACCES) want=13 ;;
        ENOSYS) want=38 ;;
      esac
      ;;
    SystemCallFilter)
      # systemd expands the set into syscall names; check it is in force and holds a member of
      # @system-service rather than comparing the expansion.
      if [[ -n "$got" && " $got " == *" read "* ]]; then got="$want"; fi
      ;;
  esac
  if [[ "$got" != "$want" ]]; then
    t_fail "$name: unit file says '$want', systemctl show says '$got'"
  fi
  printf '[systemd-unit] %s=%s\n' "$name" "$got"
}

t_setup() {
  local key="$1" password="$2"
  id mailwoman >/dev/null 2>&1 ||
    useradd --system --user-group --no-create-home --home-dir /var/lib/mailwoman \
      --shell /usr/sbin/nologin mailwoman
  install -m 0755 /tmp/mw-stage/mailwoman /tmp/mw-stage/mw-render /usr/local/bin/
  mkdir -p /usr/lib/mailwoman
  rm -rf /usr/lib/mailwoman/plugins
  cp -R /tmp/mw-stage/plugins /usr/lib/mailwoman/plugins
  chmod -R a+rX,go-w /usr/lib/mailwoman/plugins
  install -m 0644 /tmp/mw-stage/mailwoman.service /etc/systemd/system/mailwoman.service
  cmp /tmp/mw-stage/mailwoman.service /etc/systemd/system/mailwoman.service
  install -d -m 0700 /etc/mailwoman
  umask 077
  cat >/etc/mailwoman/mailwoman.env <<EOF
MW_SERVER_KEY=$key
MW_MODE=engine
MW_ADMIN_USER=root
MW_ADMIN_PASSWORD=$password
MW_CSRF_STRICT=true
MW_HEADER_AUTH=1
MW_HEADER_AUTH_TRUSTED_IPS=127.0.0.1
EOF
  mkdir -p /etc/systemd/system/mailwoman.service.d
  printf '[Service]\nEnvironmentFile=/etc/mailwoman/mailwoman.env\n' \
    >/etc/systemd/system/mailwoman.service.d/test-environment.conf
  systemctl daemon-reload
}

t_probe() {
  local password="$1" b=http://127.0.0.1:8080 out status pid name value resp
  local adm csrf ses ses_csrf

  printf '[systemd-unit] systemd %s\n' "$(systemctl --version | sed -n 1p)"
  printf '[systemd-unit] -- systemd-analyze verify\n'
  systemd-analyze verify /etc/systemd/system/mailwoman.service 2>&1 ||
    t_fail "systemd-analyze verify rejected the unit"

  systemctl start mailwoman || t_fail "systemctl start mailwoman failed"
  sleep 10
  status="$(systemctl is-active mailwoman || true)"
  [[ "$status" == active ]] || t_fail "service is '$status' 10 s after start"
  [[ "$(systemctl show -p NRestarts --value mailwoman)" == 0 ]] ||
    t_fail "service restarted $(systemctl show -p NRestarts --value mailwoman) time(s)"
  printf '[systemd-unit] active after 10 s, NRestarts=0\n'

  status="$(curl -sS -o /dev/null -w '%{http_code}' "$b/healthz")"
  [[ "$status" == 200 ]] || t_fail "GET /healthz answered $status"
  printf '[systemd-unit] GET /healthz 200\n'

  # Every hardening and filesystem directive in the unit's [Service] section, as written there.
  while IFS='=' read -r name value; do
    case "$name" in
      NoNewPrivileges | ProtectSystem | ProtectHome | PrivateTmp | PrivateDevices | \
        CapabilityBoundingSet | AmbientCapabilities | ProtectKernelTunables | \
        ProtectKernelModules | ProtectKernelLogs | ProtectControlGroups | ProtectClock | \
        ProtectHostname | ProtectProc | RestrictNamespaces | RestrictRealtime | \
        RestrictSUIDSGID | LockPersonality | MemoryDenyWriteExecute | RemoveIPC | \
        SystemCallFilter | SystemCallErrorNumber | SystemCallArchitectures | \
        RestrictAddressFamilies | UMask | StateDirectory | ReadWritePaths | User | Group)
        t_property "$name" "$value"
        ;;
    esac
  done < <(sed -n '/^\[Service\]/,/^\[/{/^[A-Za-z]/p}' /etc/systemd/system/mailwoman.service)

  pid="$(systemctl show -p MainPID --value mailwoman)"
  grep -q '^NoNewPrivs:[[:space:]]*1$' "/proc/$pid/status" || t_fail "process $pid: NoNewPrivs is not 1"
  grep -q '^Seccomp:[[:space:]]*2$' "/proc/$pid/status" || t_fail "process $pid: no seccomp filter"
  printf '[systemd-unit] process %s: NoNewPrivs 1, Seccomp 2\n' "$pid"

  # Plugin: admin sign-in, then register, approve, enable and grant spam-rspamd. The grant is
  # where wasmtime compiles the component (Component::new, which needs executable memory).
  out="$(curl -sS -D - -o /dev/null -H 'content-type: application/json' \
    -d "{\"username\":\"root\",\"password\":\"$password\"}" "$b/admin/login")"
  adm="$(sed -n 's/^[Ss]et-[Cc]ookie: mw_admin_session=\([^;]*\);.*/\1/p' <<<"$out" | tr -d '\r')"
  csrf="$(sed -n 's/^[Ss]et-[Cc]ookie: mw_csrf=\([^;]*\);.*/\1/p' <<<"$out" | tr -d '\r')"
  [[ -n "$adm" && -n "$csrf" ]] || t_fail "admin sign-in set no session: $(sed -n 1p <<<"$out")"
  local -a admin=(-H "Cookie: mw_admin_session=$adm; mw_csrf=$csrf" -H "X-CSRF-Token: $csrf"
    -H 'content-type: application/json')
  curl -sS -f "${admin[@]}" -d '{"id":"spam-rspamd"}' "$b/admin/plugins" >/dev/null ||
    t_fail "register spam-rspamd refused"
  curl -sS -f "${admin[@]}" -X POST "$b/admin/plugins/spam-rspamd/approve" >/dev/null ||
    t_fail "approve spam-rspamd refused"
  curl -sS -f "${admin[@]}" -X POST "$b/admin/plugins/spam-rspamd/enable" >/dev/null ||
    t_fail "enable spam-rspamd refused"
  resp="$(curl -sS "${admin[@]}" -d '{"capabilities":["spam-action","store-kv-scoped"]}' \
    "$b/admin/plugins/spam-rspamd/grant")"
  printf '[systemd-unit] grant: %s\n' "$resp"
  [[ "$resp" == *'"loaded":true'* ]] || t_fail "spam-rspamd did not load after the grant"
  resp="$(curl -sS -w '\n%{http_code}' "${admin[@]}" -X POST "$b/admin/plugins/spam-rspamd/test")"
  printf '[systemd-unit] classifier probe: %s\n' "$(tr '\n' ' ' <<<"$resp")"
  [[ "${resp##*$'\n'}" == 200 && "$resp" == *'"verdict"'* ]] ||
    t_fail "spam-rspamd classifier probe did not run the guest"

  # Render: a mailbox session through trusted-header sign-in, then the sanitizer (mw-render).
  out="$(curl -sS -D - -o /dev/null -H 'X-Remote-User: probe@example.org' \
    -H 'content-type: application/json' -d '{"username":"","password":""}' \
    "$b/api/login")"
  ses="$(sed -n 's/^[Ss]et-[Cc]ookie: mw_session=\([^;]*\);.*/\1/p' <<<"$out" | tr -d '\r')"
  ses_csrf="$(sed -n 's/^[Ss]et-[Cc]ookie: mw_csrf=\([^;]*\);.*/\1/p' <<<"$out" | tr -d '\r')"
  [[ -n "$ses" && -n "$ses_csrf" ]] || t_fail "mailbox sign-in set no session: $(sed -n 1p <<<"$out")"
  resp="$(curl -sS -w '\n%{http_code}' -H "Cookie: mw_session=$ses; mw_csrf=$ses_csrf" \
    -H "X-CSRF-Token: $ses_csrf" -H 'content-type: application/json' \
    -d '{"html":"<p onclick=x>hi</p><script>alert(1)</script>"}' "$b/api/sanitize")"
  printf '[systemd-unit] sanitize: %s\n' "$(tr '\n' ' ' <<<"$resp")"
  [[ "${resp##*$'\n'}" == 200 && "$resp" == *'hi'* && "$resp" != *'<script'* ]] ||
    t_fail "sanitize through mw-render failed"

  [[ "$(systemctl is-active mailwoman || true)" == active ]] || t_fail "service stopped during the probes"
  printf '[systemd-unit] -- systemd-analyze security (informational)\n'
  systemd-analyze security --no-pager mailwoman.service 2>&1 | tail -n 25 || true
}

t_cleanup() {
  systemctl stop mailwoman 2>/dev/null || true
  rm -rf /etc/systemd/system/mailwoman.service /etc/systemd/system/mailwoman.service.d \
    /etc/mailwoman /usr/lib/mailwoman /usr/local/bin/mailwoman /usr/local/bin/mw-render \
    /var/lib/mailwoman /tmp/mw-stage
  systemctl daemon-reload || true
  userdel mailwoman 2>/dev/null || true
}

# ----------------------------------------------------------------------------------------------

log "image $image; unit $unit"
stage="$work/mw-stage"
mkdir -p "$stage"
cid="$(docker create "$image")"
for f in /usr/local/bin/mailwoman /usr/local/bin/mw-render; do
  MSYS_NO_PATHCONV=1 docker cp "$cid:$f" "$(native_path "$stage")/" >/dev/null
done
MSYS_NO_PATHCONV=1 docker cp "$cid:/usr/lib/mailwoman/plugins" "$(native_path "$stage")/plugins" >/dev/null
docker rm "$cid" >/dev/null
cid=""
cp "$unit" "$stage/mailwoman.service"
log "copied mailwoman, mw-render and $(find "$stage/plugins" -name '*.wasm' | wc -l | tr -d ' ') plugin(s) out of the image"

if $host; then
  [[ "$(ps -p 1 -o comm=)" == systemd ]] || die "--host needs systemd as PID 1"
  as_root rm -rf /tmp/mw-stage
  to_target "$stage" /tmp/mw-stage
else
  log "booting systemd in container $ctr ($base)"
  started_ctr=1
  MSYS_NO_PATHCONV=1 docker run -d --name "$ctr" --privileged --cgroupns=private \
    --tmpfs /run --tmpfs /run/lock -e container=docker "$base" \
    bash -c 'export DEBIAN_FRONTEND=noninteractive
      apt-get update -qq && apt-get install -y -qq --no-install-recommends \
        systemd systemd-sysv curl ca-certificates >/dev/null && exec /lib/systemd/systemd' >/dev/null
  for _ in $(seq 1 120); do
    state="$(MSYS_NO_PATHCONV=1 docker exec "$ctr" systemctl is-system-running 2>/dev/null || true)"
    [[ "$state" == running || "$state" == degraded ]] && break
    [[ "$(docker inspect -f '{{.State.Running}}' "$ctr")" == true ]] ||
      die "container exited: $(docker logs --tail 20 "$ctr" 2>&1)"
    sleep 2
  done
  [[ "$state" == running || "$state" == degraded ]] || die "systemd did not boot (state '$state')"
  log "systemd is $state"
  to_target "$stage" /tmp/mw-stage
fi

key="$(od -An -N32 -tx1 /dev/urandom | tr -d ' \n')"
password="$(od -An -N18 -tx1 /dev/urandom | tr -d ' \n')"
in_target "t_setup $key $password"
in_target "t_probe $password"
log "PASS: the unit starts and runs a plugin and a render under its own restrictions"
