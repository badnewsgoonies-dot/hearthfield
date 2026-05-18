//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



/// Format a gold amount as a display string (e.g. "1,234g").
#[allow(dead_code)]
pub fn format_gold_helper(amount: u32) -> String {
    let s = amount.to_string();
    let mut result = String::new();
    let digits: Vec<char> = s.chars().collect();
    for (i, ch) in digits.iter().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(*ch);
    }
    result.push('g');
    result
}


