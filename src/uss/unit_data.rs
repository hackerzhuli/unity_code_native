use std::collections::HashMap;

use crate::uss::definitions::UnitInfo;


/// Create unit information with documentation
pub fn create_unit_info() -> HashMap<&'static str, UnitInfo> {
    let mut units = HashMap::new();

    // Length units
    units.insert("px", UnitInfo {
        name: "px",
        category: "Length",
        description: "Absolute length unit representing device pixels.",
        details: None,
    });

    units.insert("%", UnitInfo {
        name: "%",
        category: "Length",
        description: "Relative unit based on the parent element's corresponding property.",
        details: None,
    });

    // Angle units
    units.insert("deg", UnitInfo {
        name: "deg",
        category: "Angle",
        description: "Angle unit where 360deg = full rotation.",
        details: None,
    });

    units.insert("rad", UnitInfo {
        name: "rad",
        category: "Angle",
        description: "Angle unit where 2π rad = full rotation.",
        details: None,
    });

    units.insert("grad", UnitInfo {
        name: "grad",
        category: "Angle",
        description: "Angle unit where 400grad = full rotation.",
        details: None,
    });

    units.insert("turn", UnitInfo {
        name: "turn",
        category: "Angle",
        description: "Angle unit where 1turn = full rotation.",
        details: None,
    });

    // Time units
    units.insert("s", UnitInfo {
        name: "s",
        category: "Time",
        description: "Time unit for durations and delays.",
        details: None,
    });

    units.insert("ms", UnitInfo {
        name: "ms",
        category: "Time",
        description: "Time unit for durations and delays.",
        details: Some("1s = 1000ms"),
    });

    units
}
