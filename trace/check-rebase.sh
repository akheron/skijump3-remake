#!/bin/sh
# Usage: check-rebase.sh [branch] [onto]
# Rebases the trace branch (default sp4-trace) onto the SDL port's master (or
# <onto>) in a temporary worktree, and prints the conflicting files of each
# commit with the number of conflicting hunks. Conflicting hunks are resolved
# with the version of the trace branch, so that the rebase can go on to the
# next commit. No branch is changed, and the worktree is removed at the end.
. "$(dirname "$0")/env.sh"
branch=${1:-sp4-trace}; onto=${2:-master}
tmp=$(mktemp -d)
cleanup() {
  git -C "$SJ3_SDL" worktree remove --force "$tmp/wt"
  rm -rf "$tmp"
}
trap cleanup EXIT
git -C "$SJ3_SDL" worktree add -q --detach "$tmp/wt" "$branch" || exit 1
cd "$tmp/wt" || exit 1
git="git -c rerere.enabled=false -c core.editor=true"

echo "Rebasing $branch ($(git rev-parse --short HEAD)) onto $onto ($(git rev-parse --short "$onto"))"
hunks=0
$git rebase -q "$onto" > "$tmp/rebase.log" 2>&1
while git rev-parse -q --verify REBASE_HEAD > /dev/null; do
  files=$(git diff --name-only --diff-filter=U)
  if [ -z "$files" ]; then
    cat "$tmp/rebase.log"
    echo "Rebase stopped without conflicts"
    exit 1
  fi
  echo "$(git log -1 --format='%h %s' REBASE_HEAD)"
  for f in $files; do
    for stage in 1 2 3; do
      git show ":$stage:$f" > "$tmp/$stage" 2> /dev/null || {
        echo "  $f: not a content conflict"
        exit 1
      }
    done
    n=$(git merge-file -p "$tmp/2" "$tmp/1" "$tmp/3" | grep -c '^<<<<<<<')
    hunks=$((hunks + n))
    echo "  $f: $n"
    git merge-file -p --theirs "$tmp/2" "$tmp/1" "$tmp/3" > "$f"
    git add "$f"
  done
  $git rebase --continue > "$tmp/rebase.log" 2>&1
done
if [ -n "$(git status --porcelain)" ]; then
  cat "$tmp/rebase.log"
  echo "Rebase failed"
  exit 1
fi
echo "Rebased commits:"
git log --format='  %h %s' "$onto..HEAD"
echo "$hunks conflicting hunks"
