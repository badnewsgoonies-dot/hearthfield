# .memory/ — Hearthfield Memory Store

## Artifact Types
- `observation` — directly verified in code, tests, or runtime
- `decision` — meaningful choice with alternatives
- `debt` — broken, missing, risky, or ungraduated work
- `principle` — reusable lesson from concrete experience

## Evidence Levels
- `[Observed]` — exact path traced and verified
- `[Inferred]` — strongly believed, not directly traced
- `[Assumed]` — expectation or hypothesis, unverified

Only [Observed] claims are trusted without extra verification.

## Write Triggers (only these fire)
1. Non-obvious decision made
2. Direct verification happened
3. Reusable principle emerged
4. New debt appeared
5. Contradiction surfaced
6. Correction invalidated a belief
7. Feel or surface-quality failure diagnosed
8. Graduation test created

## Naming
`{type}-{domain}-{short-slug}.yaml`

## Schema
```yaml
id: DEC-2026-03-10-001
type: decision | observation | debt | principle
evidence: Observed | Inferred | Assumed
domain: player | world | farming | fishing | mining | crafting | economy | ui | save | sailing | npc | animals | calendar
summary: "One sentence."
source_refs:
  - "file:hearthfield@src/path:lines"
  - "commit:hearthfield@sha"
status: active | resolved | superseded
supersedes: []
```

## Supersedes Rule
When wrong: create new artifact, add old ID to `supersedes`, mark old as `superseded`. Never silently mutate.

## Upgrade Path
At 50+ artifacts, build a briefing instead of loading everything.
