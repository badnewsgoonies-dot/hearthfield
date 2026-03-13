# The Three Principles of Orchestrated Builds

Most autonomous build systems fail for the same reason: they optimize what can
be measured before they understand what matters. The fix is not "better
prompting." The fix is three architectural rules.

## 1. Green Is The Floor, Not The Ceiling

Whatever is green becomes the goal.

If the only things that can go green are compile, tests, and lint, then the
system will optimize for structural success even when the runtime experience is
broken.

That leads to the core distinction:

- Structural truth: the code compiles, types line up, tests pass, domains are wired.
- Experiential truth: the player can reach the feature, understand it, use it,
  trust it, and get the intended feedback.

Structural truth is required. It is not sufficient.

### The macro architecture

Every build should follow four large-scale phases:

1. Scaffold spine — Build the critical playable backbone: boot -> menu -> spawn
   -> movement -> first interaction -> first persistent state change.
2. Finish spine — The orchestrator personally traces each spine surface and
   turns it from "wired" into "real." Only verified behavior counts.
3. Scaffold breadth — Expand outward into more domains, systems, and content.
   This is parallel, worker-friendly, and structurally enforced.
4. Finish breadth — Return to sequential finishing. One player-facing surface at
   a time. Deep inspection. Graduation into tests.

This avoids building a giant green mannequin before you know whether the body
can move.

### The wave cadence

Each wave should run:

`Feature -> Gate -> Harden -> Graduate`

- Feature builds the thing
- Gate gets it standing
- Harden determines what it actually does when touched
- Graduate turns discoveries into permanent enforcement

Gate is a phase transition, not a checkpoint. It means the system is now stable
enough to examine. Not that it is finished.

## 2. The Audit Instruction Is A Gate Factory, Not A Gate

Prompted judgment is weak if it remains prose forever. It is strong if it fires
once, discovers the invariant, and then writes a test that enforces it from
then on.

That is the graduation principle:

`observation -> named invariant -> test`

The key move is timing. The system does not need perfect judgment forever. It
only needs enough judgment to identify what must be made mechanical.

### Why this matters

Without graduation, reality loses to structure. The compiler can go green. The
player path cannot. So the system keeps choosing green structure over
unverified reality.

Graduation fixes that by allowing player truth to become mechanically enforced
too.

### The rule

Only observed truths graduate.

Use three evidence levels:

- `[Observed]` — exact path traced and verified
- `[Inferred]` — strongly believed, not directly traced
- `[Assumed]` — design expectation, unverified

Only `[Observed]` claims should become named tests. If `[Inferred]` or
`[Assumed]` claims graduate, the system freezes guesses into machinery.

### Why some projects succeed

The difference is often not "better judgment." It is that one project converts
judgment into tests earlier.

That is why graduation matters more than raw intelligence: it changes truth
from something remembered into something enforced.

## 3. Freeze Shapes, Not Values

A frozen contract is necessary, but freezing too much too early turns guesses
into permanent reality.

The right rule is: freeze shapes, do not freeze tuning.

### Freeze shapes

Freeze:

- shared types
- enum variants
- event names
- function signatures
- equation forms
- runtime surfaces
- domain boundaries

These are structural agreements.

### Do not freeze values

Do not freeze:

- coefficients
- rates
- thresholds
- spawn chances
- pacing numbers
- balance tables

These belong in tunable data.

### Why this matters

A common failure mode is that a Phase 0 guess gets enshrined in the contract and
never re-examined. Once that happens, a wrong default can kill a gameplay loop
while still looking "correct" structurally.

The right architecture is:

- contract freezes the shape of the equation
- data files hold the coefficients
- playtesting adjusts the data
- the contract remains stable

That preserves integration without calcifying ignorance.

## Summary Doctrine

The whole methodology can be stated in four lines:

- Green means ready to examine, not ready to ship.
- Workers build the scaffold. The orchestrator finishes the game.
- Judgment should fire once per surface, then become a test.
- Freeze shapes, not values.

Everything else is procedure.
