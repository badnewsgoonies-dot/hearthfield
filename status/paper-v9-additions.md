# Paper v9 Additions — March 19-20, 2026

## INV-025: Structural enforcement degrades at opaque tool boundaries

**Finding:** The same enforcement mechanism (`--deny-tool`) succeeds or fails
depending on whether the target tool exposes a typed interface or an opaque one.

**Setup:** GitHub Copilot CLI v1.0.9, `claude-sonnet-4.6`, identical prompts.

| Command | Tool type | Interface | Result |
|---------|-----------|-----------|--------|
| `--deny-tool "write(*)"` | File write | Typed (path, content) | Blocked |
| `--deny-tool "shell(*)"` | Shell execution | Opaque (arbitrary string) | Not blocked |

**Analysis:** The `write` tool has a discrete, typed interface — the enforcement
boundary sees `write(path="/tmp/test.txt")` and can match the pattern without
interpreting the content. The `shell` tool takes an arbitrary command string — the
boundary sees `shell(command="echo hello")` but cannot determine whether the
command is dangerous without parsing and understanding bash.

This is the same mechanism that explains every enforcement result in this paper:

| Enforcement attempt | Mechanism | Interface type | Outcome |
|---------------------|-----------|---------------|---------|
| CLAUDE.md "don't touch shared/" | Prompt instruction | Content (opaque) | 0/20 |
| `disallowedTools: ["Task"]` | Config file | Structural (typed) | 20/20 |
| `--deny-tool "write(*)"` | CLI flag | Structural (typed) | Blocks |
| `--deny-tool "shell(*)"` | CLI flag | Content (opaque) | Fails |
| "Respond only in JSON" | Prompt instruction | Content (opaque) | Unreliable |
| `responseMimeType: "application/json"` | API parameter | Structural (typed) | 100% |
| Evidence tags (~200 chars YAML) | Inline metadata | Structural (typed) | 98% |

**The law:** You can constrain what you can see. Enforcement operates on the
envelope (typed interface) or the payload (opaque content). Envelope enforcement
succeeds. Payload enforcement fails.

Seven independent examples across tools built by different teams at different
companies (Anthropic, GitHub, Google, OpenAI) converge on the same boundary.
This is not a coincidence — it is a structural property of enforcement at
tool interfaces.

**Evidence level:** Observed (direct experiment, reproducible)
**Replication:** Single trial per condition, but the pattern holds across 7
independent systems. The converging evidence across unrelated tools constitutes
stronger validation than repeated trials on a single system.

---

## INV-026: VLM evaluation of rendered game output (Godogen loop)

**Finding:** A vision language model (Gemini 3 Flash) correctly evaluates
rendered game screenshots against natural-language assertions, enabling
automated visual QA without human review.

**Setup:**
- Bevy 0.15 battle scene rendered via software renderer (Mesa llvmpipe, xvfb)
- Screenshot captured programmatically (Screenshot::primary_window())
- Gemini 3 Flash via Vertex AI with schema-enforced JSON output
- `responseMimeType: "application/json"` + `responseSchema` (API-level guarantee)

**Result:** 4/4 assertions PASS at 1.0 confidence:

| Assertion | Result | Evidence from VLM |
|-----------|--------|-------------------|
| Player character visible | PASS | Identified player sprite by position |
| Enemy sprites visible | PASS | Named all three: Green Slime, Bat, Rock Crab |
| HP bars present | PASS | Located four HP bars with correct positions |
| Action menu at bottom | PASS | Identified Attack, Defend, Item, Flee |

**Key observation:** The VLM identified enemies by name despite never seeing
the source code, asset manifest, or entity definitions. It derived the names
purely from the rendered pixel art. This validates the Godogen principle:
the verification agent sees only pixels, preventing self-bias.

**Pipeline performance:** 4 seconds from rendered frame to structured verdict.
Render (software) + screenshot capture + Gemini API call + JSON parse = 4s total.

**Quality gating application:** The same pipeline was used to evaluate 160
generated sprites (137 enemies + 23 djinn). Imagen 3 generated sprites;
Gemini evaluated quality on a 1-10 scale with schema-enforced JSON.
65% PASS rate on first generation; REDO sprites queued for regeneration
with refined prompts. Total generation time for 160 sprites: ~60 minutes.

**Architectural significance:** This closes the verification loop that the
paper identifies as the fundamental problem with AI-generated code. Models
cannot self-verify (INV-009 through INV-013). External verification requires
ground truth. For game code, the ground truth is what appears on screen.
The VLM provides automated access to that ground truth without human review.

The full chain: manifest (TOML) → Imagen 3 (generate) → bg removal →
downscale → Gemini eval (PASS/REDO) → asset directory → build.rs validates →
sprite_loader loads → battle_scene renders → screenshot captures → VLM asserts.

Every link is mechanical. The human writes the manifest. Everything else
is automated.

**Evidence level:** Observed (direct experiment, single scene, reproducible)
**Limitation:** Single scene type (battle). Needs validation on overworld,
interior, farm, and menu scenes. The 1.0 confidence on all assertions may
reflect the simplicity of the test scene — complex scenes with occlusion,
animation, or ambiguous layout may produce lower confidence.

---

## INV-027: Bounded types surface pre-existing bugs in AI-generated code

**Finding:** Retrofitting bounded types onto an existing AI-generated codebase
does not just prevent future bugs — it surfaces existing ones that were invisible
under bare primitive types.

**Setup:** Two Rust/Bevy game codebases:
- Hearthfield: 64,750 LOC, 812 commits, 60K+ lines written by AI workers
- Vale Village v3: 15,778 LOC, 136 commits, 231 tests

Three proc macros applied:
- `#[game_value(min = 0, max = 999)]` — bounded newtypes (e.g., Health, Gold)
- `#[game_lifecycle(Seed -> Sprout -> Mature)]` — typestate transitions
- `#[game_entity(requires = [Name, Position, Sprite])]` — typed builders

**Results:**

| Metric | Hearthfield | Vale Village | Combined |
|--------|-------------|-------------|----------|
| Bounded types wired | 8 | 14 | 22 |
| Callsites scanned | 327 | 228 | 555 |
| Runtime paths upgraded | 29 | 37 | 66 |
| Computed paths clamped | — | 50 | 50 |
| Real bugs found | 3 (Gold overflow) | Same class likely | 3+ |
| Tests after wiring | compile clean | 231 pass | all green |

**The Gold overflow bug:** Three callsites in Hearthfield where
`player.gold + reward` could exceed the logical maximum (9,999,999).
Under bare `u32`, the arithmetic silently succeeded with an invalid
game state. Under `Gold` (bounded 0-9,999,999), the type system forced
every callsite to confront whether it handled the upper bound. Three
weren't. Cost to find: 0.99 premium (3 Haiku audit dispatches).

**The new_unchecked pattern:** Across both repos, 555 callsites used
`new_unchecked()` (bypasses validation) and 0 used `new()` (validates).
This is safe for constants but dangerous for runtime values. The audit
identified 66 runtime paths that needed upgrading to `new()` with error
handling — save file data, player input, computed combat results.

**Mechanism:** The bounded types work because AI workers chase green
builds with 100% reliability. If the design rule IS a compiler rule,
then a green build means all design rules are satisfied. This converts
game design constraints from documentation (which workers ignore) to
compilation requirements (which workers cannot ignore).

This is the same envelope/payload principle from INV-025: the type
signature is the envelope, the runtime value is the payload. The
compiler enforces the envelope. The bounded type makes the payload
visible at the envelope layer by encoding the valid range in the type.

**f32 edge case:** The macro initially derived Eq, Ord, and Hash
unconditionally. f32 does not implement these traits. The compiler
caught this immediately — the macro was fixed with conditional derives.
This is the system working as designed: the compiler catches the bug
in the enforcement mechanism itself.

**Evidence level:** Observed (both repos, 555 callsites, 3 confirmed bugs)
**Replication:** Two independent codebases, same macro crate, same pattern.

---

## Updated Unified Architecture Table

One principle at every layer: structured metadata → mechanical enforcement →
impossible violations.

| Layer | Metadata | Enforcement | Violation |
|-------|----------|-------------|-----------|
| Memory | Evidence tags (YAML) | Tag presence/absence | Cannot cite untagged claims |
| Orchestration | Dispatch configs (CSV) | spawn_agents_on_csv | Cannot skip rows |
| Code types | Bounded newtypes (Rust) | Compiler | Cannot use invalid values |
| Code lifecycle | Typestate (PhantomData) | Compiler | Cannot skip states |
| Code entities | Required fields (Builder) | Compiler | Cannot omit fields |
| Assets | Manifest (TOML) | build.rs | Cannot ship missing sprites |
| Tool permissions | deny-tool (typed tools) | CLI flag | Cannot call denied tools |
| Output format | responseMimeType (JSON) | API parameter | Cannot return non-JSON |
| Verification | VLM assertions (schema) | Gemini + responseSchema | Cannot return unstructured |
| Visual QA | Godogen loop (pixels) | Separate vision agent | Cannot self-verify |

The human decides what the constraints are. The system makes violating them
physically impossible. The dividing line is always the same: enforcement
works when the interface is typed (envelope), fails when it is opaque (payload).

**"You can constrain what you can see."**

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

No system can cheaply verify undeclared constraints. This is not an empirical observation but a structural impossibility. Consider any candidate counterexample: a system that achieves cheap verification of a constraint that is not declared at the interface. For the verifier to check the constraint, it must first know what the constraint is. If the constraint is not at the interface, the only place it can exist is in the content. To extract it from the content, the verifier must inspect the content. Content inspection cost is unbounded — it requires the verifier to reconstruct the reasoning that produced the content, with no guarantee of arriving at the same constraint the author intended. Therefore no cheap verification of undeclared constraints exists. Apparent counterexamples where content spaces are small enough to enumerate reduce to implicit constraint declaration by the type system — a `bool` is a declared constraint, a `u8` used as a bool is not. The absence of counterexamples across 1,831 trials, 7 independent systems, and 8 non-software domains is consistent with this impossibility, but the claim does not rest on the empirical record. It rests on the structure of the cost function.

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

---

## Method Note: Multi-Session Adversarial Review

Part Nine was refined through a multi-session relay: drafts were passed
between four independent Claude sessions with no shared conversation
state. Each session engaged with the prior draft as received text,
applied genuine critique, and returned a revised version.

Improvements produced by this process:
- The impossibility proof was tightened from an informal argument to a
  contradiction proof (extracting a constraint from content without
  inspecting content)
- Scope was calibrated from "law" to "invariant" — the theoretical
  claim is sound but the empirical base is one research program
- A boolean gap in the evidence tag analysis was caught and corrected
- The convergence explanation was sharpened from "not coincidence" to
  "single fixed point in the cost landscape"

This is the same mechanism that improves code quality in multi-agent
dispatch: fresh context, no accumulated momentum, genuine engagement
with the artifact rather than the conversation history. The method
generalizes from code review to prose review. The sessions did not
know about each other. The quality came from each session verifying
declared constraints (the draft text) at a typed interface (the chat
input) without needing to reconstruct the full provenance chain.

**Evidence level:** Observed (single instance, four sessions, one section)
**Limitation:** n=1. The process improved one section of one paper.
Generalization to other writing tasks is plausible but unmeasured.
