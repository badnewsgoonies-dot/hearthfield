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
