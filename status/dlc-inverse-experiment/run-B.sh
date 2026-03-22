#!/bin/bash
cd /home/claude/dlc-inverse
. ~/.env_tokens
mkdir -p outputs/B
timeout 120 copilot -p "# Mini-Module: Tavern Social System

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

Write tavern.rs to stdout. Include all types, logic, tests, and demo function in a single file."   --model claude-opus-4.6 -s > outputs/B/tavern.rs 2>outputs/B/stderr.log
echo "B done: $(wc -l outputs/B/tavern.rs) lines"
