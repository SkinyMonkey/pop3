# RE Specifications

- `legacy/` — first-generation RE docs, written by weaker models. **Untrusted**: use
  only as an address/boundary map. Never implement from these without re-derivation.
- `v2/` — re-derived specs, one per subsystem, created just-in-time before that
  subsystem's rewrite. Every claim cites evidence: a binary address, a
  `ghidra-bridge dump-asm` snippet, or a capture fixture.

Trust ladder and workflow: see `PROGRESS.md` (root) and `tmp/RESTART.md`.
