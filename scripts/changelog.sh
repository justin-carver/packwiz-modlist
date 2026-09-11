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

# git-cliff always emits the version heading, so a release whose commits were all
# skipped (ci, chore, build, test) comes back as a heading and nothing under it.
# That is a legitimate maintenance bump, but it has to say so: an empty section
# would leave the GitHub release with no body at all.
if ! printf '%s\n' "$section" | grep -q '^### '; then
    section="${section}"$'\n\n'"_Maintenance release. No user-facing changes; see the commit log for build, CI and tooling work._"
    echo "No user-facing commits since the last tag; writing a maintenance-release note." >&2
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

# 0.1.0 and 0.1.1 were never tagged, so a compare range against them 404s.
# The version being released is not tagged yet either, since cargo-release runs
# this before it tags.
exists() {
    [[ "$1" == "$tag" ]] || git rev-parse -q --verify "refs/tags/$1" >/dev/null
}

{
    sed '/^<!-- next-url -->$/q' "$tmp"
    echo "[Unreleased]: ${remote}/compare/${tag}...HEAD"
    for i in "${!versions[@]}"; do
        current="v${versions[i]}"

        # No tag, no link. The heading then renders as plain text, which is
        # honest, where a dead link is not.
        exists "$current" || continue

        previous=""
        if ((i + 1 < ${#versions[@]})); then
            previous="v${versions[i + 1]}"
        fi

        if [[ -n "$previous" ]] && exists "$previous"; then
            echo "[${versions[i]}]: ${remote}/compare/${previous}...${current}"
        else
            echo "[${versions[i]}]: ${remote}/tree/${current}"
        fi
    done
} >"$changelog"

echo "CHANGELOG.md updated for ${tag}."
