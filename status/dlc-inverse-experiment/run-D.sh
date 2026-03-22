#!/bin/bash
cd /home/claude/dlc-inverse
. ~/.env_tokens
mkdir -p outputs/D
timeout 120 codex exec --dangerously-bypass-approvals-and-sandbox --skip-git-repo-check -C /home/claude/dlc-inverse   "# Mini-Module: Tavern Social System

Build a self-contained Rust module implementing a tavern social interaction system for an RPG.

## Requirements
- A `Tavern` struct with name, available NPCs (3-5), and current time_of_day
- An `NpcProfile` struct: name, mood (Happy/Neutral/Grumpy), topics (vec of strings), affinity (0-100)
- A `Conversation` system: player picks an NPC, picks a topic, gets a response influenced by mood + affinity
- Topic responses: at least 3 topics per NPC, with mood-variant text
- Affinity changes: good topic match → +5, bad match → -10, neutral → +1
- A `buy_drink` action that improves mood by one tier (Grumpy→Neutral→Happy) for 10 gold
- Save/load: serialize and deserialize all tavern state via serde
- At least 5 unit tests covering: conversation flow, affinity changes, mood transitions, buy_drink, save/load roundtrip

## Deliverables
- `tavern.rs` — all types and logic
- Tests inline or in a tests submodule
- A `demo()` function that simulates a player visiting the tavern: enter, talk to NPC, buy drink, talk again, save, load, verify state

## Constraints
- Pure Rust, no external dependencies beyond serde/serde_json
- Must compile with `rustc --edition 2021` or `cargo check`
- No placeholder/todo implementations — every function must be complete

Write a complete tavern.rs file. Include all types, logic, tests, and demo function."   > outputs/D/raw.log 2>&1
grep -v "^codex\|^Running\|^exec\|tokens used" outputs/D/raw.log > outputs/D/tavern.rs 2>/dev/null
echo "D done: $(wc -l outputs/D/tavern.rs) lines"
