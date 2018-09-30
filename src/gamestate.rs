use protocol::server::*;
use protocol::*;

use std::time::Instant;

use fnv::FnvHashMap;

#[derive(Debug, Clone)]
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
    pub keystate: ServerKeyState,

    pub votemuted: bool,
}

#[derive(Debug, Clone, Default)]
pub struct MyPlayerData {
    pub id: Player,
    pub token: String,
}

#[derive(Debug, Clone)]
pub struct MobData {}

#[derive(Default, Debug, Clone)]
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
                    keystate: Default::default(),

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

    fn handle_player_update(&mut self, packet: &PlayerUpdate) {
        self.clock = packet.clock;

        if let Some(player) = self.players.get_mut(&packet.id) {
            player.keystate = packet.keystate;
            player.upgrades = packet.upgrades;
            player.pos = packet.pos;
            player.rot = packet.rot;
            player.vel = packet.speed;
        }
        else {
            info!("Got update for nonexistent player {}", packet.id.0);
        }
    }

    fn handle_chat_vote_muted(&mut self) {
        self.players
            .get_mut(&self.me.id)
            .expect("The current player doesn't exist!")
            .votemuted = true;
    }
    fn handle_chat_vote_mute_passed(&mut self, packet: &ChatVoteMutePassed) {
        self.players
            .get_mut(&packet.id)
            .map(|x| x.votemuted = false);
    }

    /// Handle a packet from the server
    pub(crate) fn update_state(&mut self, packet: &ServerPacket) {
        use self::ServerPacket::*;

        match packet {
            Login(p) => self.handle_login(p),
            PlayerUpdate(p) => self.handle_player_update(p),
            ChatVoteMuted => self.handle_chat_vote_muted(),
            ChatVoteMutePassed(p) => self.handle_chat_vote_mute_passed(p),
            _ => (),
        }
    }

    /// Update game state given that a frame has passed
    pub(crate) fn update_frame(&mut self, now: Instant) {}
}
