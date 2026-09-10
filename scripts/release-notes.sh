#!/usr/bin/env bash
# Prints the CHANGELOG.md section for a version, and fails if it has none.
#
# The release workflow runs this twice: once in the metadata job as an early
# gate, and again in the publish job to build the GitHub release body. Gating
# early matters because crates.io publishing cannot be undone, only yanked.

set -euo pipefail

version="${1:-}"
if [[ -z "$version" ]]; then
    echo "usage: ${0##*/} <version> [changelog]" >&2
    exit 1
fi
changelog="${2:-CHANGELOG.md}"

notes="$(awk -v ver="$version" '
    index($0, "## [" ver "]") == 1 { grab = 1; next }
    grab && /^## \[/ { exit }
    grab { print }
' "$changelog" | sed -e '/./,$!d')"

if [[ -z "${notes//[[:space:]]/}" ]]; then
    echo "${changelog} has no notes under [${version}]." >&2
    exit 1
fi

printf '%s\n' "$notes"
