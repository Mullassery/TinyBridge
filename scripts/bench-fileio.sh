#!/usr/bin/env bash
#
# VirtioFS / 9p file-sharing throughput benchmark.
#
# Measures write + read throughput for a shared folder, from inside the guest,
# against the guest's own disk as a control. This is the harness behind
# RECHECK-062: the project's "~90% native file I/O" figure was an aspirational
# target, not a measurement -- run this to replace it with a number.
#
# Usage:
#   scripts/bench-fileio.sh [share_mount] [size_mb]
#     share_mount  guest path to the shared folder (default: /mnt/share)
#     size_mb      test file size in MiB (default: 256)
#
# Environment:
#   GUEST_HOST   ssh target for the guest (default: vm@192.168.105.2)
#   RESULTS_DIR  where to write the JSON result (default: benchmark-results)
#
# Requires a *running* TinyBridge guest with the share mounted at share_mount;
# on Linux guests the 9p path is currently a stub (see docs/FILE_IO_BENCHMARK.md),
# so this will fail there until that lands -- by design, not silently.
#
set -euo pipefail

SHARE_MOUNT="${1:-/mnt/share}"
SIZE_MB="${2:-256}"
GUEST_HOST="${GUEST_HOST:-vm@192.168.105.2}"
RESULTS_DIR="${RESULTS_DIR:-benchmark-results}"

for bin in ssh awk date; do
  command -v "$bin" >/dev/null 2>&1 || { echo "error: '$bin' is required" >&2; exit 1; }
done
case "$SIZE_MB" in
  ''|*[!0-9]*) echo "error: size_mb must be a positive integer" >&2; exit 2 ;;
esac

# dd prints "<bytes> bytes ... in <secs> s, <rate>" to stderr; pull the MB/s.
dd_write_rate() { # $1 = path inside guest
  ssh "$GUEST_HOST" "dd if=/dev/zero of='$1' bs=1M count=$SIZE_MB conv=fdatasync 2>&1 >/dev/null" \
    | awk -F', ' '/s,/{print $NF}' | tail -1
}
dd_read_rate() { # $1 = path inside guest (drop caches first if we can)
  ssh "$GUEST_HOST" "sync; dd if='$1' of=/dev/null bs=1M 2>&1 >/dev/null" \
    | awk -F', ' '/s,/{print $NF}' | tail -1
}

run_case() { # $1 = label, $2 = guest path prefix
  local label="$1" prefix="$2" file="$2/.tinybridge-bench-$$"
  echo "==> $label ($SIZE_MB MiB at $prefix)"
  local write read
  write="$(dd_write_rate "$file")"
  read="$(dd_read_rate "$file")"
  ssh "$GUEST_HOST" "rm -f '$file'" || true
  echo "    write: ${write:-n/a}"
  echo "    read:  ${read:-n/a}"
  printf '%s\t%s\t%s\n' "$label" "$write" "$read"
}

echo "TinyBridge file-sharing benchmark"
echo "  guest: $GUEST_HOST"
echo "  share: $SHARE_MOUNT"
echo ""

if ! ssh -o ConnectTimeout=5 "$GUEST_HOST" "test -d '$SHARE_MOUNT'" 2>/dev/null; then
  echo "error: '$SHARE_MOUNT' is not a directory in the guest $GUEST_HOST." >&2
  echo "       Start a VM with a mounted share first; see docs/FILE_IO_BENCHMARK.md." >&2
  exit 1
fi

mkdir -p "$RESULTS_DIR"
TIMESTAMP="$(date -u +%Y%m%dT%H%M%SZ)"

TABLE="$( {
  run_case "guest-disk (control)" /tmp
  run_case "shared-folder" "$SHARE_MOUNT"
} )"

echo ""
echo "Results:"
printf '%s\n' "$TABLE" | column -t -s $'\t' 2>/dev/null || printf '%s\n' "$TABLE"
echo ""
echo "Note: 'shared-folder' is the VirtioFS (macOS) / 9p (Linux) path. The"
echo "control measures the guest's own virtual disk; the ratio is the honest"
echo "'% of native' figure -- compare it against Lima's published 80-90%."
echo ""

JSON="$RESULTS_DIR/fileio_$TIMESTAMP.json"
{
  echo "{"
  echo "  \"timestamp\": \"$TIMESTAMP\","
  echo "  \"guest\": \"$GUEST_HOST\","
  echo "  \"share_mount\": \"$SHARE_MOUNT\","
  echo "  \"size_mib\": $SIZE_MB,"
  printf '%s\n' "$TABLE" | awk -F'\t' '
    BEGIN { printf "  \"cases\": [\n" }
    { printf "%s    {\"case\": \"%s\", \"write\": \"%s\", \"read\": \"%s\"}", (NR>1?",\n":""), $1, $2, $3 }
    END { printf "\n  ]\n" }
  '
  echo "}"
} > "$JSON"

echo "Wrote $JSON"
