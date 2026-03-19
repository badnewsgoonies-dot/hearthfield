# Paper v9 Additions — March 19, 2026

## New Finding: Structural vs Content Enforcement (INV-025)

A natural experiment emerged from testing Copilot CLI's `--deny-tool` flag.
The same mechanism (`--deny-tool "PATTERN"`) applied to two different tools
produced categorically different outcomes:

- `--deny-tool "write(*)"` — reliably blocked file writes (1/1)
- `--deny-tool "shell(*)"` — failed to block shell execution (0/1)

The write tool has a typed interface: one file path, one action. The deny
pattern matches the tool name and can inspect the argument structure. The
shell tool takes a single opaque string containing arbitrary commands. The
deny pattern matches the tool name but cannot inspect the command content.

This maps onto the paper's core thesis across six independent systems:

| Enforcement attempt                    | Mechanism     | Granularity                | Result           |
|----------------------------------------|---------------|----------------------------|------------------|
| CLAUDE.md "don't touch shared/"        | Prompt        | Content                    | 0/20             |
| disallowedTools: ["Task"]              | Config        | Structural (tool-level)    | 20/20            |
| --deny-tool "write(*)"                 | CLI flag      | Structural (discrete tool) | Blocks writes    |
| --deny-tool "shell(*)"                 | CLI flag      | Content (opaque tool)      | Fails            |
| Evidence tags                          | YAML metadata | Structural                 | 98% defense      |
| "Respond only in JSON"                 | Prompt        | Content                    | Unreliable       |
| responseMimeType: "application/json"   | API parameter | Structural                 | 100% enforcement |

None of these systems were designed as experiments. Each was built by a
different team at a different company solving a different problem. The
convergence is emergent: enforcement succeeds when the interface being
constrained is typed (the boundary can see the structure without
interpreting content), and fails when the interface is opaque.

Proposed principle: **Envelope enforcement succeeds; payload enforcement
fails.** Typed interfaces expose the envelope. Opaque interfaces hide it.
Decomposing a black box into typed operations is the prerequisite for
guardrails — not an optimization, a requirement.

## New Finding: Factory Tiering Economics (INV-026)

Copilot CLI model pricing creates a natural tiering for factory dispatch:

| Model            | Cost (Premium) | Optimal Use              |
|------------------|----------------|--------------------------|
| claude-haiku-4.5 | 0.33           | Read, audit, probe       |
| gpt-5.4-mini     | 0.33           | Read, audit, probe       |
| claude-sonnet-4.6| 1              | Code writing             |
| gpt-5.4          | 1              | Code writing             |
| claude-opus-4.6  | 3              | Complex reasoning only   |

At 0.33 premium per dispatch, audit workers cost 1/3 of code-writing
workers. A 9-worker inventory batch at Haiku costs the same as 3 Sonnet
workers. This changes factory economics: the read/audit/probe phase
(which produces the specifications workers implement against) becomes
nearly free relative to the implementation phase.

Validated: One Haiku 4.5 worker at 0.33 premium produced a complete
type catalog of a 2,353-line Rust module — 77 bounded fields, 10
lifecycle chains, 14 entity definitions — in a single dispatch.

## New Finding: Ironclad Integration Results (INV-027)

The Ironclad proc-macro crate (3 macros, 442 lines) was integrated into
a 60K LOC Bevy game codebase (Hearthfield) and wired into production
game structs across a single session using factory workers.

### What was built
- `#[game_value(min, max)]` — bounded newtypes with runtime validation
- `#[game_lifecycle(A -> B -> C)]` — typestate transitions
- `#[game_entity(requires = [X, Y, Z])]` — typed builders

### Integration results
8 bounded types wired across 25+ files:

| Type          | Struct       | Fields wired              | Callsites changed |
|---------------|--------------|---------------------------|--------------------|
| Health        | PlayerState  | health                    | 2                  |
| Stamina       | PlayerState  | stamina                   | 11                 |
| Gold          | PlayerState  | gold                      | 20+                |
| Happiness     | Animal       | happiness                 | 10+                |
| MineFloor     | MineState    | current_floor, deepest    | 12+                |
| BuildingLevel | AnimalState  | coop_level, barn_level    | 8+                 |
| Friendship    | Relationships| friendship (HashMap vals) | 15+                |
| StackSize     | ItemDef      | stack_size                | 10+                |

### Key discoveries during wiring

1. **f32 incompatibility caught by compiler.** The initial game_value
   macro derived Eq, Ord, and Hash — which f32 doesn't implement. The
   compiler rejected this immediately. Fixed in one line. This is the
   thesis: the compiler catches the bug, not code review.

2. **Deref is necessary but insufficient.** Deref to the inner type
   makes Display and method calls transparent. But comparisons (<, >=),
   arithmetic (+, -), compound assignment (+=, -=), and casts (as f32)
   all require explicit .get() or *deref. Of 50 callsites audited,
   only 8 (16%) were transparent via Deref. The remaining 84% needed
   mechanical changes.

3. **Namespace collision in lifecycle macros.** Two lifecycle types
   both generated a `Basic` marker struct, causing E0428. Solved by
   putting each lifecycle in its own submodule. The compiler caught
   the collision; no human review needed.

4. **Gold cap spec conflict.** The bounded type Gold(max=999999) was
   incompatible with an achievement test that set gold to 1,000,000.
   The Haiku audit worker flagged this before any code was written.
   Cap raised to 9,999,999. The audit-first pattern prevented a
   runtime regression.

5. **Workers respected the constraint boundary.** Across 8 wiring
   dispatches (3 Codex, 5 Copilot Sonnet), zero workers modified
   the ironclad crate or bounded_types.rs beyond the instructed
   changes. The type contract boundary held mechanically.

### Wiring process
- Phase 1: Haiku audit (0.33 premium) produces per-callsite migration plan
- Phase 2: Sonnet/Codex workers (1 premium each) execute migrations
- Phase 3: cargo check confirms zero errors
- Each wired type is a permanent ratchet — no future worker can regress it

The total cost for wiring 8 types across 25+ files: approximately
8-10 premium requests (1 audit + 7-8 implementation workers).

## Anthropic Introspection Paper Connection (expanded)

The Lindsey et al. (2025) finding that models have ~20% functional
introspective awareness provides mechanistic grounding for three of
our findings:

1. **Evidence tags work because structural cues activate reliable
   detection.** Models notice the presence/absence of tags (structural
   cue) even when they can't reliably reason about tag content.

2. **Reversed labels break defense because models follow labels
   mechanically.** The 8% residual defense on reversed labels
   corresponds to the ~20% where introspection fires — the model
   notices something is wrong but can't overcome the structural cue.

3. **Tool-forced architecture sidesteps introspection entirely.**
   When the model physically can't answer without reading a file,
   introspective reliability is irrelevant. The architecture
   compensates for the capability gap.

The Ironclad integration extends this to compile-time: the compiler
is a more reliable verifier than the model. Models chase green builds
(20/20 with mechanical enforcement). Every green build now means every
game design constraint is satisfied. The review burden shifts from
"read the code" to "check if it compiled."
