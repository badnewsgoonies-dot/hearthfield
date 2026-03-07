# Early Hearthfield Commit Playbook (Applied to DLC)

This note distills the first ~40 non-merge commits in Hearthfield and turns them into a reusable orchestration sequence.

## Observed Winning Sequence

1. Contract and stubs first.
- `feat: scaffold ... type contract + domain stubs`
- Parallel work became viable immediately after shared type surface existed.

2. Parallel domain waves.
- Repeated `wave N` commits filled domain modules quickly.
- Throughput came from many scoped lanes, not monolithic commits.

3. Phase contract extensions before new feature families.
- Before big feature jumps (Phase 3/4), contract additions landed first.
- This reduced semantic drift and integration ambiguity.

4. Hardening and tests after each expansion burst.
- Critical fixes appeared right after big waves.
- Dedicated test expansion and warning cleanup commits stabilized velocity.

5. Infrastructure polish after core stability.
- Input abstraction, visual unification, and deployment pipeline came after baseline gameplay loop and correctness.

## DLC Execution Template

Use this order per milestone:
1. `C0`: freeze contract bible (`CONTRACT.md`).
2. `S0`: scaffold/adjust module surfaces against frozen names.
3. `W1..Wn`: parallel feature lanes with strict file allowlists.
4. `I0`: integration lane reconciles seams and naming.
5. `H0`: hardening lane for regressions and compiler pressure.
6. `T0`: expand deterministic and regression tests.
7. `P0`: polish/content pass.

## Practical Guardrails

- Never start a wave without contract references in task packets.
- Keep investigation lanes running during implementation lanes.
- Gate each wave on compile + deterministic checks + scope enforcement.
- If drift appears, pause lanes, patch contract bible, re-dispatch.
