# Quick Start — Every Hearthfield Session

## Session Start

1. Read [HEARTHFIELD_BUILD_METHODOLOGY.md](/home/geni/swarm/hearthfield/docs/HEARTHFIELD_BUILD_METHODOLOGY.md)
2. Read [.memory/STATE.md](/home/geni/swarm/hearthfield/.memory/STATE.md)
3. State what you are working on

The agent should then fire the first-response protocol:

- tier
- surface
- macro phase
- wave phase
- debt
- evidence level

## Before Touching a Domain

Run:

```bash
git log --oneline -15 -- <path>
```

Check:

- matching `.memory/*.yaml` artifacts if they exist
- latest worker report for that domain if one exists

Then state:

- what changed recently
- what is unresolved
- what is still `[Inferred]` / `[Assumed]`

## During Work

Follow the wave:

`Feature -> Gate -> Document -> Harden -> Graduate`

At the `Document` phase, emit artifacts to `.memory/` only for:

- non-obvious decision
- direct verification
- reusable principle
- new debt
- contradiction or correction
- feel failure
- graduation test

## Session End

1. Update [.memory/STATE.md](/home/geni/swarm/hearthfield/.memory/STATE.md) with:
   - current phase
   - debts
   - decisions
   - gate status
2. If `.memory/` changed:

```bash
git add .memory/
git commit -m "memory: update state and artifacts"
```

## What This Gets You

- every session starts fresh
- every session sees current state
- decisions, debts, and principles accumulate across sessions
- evidence levels prevent adopting stale or fabricated claims
- git history provides episodic memory for free

Nothing to install. The system is files on disk.
