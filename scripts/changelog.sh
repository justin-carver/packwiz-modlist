#!/usr/bin/env bash
# Renders the changelog section for a release and splices it into CHANGELOG.md.
#
# cargo-release runs this as a pre-release-hook with NEW_VERSION set. It also
# works by hand: scripts/changelog.sh 0.3.0
#
# Entries for 0.2.0 and earlier were written by hand and are left alone. Only
# the new section and the link-reference block at the bottom are rewritten.

set -euo pipefail

version="${NEW_VERSION:-${1:-}}"
if [[ -z "$version" ]]; then
    echo "usage: ${0##*/} <version>   (or set NEW_VERSION)" >&2
    exit 1
fi

cd "$(git rev-parse --show-toplevel)"

tag="v${version}"
changelog="CHANGELOG.md"
remote="https://github.com/justin-carver/sculkr"
# 0.1.1 predates the oldest tag still in the repo, so its compare link is anchored by hand.
oldest_anchor="v0.1.0"

if ! command -v git-cliff >/dev/null 2>&1; then
    echo "git-cliff is not installed. cargo install git-cliff" >&2
    exit 1
fi

if grep -q "^## \[${version}\]" "$changelog"; then
    echo "CHANGELOG.md already has a [${version}] entry; refusing to write a second one." >&2
    exit 1
fi

# Leading blank lines would otherwise push the heading away from the marker.
section="$(git-cliff --config cliff.toml --unreleased --tag "$tag" | sed -e '/./,$!d')"
if [[ -z "${section//[[:space:]]/}" ]]; then
    echo "git-cliff found no changelog-worthy commits since the last tag." >&2
    exit 1
fi

if [[ "${DRY_RUN:-false}" == "true" ]]; then
    echo "dry run, CHANGELOG.md unchanged. ${tag} would read:"
    echo
    echo "$section"
    exit 0
fi

tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT

awk -v section="$section" '
    { print }
    /^<!-- next-header -->$/ && !spliced { print ""; print section; spliced = 1 }
' "$changelog" >"$tmp"

# Rebuild every compare link from the versions the file now contains, so the
# chain stays correct no matter how many releases have accumulated.
mapfile -t versions < <(sed -n 's/^## \[\([0-9][^]]*\)\].*/\1/p' "$tmp")

{
    sed '/^<!-- next-url -->$/q' "$tmp"
    echo "[Unreleased]: ${remote}/compare/${tag}...HEAD"
    for i in "${!versions[@]}"; do
        if ((i + 1 < ${#versions[@]})); then
            previous="v${versions[i + 1]}"
        else
            previous="$oldest_anchor"
        fi
        echo "[${versions[i]}]: ${remote}/compare/${previous}...v${versions[i]}"
    done
} >"$changelog"

echo "CHANGELOG.md updated for ${tag}."
