#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SUPPORTED_PROVIDERS=(
  "anthropic"
  "openai"
  "google"
  "openrouter"
  "xai"
  "deepseek"
)

declare -A PROVIDER_KEY_ENV=(
  ["anthropic"]="ANTHROPIC_API_KEY"
  ["openai"]="OPENAI_API_KEY"
  ["google"]="GOOGLE_API_KEY"
  ["openrouter"]="OPENROUTER_API_KEY"
  ["xai"]="XAI_API_KEY"
  ["deepseek"]="DEEPSEEK_API_KEY"
)

usage() {
  cat <<'EOF'
Usage: tests/run_e2e.sh [--provider=<name>] [--record]

Runs live provider E2E harness with CI_E2E_TESTS=1 and writes timestamped
artifacts under tests/e2e_results/<timestamp>/.

Options:
  --provider=<name>   Run a single provider only (anthropic|openai|google|openrouter|xai|deepseek)
  --record            Copy recorded VCR cassettes into the artifact directory
  -h, --help          Show this help
EOF
}

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Missing required command: $cmd" >&2
    exit 1
  fi
}

provider_supported() {
  local provider="$1"
  local p
  for p in "${SUPPORTED_PROVIDERS[@]}"; do
    if [[ "$p" == "$provider" ]]; then
      return 0
    fi
  done
  return 1
}

provider_configured() {
  local provider="$1"
  local key_var="${PROVIDER_KEY_ENV[$provider]}"
  if [[ -n "${!key_var:-}" ]]; then
    return 0
  fi

  if [[ -f "$MODELS_PATH" ]]; then
    if jq -e --arg provider "$provider" '.providers[$provider] != null' "$MODELS_PATH" >/dev/null 2>&1; then
      return 0
    fi
  fi
  return 1
}

load_api_key_from_models() {
  local provider="$1"
  local key_var="${PROVIDER_KEY_ENV[$provider]}"
  if [[ -n "${!key_var:-}" ]]; then
    return 0
  fi
  if [[ ! -f "$MODELS_PATH" ]]; then
    return 0
  fi

  local api_key
  api_key="$(jq -r --arg provider "$provider" '.providers[$provider].apiKey // empty' "$MODELS_PATH" 2>/dev/null || true)"
  if [[ -n "$api_key" ]]; then
    export "$key_var=$api_key"
  fi
}

PROVIDER_FILTER=""
RECORD_MODE=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --provider=*)
      PROVIDER_FILTER="${1#*=}"
      shift
      ;;
    --provider)
      [[ $# -ge 2 ]] || { echo "--provider requires a value" >&2; exit 1; }
      PROVIDER_FILTER="$2"
      shift 2
      ;;
    --record)
      RECORD_MODE=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage
      exit 1
      ;;
  esac
done

if [[ -n "$PROVIDER_FILTER" ]] && ! provider_supported "$PROVIDER_FILTER"; then
  echo "Unsupported provider '$PROVIDER_FILTER'" >&2
  usage
  exit 1
fi

require_cmd cargo
require_cmd jq

MODELS_PATH="${PI_MODELS_PATH:-$HOME/.pi/agent/models.json}"
TIMESTAMP="$(date -u +%Y%m%dT%H%M%SZ)"
OUT_DIR="${ROOT}/tests/e2e_results/${TIMESTAMP}"
mkdir -p "$OUT_DIR"

RUNNER_LOG="${OUT_DIR}/runner.log"
SUMMARY_TSV="${OUT_DIR}/summary.tsv"
SUMMARY_JSON="${OUT_DIR}/summary.json"
printf "provider\tverdict\tstatus\telapsed_ms\tinput_tokens\toutput_tokens\ttotal_tokens\testimated_cost_usd\ttotal_cost_usd\tcost_source\tartifacts_dir\n" > "$SUMMARY_TSV"

TARGET_PROVIDERS=()
if [[ -n "$PROVIDER_FILTER" ]]; then
  load_api_key_from_models "$PROVIDER_FILTER"
  if ! provider_configured "$PROVIDER_FILTER"; then
    echo "Provider '$PROVIDER_FILTER' is not configured in env or ${MODELS_PATH}" >&2
    exit 1
  fi
  TARGET_PROVIDERS+=("$PROVIDER_FILTER")
else
  for provider in "${SUPPORTED_PROVIDERS[@]}"; do
    load_api_key_from_models "$provider"
    if provider_configured "$provider"; then
      TARGET_PROVIDERS+=("$provider")
    fi
  done
fi

if [[ "${#TARGET_PROVIDERS[@]}" -eq 0 ]]; then
  echo "No configured providers found. Set env keys or configure ${MODELS_PATH}." >&2
  exit 1
fi

{
  echo "run_started_utc=${TIMESTAMP}"
  echo "models_path=${MODELS_PATH}"
  echo "provider_filter=${PROVIDER_FILTER:-<none>}"
  echo "record_mode=${RECORD_MODE}"
  echo "providers=${TARGET_PROVIDERS[*]}"
} > "$RUNNER_LOG"

fail_count=0
pass_count=0
skip_count=0
attempted_count=0

for provider in "${TARGET_PROVIDERS[@]}"; do
  provider_dir="${OUT_DIR}/${provider}"
  mkdir -p "$provider_dir"

  cargo_output="${provider_dir}/cargo_output.log"
  test_log="${provider_dir}/test_log.jsonl"
  artifact_index="${provider_dir}/test_artifacts.jsonl"
  export_dir="${provider_dir}/export"
  mkdir -p "$export_dir"

  echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] running provider=${provider}" | tee -a "$RUNNER_LOG"

  set +e
  if [[ "$RECORD_MODE" -eq 1 ]]; then
    CI_E2E_TESTS=1 \
      PI_LIVE_E2E_PROVIDER="$provider" \
      PI_E2E_EXPORT_DIR="$export_dir" \
      TEST_LOG_JSONL_PATH="$test_log" \
      TEST_ARTIFACT_INDEX_PATH="$artifact_index" \
      VCR_MODE=record \
      cargo test --test e2e_live_harness e2e_live_provider_harness_smoke -- --nocapture \
      >"$cargo_output" 2>&1
  else
    CI_E2E_TESTS=1 \
      PI_LIVE_E2E_PROVIDER="$provider" \
      PI_E2E_EXPORT_DIR="$export_dir" \
      TEST_LOG_JSONL_PATH="$test_log" \
      TEST_ARTIFACT_INDEX_PATH="$artifact_index" \
      cargo test --test e2e_live_harness e2e_live_provider_harness_smoke -- --nocapture \
      >"$cargo_output" 2>&1
  fi
  cargo_status=$?
  set -e

  cat "$cargo_output" | tee -a "$RUNNER_LOG" >/dev/null

  if [[ "$RECORD_MODE" -eq 1 && -f "$test_log" ]]; then
    cassette_dir="${provider_dir}/cassettes"
    mkdir -p "$cassette_dir"
    while IFS= read -r cassette_path; do
      [[ -z "$cassette_path" ]] && continue
      [[ -f "$cassette_path" ]] || continue
      cp "$cassette_path" "${cassette_dir}/$(basename "$cassette_path")"
    done < <(jq -r '
      select((.schema == "pi.test.log.v1" or .schema == "pi.test.log.v2") and .type == "log")
      | .context.vcr_path // empty
    ' "$test_log")
  fi

  results_file="${export_dir}/live_provider_results.contract.jsonl"
  costs_file="${export_dir}/live_provider_costs.jsonl"

  status="unknown"
  elapsed_ms="n/a"
  input_tokens="n/a"
  output_tokens="n/a"
  total_tokens="n/a"
  estimated_cost="n/a"
  total_cost="n/a"
  cost_source="n/a"

  if [[ -f "$results_file" ]]; then
    status="$(jq -r --arg provider "$provider" 'select(.provider == $provider) | .status' "$results_file" | tail -n 1)"
    elapsed_ms="$(jq -r --arg provider "$provider" '
      select(.provider == $provider)
      | (.elapsedMs // .elapsed_ms)
    ' "$results_file" | tail -n 1)"
    input_tokens="$(jq -r --arg provider "$provider" 'select(.provider == $provider) | .usage.input' "$results_file" | tail -n 1)"
    output_tokens="$(jq -r --arg provider "$provider" 'select(.provider == $provider) | .usage.output' "$results_file" | tail -n 1)"
    total_tokens="$(jq -r --arg provider "$provider" '
      select(.provider == $provider)
      | (.usage.totalTokens // .usage.total_tokens)
    ' "$results_file" | tail -n 1)"
  fi

  if [[ -f "$costs_file" ]]; then
    estimated_cost="$(jq -r --arg provider "$provider" '
      select(.provider == $provider)
      | (.estimatedCostUsd // .estimated_cost_usd // "n/a")
    ' "$costs_file" | tail -n 1)"
    total_cost="$(jq -r --arg provider "$provider" '
      select(.provider == $provider)
      | (.totalCostUsd // .total_cost_usd // "n/a")
    ' "$costs_file" | tail -n 1)"
    cost_source="$(jq -r --arg provider "$provider" '
      select(.provider == $provider)
      | (.costSource // .cost_source // "n/a")
    ' "$costs_file" | tail -n 1)"
  fi

  verdict="FAIL"
  if [[ "$status" == "skipped" ]]; then
    verdict="SKIP"
    skip_count=$((skip_count + 1))
  elif [[ "$status" == "passed" && "$cargo_status" -eq 0 ]]; then
    verdict="PASS"
    pass_count=$((pass_count + 1))
    attempted_count=$((attempted_count + 1))
  else
    verdict="FAIL"
    fail_count=$((fail_count + 1))
    attempted_count=$((attempted_count + 1))
  fi

  printf "%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n" \
    "$provider" \
    "$verdict" \
    "$status" \
    "$elapsed_ms" \
    "$input_tokens" \
    "$output_tokens" \
    "$total_tokens" \
    "$estimated_cost" \
    "$total_cost" \
    "$cost_source" \
    "$provider_dir" \
    >> "$SUMMARY_TSV"
done

tail -n +2 "$SUMMARY_TSV" | jq -R -s '
  split("\n")
  | map(select(length > 0))
  | map(split("\t"))
  | map({
      provider: .[0],
      verdict: .[1],
      status: .[2],
      elapsed_ms: (.[3] | tonumber?),
      input_tokens: (.[4] | tonumber?),
      output_tokens: (.[5] | tonumber?),
      total_tokens: (.[6] | tonumber?),
      estimated_cost_usd: (.[7] | tonumber?),
      total_cost_usd: (.[8] | tonumber?),
      cost_source: .[9],
      artifacts_dir: .[10]
    })
' > "$SUMMARY_JSON"

echo
echo "E2E Summary (${OUT_DIR})"
if command -v column >/dev/null 2>&1; then
  column -t -s $'\t' "$SUMMARY_TSV"
else
  cat "$SUMMARY_TSV"
fi

echo
echo "Summary JSON: ${SUMMARY_JSON}"
echo "Runner log:   ${RUNNER_LOG}"

if [[ -n "$PROVIDER_FILTER" ]]; then
  if [[ "$fail_count" -gt 0 || "$attempted_count" -eq 0 ]]; then
    exit 1
  fi
  exit 0
fi

# Exit code contract: success only when all attempted providers passed.
if [[ "$fail_count" -eq 0 && "$attempted_count" -gt 0 ]]; then
  exit 0
fi

exit 1
