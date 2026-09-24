#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -P "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APPLY=0
CLEAN_WORKSPACE=0
CLEAN_BINARIES=0
VERBOSE=0
CANDIDATES=()

usage() {
  cat <<'USAGE'
Usage: scripts/disk-clean-safe.sh [--apply] [--build-binaries | --workspace] [--verbose]

Dry-run by default. Stop builds and tests before applying cleanup.
Only regeneratable outputs inside this Git workspace are eligible:
  - example .nuis/cache directories (other project state is retained)
  - tools/*/target, crates/*/target, example bundles and the optional preview build
  - root debug/release incremental state and the maintenance Python cache
  - --build-binaries: also remove hashed debug/release build/test executables;
    keep root CLI binaries, dependency libraries and Cargo registry downloads
  - --workspace: instead remove the complete root target directory

Every candidate is checked before any deletion. Tracked files and symlinked
paths are refused. No sibling projects, subprojects, user caches, shared temp
directories, Docker state, Git history or package sources are touched.
USAGE
}

for arg in "$@"; do
  case "$arg" in
    --apply) APPLY=1 ;;
    --workspace) CLEAN_WORKSPACE=1 ;;
    --build-binaries) CLEAN_BINARIES=1 ;;
    --verbose) VERBOSE=1 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unsupported cleanup option: $arg" >&2; usage >&2; exit 2 ;;
  esac
done

GIT_ROOT="$(git -C "$ROOT_DIR" rev-parse --show-toplevel)"
if [ "$(cd -P "$GIT_ROOT" && pwd)" != "$ROOT_DIR" ] || [ ! -f "$ROOT_DIR/Cargo.toml" ]; then
  echo "refusing cleanup outside the workspace Git root" >&2
  exit 1
fi

add_candidate() {
  if [ -e "$1" ] || [ -L "$1" ]; then
    CANDIDATES+=("$1")
  fi
}

if [ "$CLEAN_WORKSPACE" -eq 1 ]; then
  add_candidate "$ROOT_DIR/target"
else
  for profile in debug release; do
    add_candidate "$ROOT_DIR/target/$profile/incremental"
    if [ "$CLEAN_BINARIES" -eq 1 ]; then
      for path in "$ROOT_DIR/target/$profile/deps/"*; do
        name="${path##*/}"
        if [[ "$name" =~ ^[[:alnum:]_-]+-[0-9a-f]{16}(\.exe)?$ ]] && [ -f "$path" ]; then
          if [ -x "$path" ] || [[ "$name" == *.exe ]]; then
            add_candidate "$path"
          fi
        fi
      done
    fi
  done
fi

for path in "$ROOT_DIR"/tools/*/target "$ROOT_DIR"/crates/*/target; do
  add_candidate "$path"
done
add_candidate "$ROOT_DIR/tools/yir-preview-macos/build"
add_candidate "$ROOT_DIR/scripts/tests/__pycache__"
for path in "$ROOT_DIR"/examples/bins/*; do
  [ "${path##*/}" = README.md ] || add_candidate "$path"
done
if [ -d "$ROOT_DIR/examples/projects" ] && [ ! -L "$ROOT_DIR/examples/projects" ]; then
  while IFS= read -r -d '' path; do
    add_candidate "$path/cache"
  done < <(find "$ROOT_DIR/examples/projects" -type d -name .nuis -prune -print0)
fi

check_candidate() {
  local path="$1" relative parent tracked
  case "$path" in
    "$ROOT_DIR"/*) relative="${path#"$ROOT_DIR"/}" ;;
    *) echo "refusing outside-workspace path: $path" >&2; return 1 ;;
  esac
  parent="$path"
  while [ "$parent" != "$ROOT_DIR" ]; do
    if [ -L "$parent" ]; then
      echo "refusing symlinked cleanup path: $relative" >&2
      return 1
    fi
    parent="${parent%/*}"
  done
  tracked="$(git -C "$ROOT_DIR" ls-files -- "$relative")"
  if [ -n "$tracked" ]; then
    echo "refusing tracked cleanup path: $relative" >&2
    return 1
  fi
  if ! git -C "$ROOT_DIR" check-ignore -q -- "$relative"; then
    echo "refusing non-ignored cleanup path: $relative" >&2
    return 1
  fi
}

for path in "${CANDIDATES[@]-}"; do
  [ -n "$path" ] || continue
  check_candidate "$path"
done

count=0
total_kib=0
for path in "${CANDIDATES[@]-}"; do
  [ -n "$path" ] || continue
  size="$(du -sk "$path" | cut -f 1)"
  count=$((count + 1))
  total_kib=$((total_kib + size))
  if [ "$VERBOSE" -eq 1 ]; then
    printf '%s KiB\t%s\n' "$size" "${path#"$ROOT_DIR"/}"
  fi
done
printf 'Selected %s workspace outputs (%s KiB before cleanup; hardlinks may share storage).\n' "$count" "$total_kib"
if [ "$APPLY" -eq 0 ]; then
  echo 'Dry-run only. Use --verbose to inspect paths; add --apply to delete.'
  exit 0
fi

for path in "${CANDIDATES[@]-}"; do
  [ -n "$path" ] || continue
  check_candidate "$path"
  rm -rf -- "$path"
done
echo 'Workspace output cleanup complete.'
df -h "$ROOT_DIR"
