@AGENTS.md

# Claude Code Supplement

## Shared Contract

The imported `AGENTS.md` is the shared execution contract. This file adds only Claude Code-specific behavior and cannot weaken that contract or specialized normative documents. If they appear inconsistent, stop and report the conflict rather than choosing a preferred rule.

## Scoped Execution

Act as a scoped implementation agent. Begin with the active Issue or explicit task, confirming its result and requested side effects before editing. Follow the shared contract's routing, then inspect only the necessary files, symbols, history, tests, and configuration. Make the smallest coherent change and review generated work as rigorously as human-written work.

Do not explore broadly merely to appear thorough, perform opportunistic cleanup, build speculative abstractions or future infrastructure, or treat missing design as permission to generalize or refactor. Do not change instruction, memory, setting, hook, skill, or agent files unless the active task explicitly includes them.

## Claude Code Context

Project `CLAUDE.md` and its import provide guidance, not approval. User-level `CLAUDE.md`, local `CLAUDE.local.md`, auto memory, resumed-session context, and previous chat are non-authoritative personal or session context. Use them only when consistent with the active task and repository contracts; never turn their assumptions into durable policy or write repository policy into personal or local memory.

Do not modify project `CLAUDE.md` or `AGENTS.md` during ordinary implementation unless explicitly scoped. If a future task uses subagents or parallel research, bound each assignment and treat its output as evidence, never approval.

## Ambiguity and Stop Behavior

Do not resolve ambiguity through broad refactoring, new dependencies or abstractions, cloning, shared mutable state, async or concurrency choices, or public-boundary changes. Stop when unresolved ownership, architecture, API, compatibility, dependency, security, concurrency, async, or unsafe decisions block implementation. Preserve evidence, identify affected contracts, and offer realistic alternatives for maintainer resolution.

Clearly label approved facts, observed repository evidence, assumptions, proposals, and blockers. A plausible implementation, Claude's plan, generated code, or subagent output is not approval.

## Git and Pull Request Actions

Use Git or GitHub capabilities only when the active task explicitly requests and authorizes the specific action. Without that authorization, do not commit, amend or rewrite history, push or force-push, create or update a Pull Request, merge, tag, release, publish, or modify repository settings.

When authorized, inspect status and the final diff first, include only scoped files, preserve shared-history safeguards, and report results accurately. Stop if the action exceeds permission or scope. `AGENTS.md` and `.github/CONTRIBUTING.md` remain authoritative for shared workflow requirements.

## Validation and Final Report

Apply the shared validation and completion-report contract: run the active task's required validation and relevant existing checks, inspect the final file list and diff, and report every result honestly. Identify unavailable, skipped, or failed checks; distinguish completed, partial, and blocked outcomes; and report deviations, necessary adjacent changes, remaining risks, and unresolved decisions.

Keep the final response concise. Never claim a commit, push, Pull Request, merge, CI result, browser test, security test, or performance result that did not occur. Exclude private AI links and prompt transcripts from durable repository records.
