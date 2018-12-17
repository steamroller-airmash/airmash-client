use airmash_protocol::*;

pub const BASE_DIR: Vector2<f32> = Vector2 { x: 0.0, y: -1.0 };

/// Basically returns whatever value is in the
/// config for the provided plane type.
///
/// This might become obsolete in the future
/// if custom configs become more full-fledged.
pub fn rotation_rate(ty: PlaneType) -> RotationRate {
    use self::PlaneType::*;

    match ty {
        Predator => 0.065,
        Tornado => 0.055,
        Prowler => 0.055,
        Mohawk => 0.07,
        Goliath => 0.04,
    }
    .into()
}
