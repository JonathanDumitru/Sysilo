#!/usr/bin/env bash
set -euo pipefail

MIN_GO_VERSION="${MIN_GO_VERSION:-1.22}"

if ! command -v go >/dev/null 2>&1; then
  echo "BLOCKED: Go toolchain is not installed or not in PATH."
  echo "Install Go ${MIN_GO_VERSION}+ and re-run this check."
  echo "Install help: https://go.dev/doc/install"
  exit 2
fi

go_version_output="$(go version 2>&1 || true)"
if [[ -z "${go_version_output}" ]]; then
  echo "FAIL: unable to read Go version from 'go version'."
  exit 1
fi

go_version_token="$(awk '{print $3}' <<<"${go_version_output}")"
go_version="${go_version_token#go}"
go_major="${go_version%%.*}"
go_minor="$(cut -d'.' -f2 <<<"${go_version}" | tr -dc '0-9')"

min_major="${MIN_GO_VERSION%%.*}"
min_minor="${MIN_GO_VERSION##*.}"

if [[ -z "${go_major}" || -z "${go_minor}" ]]; then
  echo "FAIL: unable to parse Go version from '${go_version_output}'."
  exit 1
fi

if (( go_major < min_major )) || (( go_major == min_major && go_minor < min_minor )); then
  echo "BLOCKED: Go ${go_version} detected, but ${MIN_GO_VERSION}+ is required."
  echo "Upgrade Go and re-run verification."
  exit 2
fi

echo "PASS: ${go_version_output}"
