#!/usr/bin/env bash
set -u

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${ROOT_DIR}"

overall_status=0

run_step() {
  local label="$1"
  shift

  echo
  echo "== ${label} =="
  "$@"
  local rc=$?

  if [[ ${rc} -eq 0 ]]; then
    echo "PASS: ${label}"
  elif [[ ${rc} -eq 2 ]]; then
    echo "BLOCKED: ${label}"
    overall_status=2
  else
    echo "FAIL: ${label}"
    if [[ ${overall_status} -eq 0 ]]; then
      overall_status=1
    fi
  fi
}

run_step "Go toolchain preflight" bash scripts/verify/check_go_toolchain.sh

if command -v go >/dev/null 2>&1; then
  run_step "API gateway gap tests" go test ./services/api-gateway/... -run "SSO|SCIM|Router|Refresh|Breakglass|Team|Environment|RBAC|Connections|Routes"
else
  echo
  echo "BLOCKED: API gateway gap tests (Go not available)."
  overall_status=2
fi

run_step "Integration service targeted tests" cargo test --manifest-path services/integration-service/Cargo.toml connector_spec_contract -- --nocapture

if command -v pnpm >/dev/null 2>&1; then
  run_step "Frontend targeted tests" pnpm --filter web-app exec vitest --run --passWithNoTests api-config connections-service-rbac-path api-route-contract connections-metadata-source
else
  echo
  echo "BLOCKED: Frontend targeted tests (pnpm not available)."
  overall_status=2
fi

echo
if [[ ${overall_status} -eq 0 ]]; then
  echo "PHASE1_GAP_CLOSURE=PASS"
elif [[ ${overall_status} -eq 1 ]]; then
  echo "PHASE1_GAP_CLOSURE=FAIL"
else
  echo "PHASE1_GAP_CLOSURE=BLOCKED"
fi

exit "${overall_status}"
