
use airmash_protocol::*;
use std::marker::PhantomData;

pub const FULL_CIRCLE: Rotation = Rotation{
    value_unsafe: 1.0,
    _marker: PhantomData
};

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
    }.into()
}

