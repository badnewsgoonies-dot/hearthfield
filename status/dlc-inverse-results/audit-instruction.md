## Player Perspective Audit (required after implementation)

After implementing all features, audit from the player's perspective:

1. Walk through demo() line by line. Does every system get exercised?
2. Can a player talk to every NPC? Does every topic produce a response?
3. Does buy_drink actually change mood? Does the mood change affect subsequent conversations?
4. Does save/load restore ALL state — including affinity changes and mood changes made during play?
5. If any feature is implemented but not reachable from demo(), fix demo() to exercise it.
6. If any function exists but is never called, either call it or delete it.

Report: list every player-reachable feature and every dead/unreachable feature.
