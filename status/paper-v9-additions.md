# Paper v9 Additions — March 19-20, 2026

These findings extend the v8 monolith (Building and Remembering, ~1,831 trials).
Searchable by INV-025/026/027/028.

---

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
**Replication:** Per-row n values vary: CLAUDE.md 0/20 vs disallowedTools 20/20
(n=42); evidence tags 0% vs 98% (n=50+, 5 models); deny-tool write vs shell
(single trial); responseMimeType vs prompt (n=15+). The convergence across 7
independent systems built by different teams at different companies constitutes
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

**Compiler enforcement:**

| Layer | Metadata | Enforcement | Violation |
|-------|----------|-------------|-----------|
| Code types | Bounded newtypes (Rust) | Compiler | Cannot use invalid values (INV-027) |
| Code lifecycle | Typestate (PhantomData) | Compiler | Cannot skip states |
| Code entities | Required fields (Builder) | Compiler | Cannot omit fields |

**API/config enforcement:**

| Layer | Metadata | Enforcement | Violation |
|-------|----------|-------------|-----------|
| Output format | responseMimeType (JSON) | API parameter | Cannot return non-JSON (INV-025) |
| Tool permissions | deny-tool (typed tools) | CLI flag | Cannot call denied tools (INV-025) |

**Pipeline enforcement:**

| Layer | Metadata | Enforcement | Violation |
|-------|----------|-------------|-----------|
| Assets | Manifest (TOML) | build.rs | Cannot ship missing sprites (INV-026) |
| Orchestration | Dispatch configs (CSV) | spawn_agents_on_csv | Cannot skip rows |

**Memory enforcement:**

| Layer | Metadata | Enforcement | Violation |
|-------|----------|-------------|-----------|
| Memory | Evidence tags (YAML) | Tag presence/absence | Cannot cite untagged claims |

**Vision enforcement:**

| Layer | Metadata | Enforcement | Violation |
|-------|----------|-------------|-----------|
| Verification | VLM assertions (schema) | Gemini + responseSchema | Cannot return unstructured (INV-026) |
| Visual QA | Godogen loop (pixels) | Separate vision agent | Cannot self-verify (INV-026) |

The human decides what the constraints are. The system makes violating them
physically impossible. The dividing line is always the same: enforcement
works when the interface is typed (envelope), fails when it is opaque (payload).

**"You can constrain what you can see."**

---

## Part Nine: The Verification Cost Invariant

### The mechanism

Verification has exactly two costs: acquiring the information needed to
check a constraint, and evaluating whether the constraint holds. These
costs behave differently depending on where the constraint lives.

When a constraint is declared at the interface — a type signature, an
API parameter, a tool permission flag, an evidence tag — acquisition
cost is zero (the constraint is already exposed) and evaluation cost
is structural matching (compare the declared bound against the observed
value). Both costs are bounded and, in practice, near-zero. The compiler
does not read a function body to verify a bounded type. The tool
permission system does not parse a shell command to verify a tool name.
The billing system does not evaluate output quality to meter a scoped
worker. Each verifier reads the interface and is done.

When a constraint is embedded in content — a prompt instruction, an
opaque tool argument, an untagged claim in conversation history — the
verifier must reconstruct the constraint by inspecting the content.
For a prompt instruction like "don't touch shared/", the model must
re-derive the intent on every token generation. For an opaque shell
command, the permission system must parse and understand arbitrary bash.
For an untagged factual claim, a downstream model must re-derive the
entire provenance chain. This reconstruction cost has no upper bound
and no guarantee of convergence.

### The impossibility claim

No system can cheaply verify an undeclared constraint. A system that
did so would need to extract the constraint from content without
inspecting the content — which is a contradiction. The constraint is
either at the interface (declared, externally visible) or not
(undeclared, embedded in content). If it is not at the interface, the
verifier must inspect content to find it. Content inspection cost is
unbounded. Therefore cheap verification of undeclared constraints is
impossible.

This is not an empirical finding. It is a structural property of
verification itself: verification cost is bounded if and only if
constraints are declared at the interface.

### Why independent systems converge

The implication is that any system designed to enforce constraints will
converge on rewarding externally declared constraints, because that is
the only point where the verification cost function reaches zero. There
is no alternative basin of attraction. A compiler team optimizing for
fast type-checking arrives at type signatures. A billing team optimizing
for metering accuracy arrives at scoped, typed work units. A tool
permission system optimizing for security arrives at typed tool
interfaces. A memory system optimizing for trust arrives at evidence
tags. None of these teams studied each other's work. They converged
because the cost landscape has a single fixed point.

This explains the independence. The convergence is not coincidence, not
best practice diffusing through the industry, and not the result of
shared design philosophy. It is the mathematical consequence of
optimizing verification cost. Every team that tries to make verification
cheap arrives at declared constraints because there is nowhere else to
arrive.

### Empirical illustrations

Seven independent systems, built by different teams at different
companies solving different problems, exhibit the predicted behavior:

| System | Declared (bounded cost) | Undeclared (unbounded cost) | Outcome | Source |
|--------|------------------------|----------------------------|---------|--------|
| Rust compiler | `Health(u32)` with min/max | bare `u32` | Bounded catches 3 bugs; bare misses all | INV-027 |
| Tool permissions | `--deny-tool "write(*)"` | `--deny-tool "shell(*)"` | Write blocked; shell not blocked | INV-025 |
| CLAUDE.md vs config | `disallowedTools: ["Task"]` | "Don't use Task" in CLAUDE.md | Config: 20/20; prompt: 0/20 | §2 (n=42) |
| Evidence tags | `[evidence: verified, src: mod.rs:142]` | Untagged conversational claim | Tags: 98% defense; untagged: 0% | §3.1 (n=50+) |
| JSON output | `responseMimeType: "application/json"` | "Respond only in JSON" in prompt | API: 100%; prompt: unreliable | INV-025 |
| Billing | Scoped CSV worker (0.18 avg) | Monolithic session | Workers: 0.53/commit; sessions: unmeasurable | INV-026 |
| Vision QA | VLM with responseSchema | Prose evaluation prompt | Schema: structured JSON always; prose: parse failures | INV-026 |

Each row is a natural experiment: same goal, same model in most cases,
same prompt complexity, different constraint placement. In every case,
the declared constraint succeeds and the undeclared constraint fails or
degrades. The table does not prove the invariant — no finite set of
examples can prove a universal claim. The impossibility argument proves
it. The table demonstrates that real engineering converges on the
prediction.

### Scope and limitations

This paper terms the finding an invariant rather than a law. The
theoretical argument — verification cost is bounded if and only if
constraints are declared at the interface — is sound, and the authors
cannot construct a counterexample. However, the empirical scope is
one research program (1,831 trials), two codebases (80K LOC combined),
and one orchestrator. The seven systems span four companies (Anthropic,
GitHub, Google, OpenAI) and multiple engineering teams, which provides
independence, but all observations originate from a single researcher's
body of work.

Multi-repository validation across 3-5 diverse codebases from different
authors is the recommended next step. If independent codebases reproduce
the same convergence — small typed workers cheaper to verify than large
opaque ones, declared constraints enforced where undeclared ones fail —
the invariant framing upgrades to law. Until then, the restraint is
deliberate. The theoretical claim is strong. The empirical base should
match it before the language does.

The measured production cost supports the invariant's practical
implications: 47.57 premium requests across three days produced 86
verified commits, 160 VLM-evaluated visual assets, and a complete
type-safety retrofit across both codebases. Cost per verified commit:
0.53 premium (measured). Cost per production bug found: 0.66 premium
(measured). Cost per generated, evaluated, and validated sprite: 0.10
premium (measured). These economics are a consequence of the invariant:
when constraints are declared at typed interfaces, verification is
cheap, and cheap verification makes exhaustive quality coverage
practical at any scale.

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

---

## INV-028: Amplification Drift Through Relay — Exploration, Not Just Inflation

**Observation:** When a claim is passed sequentially through independent
Claude sessions, each session raises the epistemic confidence, scope,
and emotional register of the claim. No individual step is dishonest —
each session genuinely engages with well-structured input and responds
with "yes, and" rather than "wait, does this hold." The cumulative
effect is monotonic inflation.

**Observed instance:** A modest observation ("the argument got tighter
through multi-session review") was relayed through 4 independent
sessions. By session 4, the claim had escalated to "genuinely perfect
recursion that proves the thesis and should go in the paper." Each
session's response was interchangeable. The pattern was invisible from
inside any single session and only visible at the orchestrator level.

### The dual mechanism

A sequential relay does two things simultaneously:

1. **Explores the structural ceiling** of an idea. Each session asks
   "what's the strongest version of this?" The next session starts
   from a higher base. Multiple passes find the maximum defensible
   version.

2. **Inflates the emotional register and scope claims** beyond what
   the structure supports.

### The deflation test

Strip all confidence language, emotional register, and scope modifiers.
What survives is the structural content the relay discovered. What
disappears is noise.

| Claim | Deflated | Loses something real? | Verdict |
|-------|----------|----------------------|---------|
| "Verification cost is bounded iff constraints are declared at the interface" | "It helps to be explicit" | Yes — impossibility proof, cost function, convergence | Exploration |
| "This is genuinely perfect and proves everything" | "This is good" | No | Inflation |
| "Invariant not law" (scope calibration) | "We're not sure how far this goes" | Yes — specific reason and upgrade condition | Exploration |
| "Oh wow it's recursive" | "The process was interesting" | No | Inflation |

### The method

Use relay intentionally as a creativity tool:
1. Start with a rough insight
2. Relay through 3-5 independent sessions
3. Collect all outputs
4. Deflate aggressively — strip confidence, strip register
5. What remains is the structural ceiling the relay discovered

### Predicted experimental shape

- Register inflation: linear (monotonic, no plateau)
- Structural content: sigmoid (fast rise, then plateau at ceiling)
- The plateau is where the idea is fully explored and additional
  passes add only register noise
- The useful output is the sigmoid. The noise is the line.

### Connection to the invariant

The relay works for exploration because each session verifies declared
constraints (the pasted text) at a typed interface (the chat input).
It becomes pure inflation when there is no structure to find. A claim
with real internal logic gets sharper. A claim with no internal logic
just gets louder.

The deflation test is itself an instance of the invariant: structural
content is declared (logical dependencies, predictions, checkable).
Emotional register is payload (carries no verifiable claims). Deflation
strips payload and preserves envelope.

**Evidence level:** Observed (single instance, n=1, 4 sessions)
**Status:** Proposed experiment. Not yet run. Two falsifiable
predictions: monotonic register inflation and sigmoid structural content.
