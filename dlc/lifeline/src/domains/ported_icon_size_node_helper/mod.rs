//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



/// Standard 20×20 Node for screen title icons.
pub fn icon_size_node_helper() -> Node {
    Node {
        width: Val::Px(20.0),
        height: Val::Px(20.0),
        ..Default::default()
    }
}


