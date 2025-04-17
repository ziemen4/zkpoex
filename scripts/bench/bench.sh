#!/usr/bin/env bash
set -euxo pipefail

# ─── 0) Inputs and defaults ────────────────────────────────────────────────
FUNCTION="$1";    PARAMS="$2"
CONTEXT_STATE="$3"; PROGRAM_SPEC="$4"
VALUE="${5:-0}"
PPROF_OUT="${PPROF_OUT:-profile.pb}"

OS=$(uname -s)              # e.g. Darwin or Linux
ARCH=$(uname -m)            # e.g. x86_64 or arm64
if [ "$OS" = "Darwin" ]; then
  CORES=$(sysctl -n hw.ncpu)
  total_ram=$(sysctl -n hw.memsize)
else
  CORES=$(nproc)
  total_ram=$(grep MemTotal /proc/meminfo | awk '{print $2 * 1024}')
fi

# round RAM to GB for folder name
ram_gb=$(( total_ram / 1024 / 1024 / 1024 ))
RESULTS_DIR="scripts/bench/results/${OS}_${ARCH}_${CORES}c_${ram_gb}G"
mkdir -p "$RESULTS_DIR"
CSV="$RESULTS_DIR/bench_results.csv"

# ─── 1) Make sure we have a header ─────────────────────────────────────────
if [ ! -f "$CSV" ]; then
  echo "label,wall_s,user_s,sys_s,maxrss_kB,cycles,cpu_pct,mem_pct,exec_time_ms,segments,rz_total_cycles,user_cycles,user_pct,paging_cycles,paging_pct,reserved_cycles,reserved_pct,timestamp" > "$CSV"
fi

# ─── 2) Figure out which `time` and RSS units ───────────────────────────────
case "$(uname -s)" in
  Darwin*)
    tcmd="/usr/bin/time -l"
    rss_key="maximum resident"
    rss_in_bytes=true
    total_ram=$(sysctl -n hw.memsize)
    ;;
  *)
    tcmd="/usr/bin/time -v"
    rss_key="Maximum resident"
    rss_in_bytes=false
    total_ram=$(grep MemTotal /proc/meminfo | awk '{print $2 * 1024}')
    ;;
esac

# ─── 3) Capture everything into a temp log ─────────────────────────────────
tmp=$(mktemp)
trap 'rc=$?; echo "❌ bench.sh failed (exit $rc); dumping log:" >&2; cat "$tmp" >&2; rm "$tmp"; exit $rc' ERR

export RISC0_PPROF_OUT="$PPROF_OUT"
export RISC0_PPROF_ENABLE_INLINE_FUNCTIONS=yes
export RISC0_DEV_MODE=1
export RISC0_INFO=1

# run host + zkVM + time, tee both to console and into $tmp
{
  $tcmd cargo run --release -p host -- \
    --function      "$FUNCTION" \
    --params        "$PARAMS" \
    --context-state "$CONTEXT_STATE" \
    --program-spec  "$PROGRAM_SPEC" \
    --value         "$VALUE"
} 2>&1 | tee "$tmp"

trap - ERR

# ─── 4) Parse host timing & RSS ────────────────────────────────────────────
wall=$(grep -Eo '^[0-9]+\.[0-9]+$$' "$tmp" \
       || grep -Eo '[0-9]+\.[0-9]+ real' "$tmp" | awk '{print $1}')
user=$(grep -Eo '[0-9]+\.[0-9]+ user' "$tmp"    \
       | grep -Eo '[0-9]+\.[0-9]+'             \
       | head -n1                              \
       || echo 0)

sys=$(grep -Eo '[0-9]+\.[0-9]+ sys' "$tmp"      \
      | grep -Eo '[0-9]+\.[0-9]+'               \
      | head -n1                                \
      || echo 0)
rss_raw=$(grep "$rss_key" "$tmp" | grep -Eo '[0-9]+' || echo 0)
if [ "$rss_in_bytes" = true ]; then
  rss_bytes=$rss_raw; rss_kb=$((rss_bytes/1024))
else
  rss_kb=$rss_raw;    rss_bytes=$((rss_kb*1024))
fi

# ─── 5) Parse RISC0 zkVM logs ───────────────────────────────────────────────
exec_time_ms=$(grep -Eo 'execution time: [0-9]+\.[0-9]+ms' "$tmp" \
               | awk '{print $3}' | sed 's/ms//')
segments=$(grep -Eo 'number of segments: [0-9]+' "$tmp" | awk '{print $4}')
rz_total_cycles=$(grep -Eo '[0-9]+ total cycles' "$tmp" | awk '{print $1}')
rz_user_cycles=$(grep -Eo '[0-9]+ user cycles' "$tmp" | awk '{print $1}')
rz_user_pct=$(grep -Eo '[0-9]+\.[0-9]+%\)' "$tmp" | head -n1 | sed 's/)//')
rz_paging_cycles=$(grep -Eo '[0-9]+ paging cycles' "$tmp" | awk '{print $1}')
rz_paging_pct=$(grep -Eo '[0-9]+\.[0-9]+%\)' "$tmp" | sed -n '2p' | sed 's/)//')
rz_reserved_cycles=$(grep -Eo '[0-9]+ reserved cycles' "$tmp" | awk '{print $1}')
rz_reserved_pct=$(grep -Eo '[0-9]+\.[0-9]+%\)' "$tmp" | sed -n '3p' | sed 's/)//')

# ─── 6) Compute host CPU% & MEM% ──────────────────────────────────────────
cpu_pct=$(awk -v u="$user" -v s="$sys" -v w="$wall" \
             'BEGIN{ if(w>0) printf("%.2f", (u+s)/w*100); else print "0" }')
mem_pct=$(awk -v r="$rss_bytes" -v t="$total_ram" \
             'BEGIN{ if(t>0) printf("%.2f", r/t*100); else print "0" }')

# ─── 7) Append one CSV row ────────────────────────────────────────────────
label="fn=$FUNCTION params=$PARAMS"
# human‑readable ISO timestamp for CSV
ts=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
# compact timestamp (no colons) for filenames
ts_file=$(date -u +"%Y%m%dT%H%M%SZ")

echo "$label,$wall,$user,$sys,$rss_kb,$rz_total_cycles,$cpu_pct,$mem_pct,$exec_time_ms,$segments,$rz_total_cycles,$rz_user_cycles,$rz_user_pct,$rz_paging_cycles,$rz_paging_pct,$rz_reserved_cycles,$rz_reserved_pct,$ts" >> "$CSV"

echo "✅  results/bench_results.csv ← $rz_total_cycles cycles (exec ${exec_time_ms} ms)"

rm "$tmp"

# ─── 8) Save the pprof file into the results folder ───────────────────────────
out_pb="${RESULTS_DIR}/profile_${FUNCTION//[^a-zA-Z0-9]/_}_${ts_file}.pb"
mv "${PPROF_OUT}" "$out_pb"
echo "✅  saved pprof data → $out_pb"

exit 0