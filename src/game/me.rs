use std::time::Instant;

use bstr::BString;

#[derive(Debug, Default, Copy, Clone)]
pub struct ClientUpgrades {
    pub speed: u8,
    pub defense: u8,
    pub energy: u8,
    pub missile: u8,
    pub unused: u16,
}

#[derive(Debug, Clone)]
pub struct CurrentPlayer {
    pub id: u16,
    pub upgrades: ClientUpgrades,
    pub powerup_expiry: Option<Instant>,

    pub token: BString,
}

impl Default for CurrentPlayer {
    fn default() -> Self {
        Self {
            id: 0,
            upgrades: ClientUpgrades::default(),
            powerup_expiry: None,
            token: String::default().into()
        }
    }
}
