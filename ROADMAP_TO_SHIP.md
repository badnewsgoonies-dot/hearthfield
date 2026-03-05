# ROADMAP TO SHIP — Hearthfield

## Guiding Principle
Fix in dependency order. Don't polish what's broken. Don't add content to systems that don't work.

---

## PHASE 1: PLAYABLE (Can a player complete Day 1-7 without confusion or soft-locks?)

**Status: ~80% done after current wave**

### Already Fixed (verified, gates passed)
- [x] Building door entry (4 doors wired)
- [x] Beach spawn soft-lock
- [x] Interior exit coords (stores were swapped)
- [x] NPC waypoints (12 conflicts, no more rooftop NPCs)
- [x] Duplicate starter items consolidated
- [x] Controls hint (intro + 60s HUD overlay)
- [x] Audio polish (toast SFX, footsteps, door sounds)

### In Progress (workers running)
- [ ] Building collision (players walk through walls) — worker dispatched
- [ ] Tutorial flow (exit house objective, shipping bin hint, Days 2-3) — worker dispatched
- [ ] Visual readability (font sizes, weather particles, mine clarity, stamina pulse) — worker dispatched

### Remaining Phase 1 Work
- [ ] Shop entry triggers GameState::Shop automatically (verify with shop audit)
- [ ] Eager atlas pre-loading (prevent fallback rectangles on first frame)
- [ ] Verify crafting bench interaction works end-to-end

**Exit criteria**: New player can start game → follow tutorial → plant/water/harvest → ship items → buy seeds from shop → sleep → repeat for 7 days. Zero soft-locks, zero confusion points.

---

## PHASE 2: COMPLETE LOOPS (Every core activity works start to finish)

### Farming Loop ✓ (verified working)
- Plant → water → grow → harvest → ship → gold
- Known limitation: quality lost on ship (contract issue, defer)

### Fishing Loop (needs verification)
- [ ] Cast → bite → minigame → catch → inventory → ship
- [ ] Verify fish_select works with location/season/time filters
- [ ] Verify fishing skill progression

### Mining Loop (needs verification)
- [ ] Enter mine → break rocks → collect ores → fight enemies → find ladder → descend
- [ ] Verify elevator unlocks at floors 5/10/15/20
- [ ] Verify knockout penalty (gold loss, teleport home)

### Crafting Loop (needs verification)
- [ ] Open crafting bench → select recipe → craft → get item
- [ ] Verify cooking at kitchen stove (requires house upgrade)
- [ ] Verify machine placement and processing (furnace, preserves jar, etc.)

### Social Loop (needs verification)
- [ ] Talk to NPC → friendship gain → gift items → relationship progression
- [ ] Verify romance flow (bouquet → dating → proposal → wedding)
- [ ] Verify quest system (accept → track → complete → reward)

### Economy Loop
- [ ] Buy seeds at General Store → farm → sell harvest
- [ ] Upgrade tools at Blacksmith → faster farming
- [ ] Upgrade buildings → more animals → more products

**Exit criteria**: Every activity loop completes without errors. Player can spend a full in-game year doing varied activities.

---

## PHASE 3: VISUAL COHERENCE (Game looks intentional, not broken)

### Atlas & Sprite Loading
- [ ] Eager-load ALL atlases during Loading state (not lazy on first use)
- [ ] Verify all 10 NPC sprites render correctly
- [ ] Verify crop growth sprites display (not colored dots)
- [ ] Verify animal sprites (5 have art; Duck/Goat/Pig/Rabbit/Horse need art or better fallbacks)

### Fallback Quality
- [ ] Replace colored rectangle fallbacks with outlined/labeled boxes (if no art available)
- [ ] Add item name labels to fallback rectangles
- [ ] Mine rocks: use atlas indices properly instead of flat colors
- [ ] Dialogue portrait: use NPC-colored silhouette instead of blue box

### UI Polish
- [ ] Button hover states (color change on selection)
- [ ] Toast categories (gold border for money, green for success, red for error)
- [ ] Menu transitions (fade in/out, not instant appear)
- [ ] Consistent font sizing across all screens

### Visual Juice
- [ ] Particles on harvest (leaf burst)
- [ ] Particles on tool impact (dirt splash for hoe, water droplets for can)
- [ ] Screen shake on pickaxe hit
- [ ] Bounce animation on item pickup
- [ ] Floating "+60g" text on shipping bin sale

**Exit criteria**: Screenshots look intentional. No colored rectangles visible in normal gameplay. UI feels responsive.

---

## PHASE 4: CONTENT & NARRATIVE (Game has personality and direction)

### Guided First Week
- [ ] Day 1: Farm basics (till, plant, water)
- [ ] Day 2: Harvest first crops
- [ ] Day 3: Ship items, earn gold, buy new seeds
- [ ] Day 4: Explore town, meet NPCs
- [ ] Day 5: Try fishing or mining
- [ ] Day 6-7: Open-ended with gentle hints

### Story Quests (5-10 hand-crafted)
- [ ] Mayor's Welcome Quest: "Bring me 5 turnips" (teaches farming loop)
- [ ] Margaret's Request: "Deliver bread to Doc" (teaches NPC interaction)
- [ ] Elena's Challenge: "Bring 10 copper ore" (teaches mining)
- [ ] Old Tom's Lesson: "Catch any fish" (teaches fishing)
- [ ] Community Goal: "Ship 100 items total" (teaches economy)

### NPC Depth
- [ ] 3+ dialogue tiers per NPC (stranger → friend → close friend)
- [ ] Birthday events with special dialogue
- [ ] Festival participation feels meaningful

### Seasonal Variety
- [ ] Each season has distinct crop options
- [ ] Visual seasonal changes noticeable (already working via tints)
- [ ] Festival on each season's festival day

**Exit criteria**: First 2 hours feel guided and engaging. Player has clear goals beyond "farm stuff."

---

## PHASE 5: SHIP (Deployable, saveable, shareable)

### Save System ✓ (verified 100% round-trip)
- All 37 data structures survive save/load

### WASM Build
- [ ] Run build_wasm.sh end-to-end
- [ ] Test in browser (Chrome, Firefox, Safari)
- [ ] Verify audio works in WASM (autoplay policies)
- [ ] Verify input works (keyboard, possibly touch)
- [ ] Test save/load in browser (localStorage or IndexedDB)

### Performance
- [ ] Profile frame time on target hardware
- [ ] Verify weather particle cap (600 max) prevents lag
- [ ] Test with full farm (all tiles planted, all animals)

### Final Gate
- [ ] Full year playthrough (112 days) without crash
- [ ] All 88+ headless tests passing
- [ ] Zero clippy warnings
- [ ] Save at each season boundary, load successfully
- [ ] WASM bundle < 50MB

**Exit criteria**: Playable build uploaded to itch.io. Runs in browser. Saves work.

---

## EXECUTION ORDER

```
PHASE 1 (current) ──→ PHASE 2 ──→ PHASE 3 ──→ PHASE 4 ──→ PHASE 5
  Playable           Loops work    Looks good   Has soul     Ships
  ~done              ~8 workers    ~6 workers   ~8 workers   ~3 workers
```

Total estimated: ~25 worker dispatches remaining across 4 phases.

Do NOT skip phases. A beautiful game that soft-locks is worse than an ugly game that works.
