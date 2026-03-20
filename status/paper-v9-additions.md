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

## The Verification Cost Invariant

### The Mechanism

Verification has exactly two costs: acquiring the information needed to check a claim, and evaluating whether that information satisfies the constraint. These are the only two operations any verification system performs, regardless of domain.

When a constraint is declared at the interface — a type bound, an evidence tag, a schema annotation, a tool permission — both costs collapse to zero for external verifiers. The information is already exposed (no acquisition cost) and the evaluation is structural matching against a declared specification (no interpretation cost). A compiler checking `Health(u32)` against `min=0, max=999` performs a mechanical comparison. A reviewer checking an evidence tag against its cited source performs a mechanical comparison. Neither requires understanding the content that produced the value.

When a constraint is hidden in content — a bare `u32` that "should" be between 0 and 999, a model's reasoning that "should" be grounded in sources, a shell command that "should not" delete files — neither cost is bounded. The verifier must first extract the constraint from the content (reverse-engineering intent from behavior), then evaluate whether the content satisfies the constraint it just extracted. Both operations require interpreting the content itself, and content interpretation has no guaranteed upper bound on cost or convergence.

### The Impossibility Claim

No system can cheaply verify undeclared constraints. This is not an empirical observation but a structural impossibility. Consider any candidate counterexample: a system that achieves cheap verification of a constraint that is not declared at the interface. For the verifier to check the constraint, it must first know what the constraint is. If the constraint is not at the interface, the only place it can exist is in the content. To extract it from the content, the verifier must inspect the content. Content inspection cost is unbounded — it requires the verifier to reconstruct the reasoning that produced the content, with no guarantee of arriving at the same constraint the author intended. Therefore no cheap verification of undeclared constraints exists. The absence of counterexamples across 1,831 trials, 7 independent systems, and 8 non-software domains is consistent with this impossibility, but the claim does not rest on the empirical record. It rests on the structure of the cost function.

### The Convergence Explanation

This impossibility explains why independent teams converge on the same architecture. The Rust compiler team did not study billing systems. The billing team did not study tool-permission frameworks. The evidence tag experiments were not informed by JSON schema enforcement. Yet all arrived at the same structural choice: declare constraints at the interface, enforce them mechanically, make violations impossible rather than detectable.

They converged because the constraint landscape has one basin of attraction. If you are building a system that must verify claims — about types, about sources, about permissions, about data shapes — declared constraints are the only place where verification cost is bounded. Every other point in the design space has unbounded verification cost. This is what makes the convergence inevitable rather than coincidental: there is nowhere else to arrive.

### Empirical Illustrations

The following systems were built independently by different teams solving different problems. None referenced each other. All arrived at the same structural choice.

| System | Undeclared (fails) | Declared (succeeds) | Verification cost |
|--------|-------------------|--------------------|--------------------|
| CLAUDE.md "don't touch shared/" | Prompt in content | `disallowedTools: ["Task"]` in config | 0/20 → 20/20 |
| Copilot `--deny-tool "write(*)"` | `--deny-tool "shell(*)"` (opaque) | `--deny-tool "write(*)"` (typed) | Fails → succeeds |
| Evidence tags | Model's internal reasoning | ~200 chars YAML at interface | 0% → 96% correct |
| `responseMimeType` | "Respond only in JSON" (prompt) | `responseMimeType: "application/json"` | Unreliable → 100% |
| Ironclad `#[game_value]` | Bare `u32` (opaque) | `Health(u32)` with min/max | Runtime trace → compile-time |
| TOML sprite manifests | Hardcoded paths in code | Manifest entries validated by `build.rs` | Play-test → build-time |
| Tool-forced architecture | Context-window provenance | Empty context + remote store | Prompt bypass → 100% defense |

The bare `u32` example is load-bearing. A `u32` and a `Health(u32)` have the same memory footprint, the same runtime performance, the same content. The only difference is whether the constraint is visible to external inspection. One requires whole-system tracing to verify. The other is checked at zero marginal cost by the compiler. The difference is not in the data. It is in whether the constraint is declared at the interface.

This connects to the evidence tag finding at the deepest level. A model's internal reasoning is a bare `u32` — opaque, might be correct, cannot be checked without re-deriving the entire chain. An evidence tag is a `Health(u32)` — the same content, but the constraint is now at the interface where any external system (another model, a human reviewer, a mechanical checker) can verify it without opening the box. The 0% → 96% result is not surprising under this framing. The tags did not make the model smarter. They made its claims inspectable.

### Scope Limitation

We describe this as an invariant, not a law. The theoretical argument — verification cost is bounded if and only if constraints are declared at the interface — is structurally sound and we cannot construct a counterexample. The empirical record is consistent across 7 independent systems, 8 non-software domains, and 1,831 experimental trials. However, all empirical data originates from a single research program. Multi-repository validation across 3–5 diverse codebases from independent authors is the recommended next step. If the invariant holds under external replication, the stronger "law" designation would be warranted. Until then, "invariant" is the honest word: same mathematical weight, transparent about scope.
