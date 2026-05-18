use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Debug, Default)]
pub struct PatientLog { pub entries: Vec<String> }
impl PatientLog {
    pub fn try_remove(&mut self, _id: &str, dose: u8) -> u8 {
        let mut n: u8 = 0;
        while n < dose && self.entries.pop().is_some() { n += 1; }
        n
    }
}

#[derive(Debug, Clone, Default)]
pub struct TreatmentPlan { pub medications: Vec<(String, u8)> }


/// Consume all non-wildcard medications.
pub fn dispense_treatment(inventory: &mut PatientLog, recipe: &TreatmentPlan) {
    for (med_id, dose) in &recipe.medications {
        if med_id == "placebo" {
            continue;
        }
        let dispensed = inventory.try_remove(med_id, *dose);
        if dispensed < *dose {
            warn!(
                "dispense_treatment: only dispensed {} of {} '{}'",
                dispensed, dose, med_id
            );
        }
    }
}


