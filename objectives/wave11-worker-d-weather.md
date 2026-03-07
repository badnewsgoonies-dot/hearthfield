# Worker D: Weather + Season Notifications

## Scope: src/world/ AND src/calendar/ ONLY. All other edits will be reverted.

## Read first (DO NOT MODIFY):
- src/shared/mod.rs — find ToastEvent, PlaySfxEvent, WeatherKind (or Weather enum)
- src/world/weather_fx.rs — weather particle/visual system
- src/calendar/mod.rs — season change handling

## Task 1: Weather change notification (world/weather_fx.rs)
Find where weather state transitions (rain starts, rain stops, etc).
When weather changes, send a toast:
- Rain starts: "It started raining."
- Rain stops: "The rain stopped."
- Other weather changes: appropriate message

Use a Local<Option<WeatherKind>> (or whatever the weather type is) to track previous weather
and only fire the toast on actual transitions, not every frame.

```rust
pub fn weather_change_notification(
    weather: Res<Weather>,  // or whatever the weather resource is
    mut toast_events: EventWriter<ToastEvent>,
    mut prev_weather: Local<Option<WeatherKind>>,
) {
    let current = weather.kind; // adjust field name
    if Some(current) != *prev_weather {
        if let Some(_prev) = *prev_weather {
            let msg = match current {
                WeatherKind::Rain => "It started raining.",
                WeatherKind::Clear | WeatherKind::Sunny => "The rain stopped.",
                _ => "",
            };
            if !msg.is_empty() {
                toast_events.send(ToastEvent {
                    message: msg.into(),
                    duration_secs: 3.0,
                });
            }
        }
        *prev_weather = Some(current);
    }
}
```

Register this system in the appropriate plugin. READ the existing code to find the right
weather types and resource names — don't guess.

## Task 2: Season arrival (calendar/mod.rs)
Check if season change already sends a toast or shows text. The sleep cutscene in trigger_sleep
already shows "Spring has arrived!" etc. If season can change WITHOUT going through trigger_sleep
(e.g. loading a save at season boundary), add a toast. If trigger_sleep always handles it, skip this.

## Validation: cargo check must pass.
## When done: write report to status/workers/feedback-weather-season.md
