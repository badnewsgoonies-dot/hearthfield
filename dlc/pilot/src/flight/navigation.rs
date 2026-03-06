//! Navigation system — waypoint tracking, course deviation, ETA, heading hints.

use crate::shared::*;
use bevy::prelude::*;

/// A point along the flight route.
#[derive(Clone, Debug)]
pub struct Waypoint {
    pub name: String,
    pub distance_from_origin_nm: f32,
    pub heading_deg: f32,
}

/// Persistent navigation state for the active flight.
#[derive(Resource, Default)]
pub struct NavigationState {
    pub route: Vec<Waypoint>,
    pub current_waypoint_index: usize,
    pub deviation_deg: f32,
    pub eta_secs: f32,
    pub on_course: bool,
    pub course_correction_hint: Option<String>,
    pub distance_to_next_wp: f32,
    pub total_route_distance: f32,
}

/// Calculate intermediate waypoints for a route between two airports.
pub fn calculate_route(from: AirportId, to: AirportId) -> Vec<Waypoint> {
    let total = airport_distance(from, to);
    let bearing = route_bearing(from, to);
    let mut waypoints = Vec::new();

    waypoints.push(Waypoint {
        name: format!("{} DEP", from.icao_code()),
        distance_from_origin_nm: 0.0,
        heading_deg: bearing,
    });

    if total > 100.0 {
        let segment_count = (total / 80.0).ceil() as usize;
        let segment_len = total / segment_count as f32;
        for i in 1..segment_count {
            let dist = segment_len * i as f32;
            let jitter = ((i as f32) * 7.3).sin() * 5.0;
            waypoints.push(Waypoint {
                name: format!("WP{}", i),
                distance_from_origin_nm: dist,
                heading_deg: normalize_heading(bearing + jitter),
            });
        }
    }

    waypoints.push(Waypoint {
        name: format!("{} ARR", to.icao_code()),
        distance_from_origin_nm: total,
        heading_deg: bearing,
    });

    waypoints
}

fn route_bearing(from: AirportId, to: AirportId) -> f32 {
    let idx_from = airport_index(from);
    let idx_to = airport_index(to);
    let dx = (idx_to % 4) as f32 - (idx_from % 4) as f32;
    let dy = (idx_to / 4) as f32 - (idx_from / 4) as f32;
    let angle = dy.atan2(dx).to_degrees();
    normalize_heading(90.0 - angle)
}

fn airport_index(id: AirportId) -> i32 {
    match id {
        AirportId::HomeBase => 0,
        AirportId::Windport => 1,
        AirportId::Frostpeak => 2,
        AirportId::Sunhaven => 3,
        AirportId::Ironforge => 4,
        AirportId::Cloudmere => 5,
        AirportId::Duskhollow => 6,
        AirportId::Stormwatch => 7,
        AirportId::Grandcity => 8,
        AirportId::Skyreach => 9,
    }
}

fn normalize_heading(h: f32) -> f32 {
    let mut r = h % 360.0;
    if r < 0.0 {
        r += 360.0;
    }
    r
}

fn heading_delta(current: f32, target: f32) -> f32 {
    let mut d = target - current;
    while d > 180.0 {
        d -= 360.0;
    }
    while d < -180.0 {
        d += 360.0;
    }
    d
}

fn calculate_eta(distance_nm: f32, speed_knots: f32) -> f32 {
    if speed_knots < 1.0 {
        return f32::MAX;
    }
    (distance_nm / speed_knots) * 3600.0
}

pub fn format_eta(eta_secs: f32) -> String {
    if eta_secs >= f32::MAX * 0.5 {
        return "N/A".to_string();
    }
    let mins = (eta_secs / 60.0) as u32;
    if mins >= 60 {
        format!("{}h {:02}m", mins / 60, mins % 60)
    } else {
        format!("{}m", mins)
    }
}

/// Main navigation update system.
pub fn update_navigation(
    flight_state: Res<FlightState>,
    mut nav: ResMut<NavigationState>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    if !matches!(
        flight_state.phase,
        FlightPhase::Climb | FlightPhase::Cruise | FlightPhase::Descent | FlightPhase::Approach
    ) {
        return;
    }

    if nav.route.is_empty() {
        return;
    }

    let distance_flown = nav.total_route_distance - flight_state.distance_remaining_nm;

    while nav.current_waypoint_index + 1 < nav.route.len() {
        let next = &nav.route[nav.current_waypoint_index + 1];
        if distance_flown >= next.distance_from_origin_nm {
            nav.current_waypoint_index += 1;
            let wp_name = nav.route[nav.current_waypoint_index].name.clone();
            toast_events.send(ToastEvent {
                message: format!("📍 Passing waypoint {}", wp_name),
                duration_secs: 2.5,
            });
        } else {
            break;
        }
    }

    if nav.current_waypoint_index + 1 < nav.route.len() {
        // Copy values out to avoid borrow conflict
        let wp_dist = nav.route[nav.current_waypoint_index + 1].distance_from_origin_nm;
        let wp_heading = nav.route[nav.current_waypoint_index + 1].heading_deg;
        let wp_name = nav.route[nav.current_waypoint_index + 1].name.clone();

        nav.distance_to_next_wp = (wp_dist - distance_flown).max(0.0);

        let delta = heading_delta(flight_state.heading_deg, wp_heading);
        nav.deviation_deg = delta;
        nav.on_course = delta.abs() < 10.0;

        if delta.abs() > 10.0 {
            let direction = if delta > 0.0 { "right" } else { "left" };
            let correction = delta.abs().round() as i32;
            nav.course_correction_hint = Some(format!(
                "Turn {} {}° toward {}",
                direction, correction, wp_name
            ));
        } else {
            nav.course_correction_hint = None;
        }
    } else {
        nav.distance_to_next_wp = flight_state.distance_remaining_nm;
        nav.on_course = true;
        nav.course_correction_hint = None;
    }

    nav.eta_secs = calculate_eta(flight_state.distance_remaining_nm, flight_state.speed_knots);

    if nav.deviation_deg.abs() > 45.0 {
        toast_events.send(ToastEvent {
            message: "⚠ Significant course deviation!".to_string(),
            duration_secs: 3.0,
        });
    }
}

/// Reset navigation state when a flight starts.
pub fn reset_navigation(
    mut events: EventReader<FlightStartEvent>,
    mut nav: ResMut<NavigationState>,
) {
    for ev in events.read() {
        let route = calculate_route(ev.origin, ev.destination);
        let total = airport_distance(ev.origin, ev.destination);
        *nav = NavigationState {
            route,
            current_waypoint_index: 0,
            deviation_deg: 0.0,
            eta_secs: 0.0,
            on_course: true,
            course_correction_hint: None,
            distance_to_next_wp: 0.0,
            total_route_distance: total,
        };
    }
}
