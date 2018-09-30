use protocol::server::*;
use protocol::*;

use std::time::Instant;

use fnv::FnvHashMap;

pub struct PlayerData {
    pub pos: Position,
    pub rot: Rotation,
    pub vel: Velocity,

    pub health: Health,
    pub energy: Energy,
    pub health_regen: HealthRegen,
    pub energy_regen: EnergyRegen,

    pub name: String,
    pub flag: FlagCode,
    pub upgrades: Upgrades,
    pub score: Score,
    pub level: Level,
    pub team: Team,
    /// Whether the player is alive or dead
    pub status: PlayerStatus,
    pub plane: PlaneType,

    pub votemuted: bool,
}

#[derive(Default)]
pub struct MyPlayerData {
    pub id: Player,
    pub token: String,
}

pub struct MobData {}

pub struct MissileData {}

pub struct GameState {
    pub players: FnvHashMap<Player, PlayerData>,
    pub mobs: FnvHashMap<Mob, MobData>,
    pub me: MyPlayerData,

    pub game_ty: GameType,
    pub clock: u32,
    pub room: String,
}

impl GameState {
    fn handle_login(&mut self, packet: &Login) {
        self.players.clear();

        self.players.extend(packet.players.iter().map(|player| {
            (
                player.id,
                PlayerData {
                    pos: player.pos,
                    rot: player.rot,
                    vel: Default::default(),

                    health: Health::new(1.0),
                    energy: Energy::new(1.0),
                    health_regen: Default::default(),
                    energy_regen: Default::default(),

                    name: player.name.clone(),
                    status: player.status,
                    plane: player.ty,
                    flag: player.flag,
                    score: Default::default(),
                    level: player.level,
                    team: player.team,
                    upgrades: player.upgrades,

                    votemuted: false,
                },
            )
        }));

        self.me = MyPlayerData {
            id: packet.id,
            token: packet.token.clone(),
        };

        self.room = packet.room.clone();
        self.clock = packet.clock;
        self.game_ty = packet.ty;
    }

    /// Handle a packet from the server
    pub(crate) fn update_state(&mut self, packet: &ServerPacket) {
        use self::ServerPacket::*;

        match packet {
            Login(p) => self.handle_login(p),
            _ => (),
        }
    }

    /// Update game state given that a frame has passed
    pub(crate) fn update_frame(&mut self, now: Instant) {}
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            players: Default::default(),
            mobs: Default::default(),
            me: Default::default(),

            game_ty: GameType::FFA,

            clock: Default::default(),
            room: Default::default(),
        }
    }
}
