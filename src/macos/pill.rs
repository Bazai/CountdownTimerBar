#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PillGeometry {
    pub width: f64,
    pub height: f64,
    pub corner_radius: f64,
}

/// Pads the text by 9 pt x 3 pt, rounded up to whole points; the corner radius is
/// half the height (a full capsule).
pub fn pill_geometry(text_width: f64, text_height: f64) -> PillGeometry {
    let height = (text_height + 3.0).ceil();
    let width = (text_width + 9.0).ceil();
    PillGeometry {
        width,
        height,
        corner_radius: height / 2.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pill_geometry_matches_swift_formula() {
        let geometry = pill_geometry(40.0, 16.0);
        assert!((geometry.height - (16.0f64 + 3.0).ceil()).abs() < 1e-9);
        assert!((geometry.width - (40.0f64 + 9.0).ceil()).abs() < 1e-9);
        assert!((geometry.corner_radius - geometry.height / 2.0).abs() < 1e-9);
    }
}
