//! Single-fn substrate port — pure/near-pure.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;
use std::collections::HashMap;



/// Format a cash amount as a display string (e.g. "1,234g").
#[allow(dead_code)]
pub fn format_billing(amount: u32) -> String {
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


