#!/usr/bin/env bash
set -euo pipefail

# Run the privileged DHCP integration suite entirely inside disposable Linux
# network namespaces. This script must be invoked as root (CI uses sudo).

if [[ "$(id -u)" -ne 0 ]]; then
  printf 'error: scripts/test-privileged.sh must run as root\n' >&2
  exit 5
fi

for required in ip ss python3 cmp; do
  if ! command -v "$required" >/dev/null 2>&1; then
    printf 'error: required command not found: %s\n' "$required" >&2
    exit 70
  fi
done

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BINARY="$REPO_ROOT/target/debug/l2linkscope"
TEST_TMP="$(mktemp -d -t l2linkscope-integration.XXXXXXXX)"
CLIENT_NS="l2ls-client-$$"
SERVER_NS="l2ls-server-$$"
HOST_CLIENT_IF="l2lc$$"
HOST_SERVER_IF="l2ls$$"
CLIENT_IF="lsclient0"
SERVER_IF="lsserver0"
FIXTURE_PID=""

cleanup() {
  if [[ -n "$FIXTURE_PID" ]]; then
    kill "$FIXTURE_PID" 2>/dev/null || true
    wait "$FIXTURE_PID" 2>/dev/null || true
  fi
  ip netns delete "$CLIENT_NS" 2>/dev/null || true
  ip netns delete "$SERVER_NS" 2>/dev/null || true
  case "$TEST_TMP" in
    /tmp/l2linkscope-integration.*) rm -rf -- "$TEST_TMP" ;;
    *) printf 'refusing to remove unexpected temporary path: %s\n' "$TEST_TMP" >&2 ;;
  esac
}
trap cleanup EXIT INT TERM

cd "$REPO_ROOT"
if [[ ! -x "$BINARY" ]]; then
  printf 'error: %s is missing; run cargo build --locked -p l2linkscope before sudo\n' \
    "$BINARY" >&2
  exit 70
fi

ip netns add "$CLIENT_NS"
ip netns add "$SERVER_NS"
ip link add "$HOST_CLIENT_IF" type veth peer name "$HOST_SERVER_IF"
ip link set "$HOST_CLIENT_IF" netns "$CLIENT_NS"
ip link set "$HOST_SERVER_IF" netns "$SERVER_NS"
ip -n "$CLIENT_NS" link set "$HOST_CLIENT_IF" name "$CLIENT_IF"
ip -n "$SERVER_NS" link set "$HOST_SERVER_IF" name "$SERVER_IF"
ip -n "$CLIENT_NS" link set "$CLIENT_IF" addrgenmode none
ip -n "$SERVER_NS" link set "$SERVER_IF" addrgenmode none
ip -n "$CLIENT_NS" link set lo up
ip -n "$SERVER_NS" link set lo up
ip -n "$CLIENT_NS" link set "$CLIENT_IF" up
ip -n "$SERVER_NS" link set "$SERVER_IF" up

snapshot_state() {
  local destination="$1"
  {
    ip -n "$CLIENT_NS" -details -json link show dev "$CLIENT_IF"
    ip -n "$CLIENT_NS" -json address show dev "$CLIENT_IF"
    ip -n "$CLIENT_NS" -json route show table all dev "$CLIENT_IF"
  } > "$destination"
}

assert_json_offer_count() {
  local json_file="$1"
  local expected="$2"
  python3 - "$json_file" "$expected" <<'PY'
import json
import sys

document = json.load(open(sys.argv[1], encoding="utf-8"))
expected = int(sys.argv[2])

def find_offers(value):
    if isinstance(value, dict):
        for key in ("offers", "observations"):
            candidate = value.get(key)
            if isinstance(candidate, list):
                return candidate
        for child in value.values():
            found = find_offers(child)
            if found is not None:
                return found
    return None

offers = find_offers(document)
if offers is None:
    raise SystemExit("JSON output has no offers or observations array")
if len(offers) != expected:
    raise SystemExit(f"expected {expected} offers, found {len(offers)}")
PY
}

run_scenario() {
  local mode="$1"
  local expected_exit="$2"
  local expected_offers="$3"
  local case_dir="$TEST_TMP/$mode"
  local ready="$case_dir/ready"
  local log="$case_dir/messages.log"
  local output="$case_dir/output.json"
  local stderr_file="$case_dir/stderr.log"
  local before="$case_dir/state.before"
  local after="$case_dir/state.after"
  mkdir -p "$case_dir"

  snapshot_state "$before"
  ip netns exec "$SERVER_NS" python3 "$REPO_ROOT/scripts/dhcp_fixture.py" \
    --interface "$SERVER_IF" --mode "$mode" --log "$log" --ready "$ready" &
  FIXTURE_PID=$!

  for _attempt in {1..100}; do
    [[ -e "$ready" ]] && break
    sleep 0.02
  done
  if [[ ! -e "$ready" ]]; then
    printf 'error: DHCP fixture did not become ready for %s\n' "$mode" >&2
    return 70
  fi

  set +e
  ip netns exec "$CLIENT_NS" "$BINARY" probe dhcp4 "$CLIENT_IF" \
    --timeout 1s --json > "$output" 2> "$stderr_file"
  local actual_exit=$?
  set -e

  wait "$FIXTURE_PID"
  FIXTURE_PID=""
  snapshot_state "$after"

  if [[ "$actual_exit" -ne "$expected_exit" ]]; then
    printf 'error: %s exited %s, expected %s\n' \
      "$mode" "$actual_exit" "$expected_exit" >&2
    sed -n '1,120p' "$stderr_file" >&2
    return 1
  fi
  if ! cmp -s "$before" "$after"; then
    printf 'error: interface state changed during %s scenario\n' "$mode" >&2
    diff -u "$before" "$after" >&2 || true
    return 1
  fi
  if [[ "$(wc -l < "$log")" -ne 1 ]] || ! grep -qx '1' "$log"; then
    printf 'error: %s emitted an unexpected DHCP message or retransmission\n' "$mode" >&2
    sed -n '1,120p' "$log" >&2
    return 1
  fi
  if [[ "$expected_offers" != "-" ]]; then
    assert_json_offer_count "$output" "$expected_offers"
  fi
  printf 'ok: %s\n' "$mode"
}

run_scenario one 0 1
run_scenario two 0 2
run_scenario none 1 0
run_scenario malformed 7 -
run_scenario wrong-xid 1 0
run_scenario wrong-chaddr 1 0
run_scenario duplicate 0 1

# A process without root or capabilities must receive the documented privilege
# category. setpriv is supplied by util-linux on the CI runner.
if command -v setpriv >/dev/null 2>&1 && id nobody >/dev/null 2>&1; then
  set +e
  ip netns exec "$CLIENT_NS" setpriv --reuid=nobody --regid=nogroup \
    --clear-groups --bounding-set=-all --inh-caps=-all --ambient-caps=-all \
    "$BINARY" probe dhcp4 "$CLIENT_IF" --timeout 1s \
    > "$TEST_TMP/unprivileged.stdout" 2> "$TEST_TMP/unprivileged.stderr"
  unprivileged_exit=$?
  set -e
  if [[ "$unprivileged_exit" -ne 5 ]]; then
    printf 'error: unprivileged probe exited %s, expected 5\n' \
      "$unprivileged_exit" >&2
    sed -n '1,120p' "$TEST_TMP/unprivileged.stderr" >&2
    exit 1
  fi
else
  printf 'error: setpriv and the nobody account are required for privilege testing\n' >&2
  exit 70
fi

# Making the selected interface unavailable during collection must fail safely
# rather than silently reporting a normal no-Offer result.
set +e
ip netns exec "$CLIENT_NS" "$BINARY" probe dhcp4 "$CLIENT_IF" --timeout 3s \
  > "$TEST_TMP/disappeared.stdout" 2> "$TEST_TMP/disappeared.stderr" &
disappeared_pid=$!
socket_ready=false
for _attempt in {1..100}; do
  if ip netns exec "$CLIENT_NS" ss -H -u -a -n | grep -q ':68'; then
    socket_ready=true
    break
  fi
  sleep 0.02
done
if [[ "$socket_ready" != true ]]; then
  printf 'error: probe did not bind UDP port 68 before availability test\n' >&2
  kill "$disappeared_pid" 2>/dev/null || true
  wait "$disappeared_pid" 2>/dev/null || true
  exit 1
fi
ip -n "$CLIENT_NS" link set "$CLIENT_IF" down
wait "$disappeared_pid"
disappeared_exit=$?
set -e
if [[ "$disappeared_exit" -ne 6 ]]; then
  printf 'error: disappearing interface exited %s, expected 6\n' \
    "$disappeared_exit" >&2
  sed -n '1,120p' "$TEST_TMP/disappeared.stderr" >&2
  exit 1
fi
printf 'ok: disappearing interface\n'

printf 'All isolated privileged integration scenarios passed.\n'
