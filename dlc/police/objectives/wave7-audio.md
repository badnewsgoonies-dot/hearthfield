# Wave 7: Audio — Precinct DLC

## CRITICAL: After each step, mentally walk through what a player hears. If an action is silent, it's not done.

## Read first:
- dlc/police/src/shared/mod.rs (PlaySfxEvent, PlayMusicEvent)
- dlc/police/src/domains/ui/mod.rs (current stub readers for audio events)

## Deliverables

### 1. SFX System
- Read PlaySfxEvent, play corresponding sound via Bevy AudioPlayer
- Map event names to .ogg files in dlc/police/assets/audio/sfx/
- Key SFX: menu_click, case_assigned, evidence_collected, dispatch_call, toast_pop, 
  door_open, footstep, coffee_pour, shift_end, promotion, case_solved, case_failed
- Generate placeholder .ogg files if needed (short sine wave beeps via code)

### 2. Music System  
- Read PlayMusicEvent, crossfade to requested track
- Map-based music: precinct_theme, patrol_theme, downtown_ambient, tension_theme
- Emit PlayMusicEvent on MapTransitionEvent
- Loop by default

### 3. Ambient
- Clock ticking during shift (subtle)
- Radio static when dispatch idle

## You may edit any file under dlc/police/src/domains/
## Do NOT edit shared/mod.rs
