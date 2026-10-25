# Moonitor repository instructions

## Branch workflow

The repository owner requested on 23 October 2026 that ongoing work happen
directly on `main`. Use `main`; do not create feature branches or pull requests
unless the owner explicitly asks for them. The blueprint's separate-branch
checkpoint described the original migration and does not override this decision.

Before changing or publishing work, inspect the branch, worktree and remote
divergence. Preserve existing local changes. Fetch and reconcile safely before
pushing; never force-push or discard unrelated work. Keep commits reviewable.

## Product and verification

- Read `docs/moonitor-professional-upgrade-blueprint.md` and the relevant audit or
  qualification report for substantial work. Verify claims against the code.
- Keep Iced + wgpu unless a measured limitation justifies an architectural change.
- The supplied V3 UI specification in `docs/moonitor-v2/reference/` replaces the
  old W1 layout. Fleet is the landing view; use a compact rail, comparative matrix,
  single-host resource cockpit and persistent GUI setup. The former 3D renderer is
  retained for regression only. Follow the V2 kickoff's first vertical slice.
- The owner replaced the sage/green direction with lunar and stellar themes:
  Moonlight and Deep Space. Use `palette.rs` for shared UI, chart and scene colors.
- Missing, unsupported, stale and zero are different states. Preserve metric
  provenance, energy scope and the distinction between loaded models and requests.
- Never put synthetic telemetry into production views. Fixtures belong in tests.
- For code changes, run the relevant checks and the UI/interaction verification
  affected by the change. Compilation alone does not qualify the interface.
- For documentation-only changes, verify links, assets, commands and rendered
  structure; do not add redundant Rust tests for prose.
- Do not commit `.local/`, private fleet configuration, databases or credentials.
- Report what was measured, what passed and which acceptance gates remain open.
