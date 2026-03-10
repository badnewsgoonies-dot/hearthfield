# 01 — Research Foundation: "The Model Is the Orchestrator"

Load this FIRST. Every constraint in the playbook traces to a finding here.

## Paper Identity
Geni, February 2026. Corpus: 295M tokens, 98 sessions, 11 builds, 8 controlled experiments.

## The Four Core Findings

### 1. Statefulness Premium (the dominant cost)
- ~95% of orchestrator input tokens are re-reading prior conversation
- The orchestrator is expensive because of ARCHITECTURE, not judgment
- Simulating statefulness in a stateless protocol IS the cost
- Reasoning tokens do NOT re-enter context (confirmed 550-turn analysis)
- **Implication**: minimize orchestrator turns, use disk-based state

### 2. Scope Enforcement (the mechanical imperative)
- Prompt-only: **0/20** under compiler pressure (100% failure)
- Mechanical (git checkout post-hoc): **20/20** (100% success)
- Production: 84.2% clean (but low-pressure conditions)
- "You don't ask a saw to only cut certain wood — you clamp the piece you want cut"
- **Implication**: NEVER rely on telling workers to "only edit X" — let them edit freely, then `clamp-scope.sh` reverts out-of-scope changes

### 3. Type Contracts (shared vocabulary, not gatekeeper)
- Not required for integration at any scale tested (6-36 modules)
- Read-only integration at 36 modules: 3/3 passed, zero errors
- BUT no-contract ablation → 6 structurally incompatible Unit interfaces (false positive: compiled because domains never referenced each other)
- A 984-line contract written blind held across 10 domains with 1 error
- **Implication**: freeze the contract before dispatch, verify with checksum

### 4. Context Priming (presence is the mechanism)
- Cold (no context): **0/10** formula transfer
- Static document: **10/10** formula transfer
- Conversational dialogue: **10/10** formula transfer
- Self-generation hypothesis falsified — static docs work identically
- **Implication**: specs on disk, not in prompts. Workers read full specs. Format doesn't matter, presence does.

## External Validation and Feedback Integration

### The A/B/C Memory Ablation Framing
- New cognitive-science feedback identifies the decisive experiment as a
  three-condition ablation:
  - `A`: accumulated compaction / conversational carry-forward
  - `B`: fresh session + untyped retrieval
  - `C`: fresh session + typed retrieval with provenance
- The research program here already partially ran this structure:
  - Exp 3 / Exp 6 correspond to `A` (compacted summary adopted as truth)
  - Exp 8 corresponds to `B` (sequential / untyped note retrieval)
  - Exp 9 corresponds to `C` (typed artifacts with evidence and source refs)
- **Implication**: future write-ups should make this mapping explicit, because it
  aligns the existing results with the exact experiment outside feedback says
  the field still needs.

### Memory Is Not Neutral Storage
- The strongest conceptual addition from the new feedback is:
  **if memory changes persona, memory pipelines must be treated like
  behavior-shaping training data**
- In this framing, false remembered claims are not just retrieval failures.
  They become self-reinforcing evidence streams that can alter the agent's
  effective identity and decision posture.
- This gives a stronger interpretation of the provenance result:
  provenance tags do not merely block false factual adoption; they also
  constrain identity drift by forcing claims back through source quality.

### External References Worth Carrying Forward
- `ENGRAM`: supports typed / partitioned memory over monolithic stores
- `MemoryArena`: warns that external memory / RAG can underperform long-context
  when retrieval representation is mismatched to what the model needs
- `PI-LLM`: supports the proactive-interference account — semantically similar
  earlier material can overpower nearby correct information
- `Hindsight`: closest published architecture to typed memory with confidence,
  but notably lacks strict storage gating
- identity-drift work: larger models can drift more, and accumulated memory may
  function as a self-reinforcing identity signal

### Operational Synthesis
- The existing architecture still stands:
  typed artifacts + provenance + fresh reconstruction beat transcript replay
- But the new framing sharpens the threat model:
  memory integrity is not only about retrieval correctness; it is also about
  preserving stable evaluative identity under accumulation pressure

## Extended Findings

### Bare-Prompt Ablation
- Strong claim (models discover coordination independently): **FALSIFIED**
- Solo Opus given tools + goal: wrote everything itself, never delegated
- Solo throughput: ~325 LOC/min, invariant to project size
- Solo outperforms pyramid below ~10 domains

### Worker Model as First-Order Variable
- Same architecture, same spec: 9.8x output gap between best and worst worker models
- Architecture enables parallel throughput, worker model DETERMINES it

### Scaling
- Zero integration errors across 50 domain builds with type contract
- Sweet spot: 10 workers (2.05x speedup, Amdahl's law 44% serial fraction)
- Cost scales linearly, throughput sublinearly

### Pyramid Architecture
- L1 frontier (suspended, 3-5 turns) → L2 mid-tier (manages domains) → L3 cheap (implements)
- Inverts statefulness premium: intelligence × fewest turns = minimum cost
- Depth is scaling mechanism, not quality mechanism
- Below 10 domains: flat dispatch strictly superior
- Above 10 domains: hierarchy enables parallelism, coordination overhead inverts to savings

### Compaction Recovery
- Zero relapse across 11 compaction events
- Pattern: "state, then verify" — output expectation then read disk
- Invest in MANIFEST.md quality (task IDs, phase, decisions, blockers)

## Failure Modes
1. **Abstraction reflex** (~17 instances): builds orchestration infrastructure instead of orchestrating
2. **Self-model error** (~7): claims it lacks capabilities it has
3. **Identity paradox**: can't hold orchestrator + worker roles simultaneously
4. **Delegation compression**: each delegation level is lossy; quantities die first (80 → 8 weapons through 3 levels)
   - Fix: workers read full spec from filesystem, not prompt summaries

## Open Program Extensions

- Add an explicit A/B/C section to the long-form monolith:
  - `A`: compaction
  - `B`: untyped fresh retrieval
  - `C`: typed fresh retrieval
- Frame the poisoning-defense result as partially answering the exact external
  challenge proposed in the cognitive-science feedback
- Extend Part 7 / “What Remains Open” to include:
  - whether typed retrieval still wins when retrieval representation is
    mismatched (MemoryArena concern)
  - whether provenance structure resists identity drift under long-term memory
    accumulation
  - whether storage gating (`Observed` graduation only) is the critical advantage
    over architectures like Hindsight that accumulate broadly

## Build #11: Vale Village
- 3 sentences of human input → 26,200 LOC Rust/Bevy, 394 tests, 13 self-directed waves
- Model read the paper's own findings and cited them while orchestrating
- Paper as boot image — research initializes its own described system
