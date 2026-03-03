//! Detailed airport layout data — runway specs, frequencies, gates, taxiways, approaches.
//!
//! Each of the 10 airports has unique physical characteristics that affect
//! gameplay (e.g., short runways restrict heavy aircraft, high elevation
//! affects performance).

use crate::shared::*;
use std::collections::HashMap;

// ─── Radio Frequencies ───────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct AirportFrequencies {
    pub tower: f32,
    pub ground: f32,
    pub atis: f32,
    pub approach: f32,
}

// ─── Approach Procedure ──────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct ApproachProcedure {
    pub name: String,
    pub kind: ApproachKind,
    pub minimum_altitude_ft: u32,
    pub minimum_visibility_nm: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ApproachKind {
    Visual,
    IlsCatI,
    IlsCatII,
    Rnav,
    Vor,
    Ndb,
    Circling,
}

impl ApproachKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Visual => "Visual",
            Self::IlsCatI => "ILS CAT I",
            Self::IlsCatII => "ILS CAT II",
            Self::Rnav => "RNAV (GPS)",
            Self::Vor => "VOR",
            Self::Ndb => "NDB",
            Self::Circling => "Circling",
        }
    }
}

// ─── NOTAM ───────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct Notam {
    pub subject: String,
    pub text: String,
    pub severity: NotamSeverity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NotamSeverity {
    Info,
    Caution,
    Warning,
}

// ─── Taxiway Definition ──────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct TaxiwayDef {
    pub name: String,
    pub connects_to_runway: bool,
}

// ─── Airport Layout ──────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct AirportLayout {
    pub airport_id: AirportId,
    pub runway_length_ft: u32,
    pub runway_heading: u32,
    pub runway_designator: String,
    pub elevation_ft: u32,
    pub frequencies: AirportFrequencies,
    pub gate_count: u32,
    pub taxiways: Vec<TaxiwayDef>,
    pub approaches: Vec<ApproachProcedure>,
    pub notams: Vec<Notam>,
    pub facilities: &'static [&'static str],
    /// ASCII art hint stored as a doc comment block
    pub diagram_notes: &'static str,
}

// ─── Build all layouts ───────────────────────────────────────────────────

pub fn build_airport_layouts() -> HashMap<AirportId, AirportLayout> {
    let mut map = HashMap::new();

    // ── HomeBase (Clearfield Regional) ───────────────────────────────────
    //  Diagram:
    //   ┌──────────────────────────────────┐
    //   │  [G1] [G2]   Terminal            │
    //   │     ╲  ╱                          │
    //   │   ═══A═══  Taxiway Alpha         │
    //   │      │                            │
    //   │   ═══B═══  Taxiway Bravo         │
    //   │      │                            │
    //   │  ▓▓▓▓▓▓▓▓▓ Rwy 27 (5000ft) ▓▓▓▓ │
    //   └──────────────────────────────────┘
    map.insert(AirportId::HomeBase, AirportLayout {
        airport_id: AirportId::HomeBase,
        runway_length_ft: 5000,
        runway_heading: 270,
        runway_designator: "27/09".into(),
        elevation_ft: 800,
        frequencies: AirportFrequencies { tower: 118.7, ground: 121.9, atis: 127.25, approach: 119.1 },
        gate_count: 2,
        taxiways: vec![
            TaxiwayDef { name: "Alpha".into(), connects_to_runway: false },
            TaxiwayDef { name: "Bravo".into(), connects_to_runway: true },
        ],
        approaches: vec![
            ApproachProcedure { name: "Visual 27".into(), kind: ApproachKind::Visual, minimum_altitude_ft: 1600, minimum_visibility_nm: 3.0 },
            ApproachProcedure { name: "RNAV 27".into(), kind: ApproachKind::Rnav, minimum_altitude_ft: 1400, minimum_visibility_nm: 1.0 },
        ],
        notams: vec![
            Notam { subject: "Training".into(), text: "Student pilot training in progress — expect pattern traffic.".into(), severity: NotamSeverity::Info },
        ],
        facilities: &["Terminal", "Hangar", "Fuel", "Flight School"],
        diagram_notes: "Small regional airport. Two gates, single runway oriented E-W. Training-focused.",
    });

    // ── Grandcity International ──────────────────────────────────────────
    //  Diagram:
    //   ┌────────────────────────────────────────────┐
    //   │  [G1..G10]  Terminal A                     │
    //   │  [G11..G20] Terminal B                     │
    //   │     ╲  ╱                                    │
    //   │   ═══C═══  Charlie                         │
    //   │   ═══D═══  Delta                           │
    //   │   ═══E═══  Echo                            │
    //   │                                            │
    //   │  ▓▓▓▓▓▓▓▓▓▓▓▓▓ Rwy 09L/27R (12000ft)  ▓▓ │
    //   │  ▓▓▓▓▓▓▓▓▓▓▓▓▓ Rwy 09R/27L (11000ft)  ▓▓ │
    //   │   ═══F═══  Foxtrot                         │
    //   └────────────────────────────────────────────┘
    map.insert(AirportId::Grandcity, AirportLayout {
        airport_id: AirportId::Grandcity,
        runway_length_ft: 12000,
        runway_heading: 90,
        runway_designator: "09L/27R".into(),
        elevation_ft: 200,
        frequencies: AirportFrequencies { tower: 119.1, ground: 121.7, atis: 126.0, approach: 124.5 },
        gate_count: 20,
        taxiways: vec![
            TaxiwayDef { name: "Charlie".into(), connects_to_runway: false },
            TaxiwayDef { name: "Delta".into(), connects_to_runway: false },
            TaxiwayDef { name: "Echo".into(), connects_to_runway: true },
            TaxiwayDef { name: "Foxtrot".into(), connects_to_runway: true },
        ],
        approaches: vec![
            ApproachProcedure { name: "ILS 09L".into(), kind: ApproachKind::IlsCatII, minimum_altitude_ft: 400, minimum_visibility_nm: 0.25 },
            ApproachProcedure { name: "Visual 27R".into(), kind: ApproachKind::Visual, minimum_altitude_ft: 1000, minimum_visibility_nm: 3.0 },
            ApproachProcedure { name: "RNAV 09L".into(), kind: ApproachKind::Rnav, minimum_altitude_ft: 600, minimum_visibility_nm: 0.75 },
            ApproachProcedure { name: "VOR 27R".into(), kind: ApproachKind::Vor, minimum_altitude_ft: 800, minimum_visibility_nm: 1.5 },
        ],
        notams: vec![
            Notam { subject: "Noise".into(), text: "Noise abatement procedures in effect 22:00-06:00 local.".into(), severity: NotamSeverity::Info },
            Notam { subject: "Construction".into(), text: "Taxiway Bravo closed for repaving — use Charlie/Delta.".into(), severity: NotamSeverity::Caution },
        ],
        facilities: &["Terminal A", "Terminal B", "Cargo Terminal", "Hotel", "VIP Lounge", "Customs", "Car Rental", "Fuel Farm"],
        diagram_notes: "Major international hub. Dual parallel runways, 20 gates across two terminals.",
    });

    // ── Sunhaven Coastal ─────────────────────────────────────────────────
    map.insert(AirportId::Sunhaven, AirportLayout {
        airport_id: AirportId::Sunhaven,
        runway_length_ft: 8000,
        runway_heading: 180,
        runway_designator: "18/36".into(),
        elevation_ft: 50,
        frequencies: AirportFrequencies { tower: 118.3, ground: 121.6, atis: 127.8, approach: 120.5 },
        gate_count: 6,
        taxiways: vec![
            TaxiwayDef { name: "Alpha".into(), connects_to_runway: false },
            TaxiwayDef { name: "Bravo".into(), connects_to_runway: true },
            TaxiwayDef { name: "Charlie".into(), connects_to_runway: true },
        ],
        approaches: vec![
            ApproachProcedure { name: "Visual 18".into(), kind: ApproachKind::Visual, minimum_altitude_ft: 800, minimum_visibility_nm: 3.0 },
            ApproachProcedure { name: "ILS 18".into(), kind: ApproachKind::IlsCatI, minimum_altitude_ft: 300, minimum_visibility_nm: 0.5 },
            ApproachProcedure { name: "RNAV 36".into(), kind: ApproachKind::Rnav, minimum_altitude_ft: 500, minimum_visibility_nm: 1.0 },
        ],
        notams: vec![
            Notam { subject: "Seaplane".into(), text: "Water landing area west of Rwy 18 — seaplane ops permitted.".into(), severity: NotamSeverity::Info },
        ],
        facilities: &["Terminal", "Lounge", "Car Rental", "Customs", "Seaplane Dock", "Fuel"],
        diagram_notes: "Coastal resort airport. Runway aligned N-S along the beach. Seaplane operations.",
    });

    // ── Frostpeak Alpine ─────────────────────────────────────────────────
    map.insert(AirportId::Frostpeak, AirportLayout {
        airport_id: AirportId::Frostpeak,
        runway_length_ft: 6000,
        runway_heading: 45,
        runway_designator: "05/23".into(),
        elevation_ft: 7500,
        frequencies: AirportFrequencies { tower: 118.1, ground: 121.8, atis: 128.0, approach: 119.5 },
        gate_count: 3,
        taxiways: vec![
            TaxiwayDef { name: "Alpha".into(), connects_to_runway: true },
            TaxiwayDef { name: "Bravo".into(), connects_to_runway: false },
        ],
        approaches: vec![
            ApproachProcedure { name: "Visual 05".into(), kind: ApproachKind::Visual, minimum_altitude_ft: 9000, minimum_visibility_nm: 5.0 },
            ApproachProcedure { name: "NDB 05".into(), kind: ApproachKind::Ndb, minimum_altitude_ft: 8500, minimum_visibility_nm: 2.0 },
            ApproachProcedure { name: "Circling".into(), kind: ApproachKind::Circling, minimum_altitude_ft: 9200, minimum_visibility_nm: 3.0 },
        ],
        notams: vec![
            Notam { subject: "Icing".into(), text: "Moderate icing conditions reported in approach — de-icing available.".into(), severity: NotamSeverity::Warning },
            Notam { subject: "Terrain".into(), text: "Mountain terrain all quadrants — maintain obstacle clearance.".into(), severity: NotamSeverity::Caution },
        ],
        facilities: &["Terminal", "Hotel", "De-icing Pad", "Fuel"],
        diagram_notes: "Mountain airport at 7,500ft. Short runway in valley — challenging approach.",
    });

    // ── Ironforge Industrial ─────────────────────────────────────────────
    map.insert(AirportId::Ironforge, AirportLayout {
        airport_id: AirportId::Ironforge,
        runway_length_ft: 9000,
        runway_heading: 360,
        runway_designator: "36/18".into(),
        elevation_ft: 1200,
        frequencies: AirportFrequencies { tower: 118.5, ground: 121.75, atis: 127.5, approach: 120.1 },
        gate_count: 6,
        taxiways: vec![
            TaxiwayDef { name: "Alpha".into(), connects_to_runway: false },
            TaxiwayDef { name: "Bravo".into(), connects_to_runway: true },
            TaxiwayDef { name: "Charlie".into(), connects_to_runway: true },
        ],
        approaches: vec![
            ApproachProcedure { name: "ILS 36".into(), kind: ApproachKind::IlsCatI, minimum_altitude_ft: 1600, minimum_visibility_nm: 0.5 },
            ApproachProcedure { name: "Visual 18".into(), kind: ApproachKind::Visual, minimum_altitude_ft: 2000, minimum_visibility_nm: 3.0 },
        ],
        notams: vec![
            Notam { subject: "Heavy traffic".into(), text: "Frequent cargo operations — expect delays.".into(), severity: NotamSeverity::Info },
        ],
        facilities: &["Terminal", "Cargo Terminal", "Hotel", "Fuel Farm", "Maintenance Hangar"],
        diagram_notes: "Industrial hub. Heavy cargo ops with large apron. N-S runway.",
    });

    // ── Cloudmere Heights ────────────────────────────────────────────────
    map.insert(AirportId::Cloudmere, AirportLayout {
        airport_id: AirportId::Cloudmere,
        runway_length_ft: 7000,
        runway_heading: 315,
        runway_designator: "32/14".into(),
        elevation_ft: 6000,
        frequencies: AirportFrequencies { tower: 119.3, ground: 121.65, atis: 126.5, approach: 120.3 },
        gate_count: 5,
        taxiways: vec![
            TaxiwayDef { name: "Alpha".into(), connects_to_runway: true },
            TaxiwayDef { name: "Bravo".into(), connects_to_runway: false },
            TaxiwayDef { name: "Charlie".into(), connects_to_runway: false },
        ],
        approaches: vec![
            ApproachProcedure { name: "Visual 32".into(), kind: ApproachKind::Visual, minimum_altitude_ft: 7500, minimum_visibility_nm: 5.0 },
            ApproachProcedure { name: "RNAV 32".into(), kind: ApproachKind::Rnav, minimum_altitude_ft: 7000, minimum_visibility_nm: 1.5 },
        ],
        notams: vec![],
        facilities: &["Terminal", "Hotel", "VIP Lounge", "Fuel"],
        diagram_notes: "High-altitude airport above the cloud line. Scenic but demanding approaches.",
    });

    // ── Duskhollow Desert ────────────────────────────────────────────────
    map.insert(AirportId::Duskhollow, AirportLayout {
        airport_id: AirportId::Duskhollow,
        runway_length_ft: 8500,
        runway_heading: 90,
        runway_designator: "09/27".into(),
        elevation_ft: 2800,
        frequencies: AirportFrequencies { tower: 118.9, ground: 121.85, atis: 127.0, approach: 119.7 },
        gate_count: 5,
        taxiways: vec![
            TaxiwayDef { name: "Alpha".into(), connects_to_runway: true },
            TaxiwayDef { name: "Bravo".into(), connects_to_runway: false },
        ],
        approaches: vec![
            ApproachProcedure { name: "Visual 09".into(), kind: ApproachKind::Visual, minimum_altitude_ft: 3600, minimum_visibility_nm: 5.0 },
            ApproachProcedure { name: "VOR 27".into(), kind: ApproachKind::Vor, minimum_altitude_ft: 3400, minimum_visibility_nm: 1.5 },
        ],
        notams: vec![
            Notam { subject: "Dust".into(), text: "Blowing dust may reduce visibility — use caution.".into(), severity: NotamSeverity::Caution },
        ],
        facilities: &["Terminal", "Hotel", "Car Rental", "Fuel"],
        diagram_notes: "Desert oasis airport. Long runway on flat terrain. Dust hazards possible.",
    });

    // ── Stormwatch Research ──────────────────────────────────────────────
    map.insert(AirportId::Stormwatch, AirportLayout {
        airport_id: AirportId::Stormwatch,
        runway_length_ft: 7500,
        runway_heading: 225,
        runway_designator: "23/05".into(),
        elevation_ft: 500,
        frequencies: AirportFrequencies { tower: 118.0, ground: 121.55, atis: 126.8, approach: 121.0 },
        gate_count: 4,
        taxiways: vec![
            TaxiwayDef { name: "Alpha".into(), connects_to_runway: true },
            TaxiwayDef { name: "Bravo".into(), connects_to_runway: false },
            TaxiwayDef { name: "Delta".into(), connects_to_runway: true },
        ],
        approaches: vec![
            ApproachProcedure { name: "ILS 23".into(), kind: ApproachKind::IlsCatI, minimum_altitude_ft: 800, minimum_visibility_nm: 0.5 },
            ApproachProcedure { name: "Visual 05".into(), kind: ApproachKind::Visual, minimum_altitude_ft: 1200, minimum_visibility_nm: 3.0 },
            ApproachProcedure { name: "RNAV 23".into(), kind: ApproachKind::Rnav, minimum_altitude_ft: 700, minimum_visibility_nm: 0.75 },
        ],
        notams: vec![
            Notam { subject: "Weather".into(), text: "Weather research station on field — balloon launches possible.".into(), severity: NotamSeverity::Caution },
        ],
        facilities: &["Terminal", "Hotel", "Cargo Handling", "Weather Station", "Fuel"],
        diagram_notes: "Weather research station. Frequent severe weather — precision approaches essential.",
    });

    // ── Windport International ───────────────────────────────────────────
    map.insert(AirportId::Windport, AirportLayout {
        airport_id: AirportId::Windport,
        runway_length_ft: 8500,
        runway_heading: 135,
        runway_designator: "14/32".into(),
        elevation_ft: 300,
        frequencies: AirportFrequencies { tower: 118.4, ground: 121.7, atis: 127.3, approach: 119.9 },
        gate_count: 4,
        taxiways: vec![
            TaxiwayDef { name: "Alpha".into(), connects_to_runway: false },
            TaxiwayDef { name: "Bravo".into(), connects_to_runway: true },
        ],
        approaches: vec![
            ApproachProcedure { name: "Visual 14".into(), kind: ApproachKind::Visual, minimum_altitude_ft: 1100, minimum_visibility_nm: 3.0 },
            ApproachProcedure { name: "ILS 14".into(), kind: ApproachKind::IlsCatI, minimum_altitude_ft: 500, minimum_visibility_nm: 0.5 },
            ApproachProcedure { name: "RNAV 32".into(), kind: ApproachKind::Rnav, minimum_altitude_ft: 700, minimum_visibility_nm: 1.0 },
        ],
        notams: vec![
            Notam { subject: "Wind".into(), text: "Strong crosswinds common — check wind reports before approach.".into(), severity: NotamSeverity::Caution },
        ],
        facilities: &["Terminal", "Hotel", "Car Rental", "Customs", "Fuel"],
        diagram_notes: "Coastal city airport. Known for persistent crosswinds off the ocean.",
    });

    // ── Skyreach Elite ───────────────────────────────────────────────────
    map.insert(AirportId::Skyreach, AirportLayout {
        airport_id: AirportId::Skyreach,
        runway_length_ft: 10000,
        runway_heading: 270,
        runway_designator: "27/09".into(),
        elevation_ft: 9500,
        frequencies: AirportFrequencies { tower: 119.5, ground: 121.95, atis: 128.5, approach: 125.0 },
        gate_count: 8,
        taxiways: vec![
            TaxiwayDef { name: "Alpha".into(), connects_to_runway: false },
            TaxiwayDef { name: "Bravo".into(), connects_to_runway: true },
            TaxiwayDef { name: "Charlie".into(), connects_to_runway: false },
            TaxiwayDef { name: "Delta".into(), connects_to_runway: true },
        ],
        approaches: vec![
            ApproachProcedure { name: "ILS 27".into(), kind: ApproachKind::IlsCatI, minimum_altitude_ft: 10500, minimum_visibility_nm: 0.5 },
            ApproachProcedure { name: "Visual 27".into(), kind: ApproachKind::Visual, minimum_altitude_ft: 11000, minimum_visibility_nm: 5.0 },
            ApproachProcedure { name: "RNAV 09".into(), kind: ApproachKind::Rnav, minimum_altitude_ft: 10300, minimum_visibility_nm: 1.0 },
            ApproachProcedure { name: "Circling".into(), kind: ApproachKind::Circling, minimum_altitude_ft: 11500, minimum_visibility_nm: 3.0 },
        ],
        notams: vec![
            Notam { subject: "Performance".into(), text: "High density altitude — review performance charts before departure.".into(), severity: NotamSeverity::Warning },
            Notam { subject: "Approach".into(), text: "Mountain terrain on final — strict adherence to procedure required.".into(), severity: NotamSeverity::Warning },
        ],
        facilities: &["Terminal", "VIP Lounge", "Hotel", "Customs", "Cargo", "Fuel", "De-icing"],
        diagram_notes: "Elite endgame airport at 9,500ft. Long runway but thin air demands precision.",
    });

    map
}
