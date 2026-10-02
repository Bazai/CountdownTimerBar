#!/usr/bin/env bash
# Makes every skill in .agents/skills visible to Claude Code through relative
# symlinks in .claude/skills, drops links whose skill is gone, and creates
# CLAUDE.md (an import of AGENTS.md) if it is missing.
set -euo pipefail

project_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
cd "$project_dir"

mkdir -p .claude/skills

for link in .claude/skills/*; do
    if [[ -L "$link" && ! -e "$link" ]]; then
        rm "$link"
        printf 'agents: removed dangling %s\n' "$link"
    fi
done

for skill in .agents/skills/*/; do
    name="$(basename "$skill")"
    ln -sfn "../../.agents/skills/$name" ".claude/skills/$name"
    printf 'agents: linked %s\n' "$name"
done

if [[ ! -e CLAUDE.md ]]; then
    printf '@AGENTS.md\n' > CLAUDE.md
    printf 'agents: created CLAUDE.md\n'
fi
