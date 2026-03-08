# Police Event Connectivity Audit

Scope: `dlc/police/src/main.rs` `add_event::<...>()` registrations cross-checked against `EventWriter<T>` and `EventReader<T>` usage under `dlc/police/src/domains/`.

| Event | Writers | Readers | Status |
| --- | --- | --- | --- |
| ShiftEndEvent | `dlc/police/src/domains/calendar/mod.rs:51` | `dlc/police/src/domains/cases/mod.rs:232`<br>`dlc/police/src/domains/cases/mod.rs:285`<br>`dlc/police/src/domains/economy/mod.rs:79`<br>`dlc/police/src/domains/evidence/mod.rs:79`<br>`dlc/police/src/domains/save/mod.rs:168` | OK |
| CaseAssignedEvent | `dlc/police/src/domains/precinct/mod.rs:137` | `dlc/police/src/domains/cases/mod.rs:70` | OK |
| CaseSolvedEvent | `dlc/police/src/domains/cases/mod.rs:205` | `dlc/police/src/domains/economy/mod.rs:92`<br>`dlc/police/src/domains/skills/mod.rs:212`<br>`dlc/police/src/domains/ui/notifications.rs:201` | OK |
| CaseFailedEvent | `dlc/police/src/domains/cases/mod.rs:248` | `dlc/police/src/domains/economy/mod.rs:109`<br>`dlc/police/src/domains/ui/notifications.rs:217` | OK |
| EvidenceCollectedEvent | `dlc/police/src/domains/cases/mod.rs:109`<br>`dlc/police/src/domains/npcs/mod.rs:593`<br>`dlc/police/src/domains/precinct/mod.rs:138` | `dlc/police/src/domains/cases/mod.rs:142`<br>`dlc/police/src/domains/evidence/mod.rs:50`<br>`dlc/police/src/domains/skills/mod.rs:224`<br>`dlc/police/src/domains/ui/notifications.rs:229` | OK |
| EvidenceProcessedEvent | `dlc/police/src/domains/evidence/mod.rs:81` | `dlc/police/src/domains/ui/notifications.rs:180` | OK |
| InterrogationStartEvent | `dlc/police/src/domains/npcs/mod.rs:550` | `dlc/police/src/domains/npcs/mod.rs:589`<br>`dlc/police/src/domains/ui/screens.rs:323` | OK |
| InterrogationEndEvent | `dlc/police/src/domains/ui/screens.rs:989` | `dlc/police/src/domains/npcs/mod.rs:590`<br>`dlc/police/src/domains/ui/screens.rs:324` | OK |
| DispatchCallEvent | `dlc/police/src/domains/patrol/mod.rs:125` | `dlc/police/src/domains/ui/notifications.rs:288` | OK |
| DispatchResolvedEvent | `-` | `dlc/police/src/domains/skills/mod.rs:236` | DEAD_READER |
| PromotionEvent | `dlc/police/src/domains/economy/mod.rs:157` | `dlc/police/src/domains/cases/mod.rs:286`<br>`dlc/police/src/domains/economy/mod.rs:181`<br>`dlc/police/src/domains/ui/notifications.rs:260` | OK |
| NpcTrustChangeEvent | `dlc/police/src/domains/skills/mod.rs:298`<br>`dlc/police/src/domains/ui/screens.rs:988` | `dlc/police/src/domains/npcs/mod.rs:656` | OK |
| DialogueStartEvent | `dlc/police/src/domains/npcs/mod.rs:549` | `dlc/police/src/domains/npcs/mod.rs:564`<br>`dlc/police/src/domains/ui/screens.rs:307` | OK |
| DialogueEndEvent | `dlc/police/src/domains/npcs/mod.rs:556`<br>`dlc/police/src/domains/ui/screens.rs:387` | `dlc/police/src/domains/npcs/mod.rs:565`<br>`dlc/police/src/domains/ui/screens.rs:308` | OK |
| MapTransitionEvent | `dlc/police/src/domains/player/mod.rs:220` | `dlc/police/src/domains/npcs/mod.rs:440`<br>`dlc/police/src/domains/npcs/mod.rs:473`<br>`dlc/police/src/domains/patrol/mod.rs:175`<br>`dlc/police/src/domains/world/mod.rs:303` | OK |
| FatigueChangeEvent | `dlc/police/src/domains/precinct/mod.rs:139` | `dlc/police/src/domains/player/mod.rs:241` | OK |
| StressChangeEvent | `dlc/police/src/domains/precinct/mod.rs:140` | `dlc/police/src/domains/player/mod.rs:242` | OK |
| GoldChangeEvent | `dlc/police/src/domains/economy/mod.rs:81`<br>`dlc/police/src/domains/economy/mod.rs:93`<br>`dlc/police/src/domains/economy/mod.rs:110`<br>`dlc/police/src/domains/economy/mod.rs:195` | `dlc/police/src/domains/economy/mod.rs:126` | OK |
| XpGainedEvent | `dlc/police/src/domains/npcs/mod.rs:594`<br>`dlc/police/src/domains/skills/mod.rs:213`<br>`dlc/police/src/domains/skills/mod.rs:225` | `dlc/police/src/domains/skills/mod.rs:237`<br>`dlc/police/src/domains/ui/notifications.rs:244` | OK |
| SkillPointSpentEvent | `dlc/police/src/domains/ui/screens.rs:636` | `dlc/police/src/domains/skills/mod.rs:268`<br>`dlc/police/src/domains/ui/notifications.rs:272` | OK |
| PlaySfxEvent | `dlc/police/src/domains/ui/mod.rs:359`<br>`dlc/police/src/domains/ui/mod.rs:787` | `dlc/police/src/domains/ui/notifications.rs:416` | OK |
| PlayMusicEvent | `dlc/police/src/domains/ui/mod.rs:140`<br>`dlc/police/src/domains/ui/mod.rs:147` | `dlc/police/src/domains/ui/notifications.rs:417` | OK |
| ToastEvent | `dlc/police/src/domains/precinct/mod.rs:141`<br>`dlc/police/src/domains/ui/notifications.rs:182`<br>`dlc/police/src/domains/ui/notifications.rs:202`<br>`dlc/police/src/domains/ui/notifications.rs:218`<br>`dlc/police/src/domains/ui/notifications.rs:230`<br>`dlc/police/src/domains/ui/notifications.rs:245`<br>`dlc/police/src/domains/ui/notifications.rs:261`<br>`dlc/police/src/domains/ui/notifications.rs:273`<br>`dlc/police/src/domains/ui/screens.rs:990` | `dlc/police/src/domains/ui/notifications.rs:304` | OK |
| SaveRequestEvent | `dlc/police/src/domains/save/mod.rs:169`<br>`dlc/police/src/domains/ui/mod.rs:785` | `dlc/police/src/domains/save/mod.rs:79` | OK |
| LoadRequestEvent | `dlc/police/src/domains/ui/mod.rs:358`<br>`dlc/police/src/domains/ui/mod.rs:786` | `dlc/police/src/domains/save/mod.rs:121` | OK |

Note: status is based only on `EventWriter<T>` and `EventReader<T>` grep hits, per request. `DispatchResolvedEvent` is flagged `DEAD_READER` under that rule, but there is also a direct `Events<DispatchResolvedEvent>` send path in `dlc/police/src/domains/patrol/mod.rs:206` that is not counted here.
