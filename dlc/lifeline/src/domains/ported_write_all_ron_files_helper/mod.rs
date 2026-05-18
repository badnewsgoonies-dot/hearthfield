//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



/// Write all exported maps to `assets/maps/` as RON files.
#[cfg(test)]
pub fn write_all_ron_files_helper() -> std::io::Result<()> {
    std::fs::create_dir_all("assets/maps")?;

    for (map_id, data) in export_all_maps() {
        let name = map_id_filename(map_id);
        let path = format!("assets/maps/{}.ron", name);
        let config = ron::ser::PrettyConfig::new()
            .depth_limit(4)
            .separate_tuple_members(false)
            .enumerate_arrays(false);
        let ron_str =
            ron::ser::to_string_pretty(&data, config).expect("Failed to serialize map data");
        std::fs::write(&path, ron_str)?;
    }

    Ok(())
}


