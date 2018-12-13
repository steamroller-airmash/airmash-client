use super::*;
use super::{Mob, Player};
use crate::protocol::server::*;
use crate::protocol::*;

use std::time::{Duration, Instant};

use hashbrown::HashMap;

#[derive(Default, Debug, Clone)]
pub struct World {
    pub me: CurrentPlayer,
    pub mobs: HashMap<u16, Mob>,
    pub players: HashMap<u16, Player>,

    pub game_ty: GameType,
    pub room: String,
    pub clock: u32,
    pub key_seq: u32,
}

macro_rules! warn_unknown {
    ($class:expr, $ty:ident, $id:expr) => {
        warn!(
            "Received {} for unknown {} with id {}",
            stringify!($ty),
            $class,
            $id.0
        );
    };
}

macro_rules! warn_unknown_player {
    ($ty:ident, $id:expr) => {
        warn_unknown!("player", $ty, $id);
    };
}

macro_rules! warn_unknown_mob {
    ($ty:ident, $id:expr) => {
        warn_unknown!("mob", $ty, $id);
    };
}

impl World {
    pub fn get_me<'a>(&'a self) -> &'a Player {
        &self.players[&self.me.id]
    }

    pub fn handle_packet(&mut self, packet: &ServerPacket) {
        use self::ServerPacket::*;

        match packet {
            Login(p) => self.handle_login(p),
            ScoreBoard(p) => self.handle_score_board(p),
            ScoreUpdate(p) => self.handle_score_update(p),

            PlayerNew(p) => self.handle_player_new(p),
            PlayerUpdate(p) => self.handle_player_update(p),
            PlayerLeave(p) => self.handle_player_leave(p),
            PlayerHit(p) => self.handle_player_hit(p),
            PlayerKill(p) => self.handle_player_kill(p),
            PlayerLevel(p) => self.handle_player_level(p),
            PlayerPowerup(p) => self.handle_player_powerup(p),
            PlayerRespawn(p) => self.handle_player_respawn(p),
            PlayerReteam(p) => self.handle_player_reteam(p),
            PlayerType(p) => self.handle_player_type(p),
            PlayerUpgrade(p) => self.handle_player_upgrade(p),
            PlayerFire(p) => self.handle_player_fire(p),
            PlayerFlag(p) => self.handle_player_flag(p),

            EventBoost(p) => self.handle_event_boost(p),
            EventBounce(p) => self.handle_event_bounce(p),
            EventLeaveHorizon(p) => self.handle_event_leave_horizon(p),
            EventRepel(p) => self.handle_event_repel(p),
            EventStealth(p) => self.handle_event_stealth(p),
            _ => (),
        }
    }

    pub fn update(&mut self, _now: Instant) {}
}

// Packet handling details
impl World {
    fn handle_player_update(&mut self, update: &PlayerUpdate) {
        if let Some(player) = self.players.get_mut(&update.id.into()) {
            player.update(update);
        } else {
            warn_unknown_player!(PlayerUpdate, update.id);
        }
    }
    fn handle_player_new(&mut self, packet: &PlayerNew) {
        let new = Player {
            id: packet.id.into(),
            name: packet.name.clone(),
            status: packet.status,
            plane: packet.ty,
            team: packet.team,
            flag: packet.flag,
            upgrades: packet.upgrades,

            pos: packet.pos,
            rot: packet.rot,
            ..Default::default()
        };

        if let Some(_old) = self.players.insert(packet.id.into(), new) {
            warn_unknown_player!(PlayerNew, packet.id);
        }
    }
    fn handle_player_leave(&mut self, packet: &PlayerLeave) {
        let removed = self.players.remove(&packet.id.into());

        if removed.is_none() {
            warn_unknown_player!(PlayerLeave, packet.id);
        }
    }
    fn handle_player_hit(&mut self, packet: &PlayerHit) {
        if let None = self.mobs.remove(&packet.id.into()) {
            warn_unknown_mob!(PlayerHit, packet.id);
        }

        for data in packet.players.iter() {
            if let Some(player) = self.players.get_mut(&data.id.into()) {
                player.health = data.health;
                player.health_regen = data.health_regen;
            } else {
                warn_unknown_mob!(PlayerHit, data.id);
            }
        }
    }
    fn handle_player_kill(&mut self, packet: &PlayerKill) {
        if let Some(player) = self.players.get_mut(&packet.id.into()) {
            player.status = PlayerStatus::Dead;
            player.pos = packet.pos;
        } else {
            warn_unknown_player!(PlayerKill, packet.id);
        }
    }
    fn handle_player_level(&mut self, packet: &PlayerLevel) {
        if let Some(player) = self.players.get_mut(&packet.id.into()) {
            player.level = Some(packet.level.into());
        } else {
            warn_unknown_player!(PlayerLevel, packet.id);
        }
    }
    fn handle_player_powerup(&mut self, packet: &PlayerPowerup) {
        self.me.powerup_expiry =
            Some(Instant::now() + Duration::from_millis(packet.duration.into()));

        if let Some(_player) = self.players.get_mut(&self.me.id) {
            // FIXME: This should probably set some state
        } else {
            error!("The current player doesn't exist (id: {})", self.me.id);
        }
    }
    fn handle_player_respawn(&mut self, packet: &PlayerRespawn) {
        if let Some(player) = self.players.get_mut(&packet.id.into()) {
            player.pos = packet.pos;
            player.rot = packet.rot;
            player.upgrades = packet.upgrades;
        } else {
            warn_unknown_player!(PlayerRespawn, packet.id);
        }
    }
    fn handle_player_reteam(&mut self, packet: &PlayerReteam) {
        for data in packet.players.iter() {
            if let Some(player) = self.players.get_mut(&data.id.into()) {
                player.team = data.team;
            } else {
                warn_unknown_player!(PlayerReteam, data.id);
            }
        }
    }
    fn handle_player_type(&mut self, packet: &PlayerType) {
        if let Some(player) = self.players.get_mut(&packet.id.into()) {
            player.plane = packet.ty;
        } else {
            warn_unknown_player!(PlayerType, packet.id);
        }
    }
    fn handle_player_upgrade(&mut self, packet: &PlayerUpgrade) {
        self.me.upgrades = ClientUpgrades {
            unused: packet.upgrades,
            speed: packet.speed,
            defense: packet.defense,
            energy: packet.energy,
            missile: packet.missile,
        };
    }
    fn handle_player_fire(&mut self, packet: &PlayerFire) {
        self.clock = packet.clock;

        if let Some(player) = self.players.get_mut(&packet.id.into()) {
            player.energy = packet.energy;
            player.energy_regen = packet.energy_regen;
        } else {
            warn_unknown_player!(PlayerFire, packet.id);
        }

        for projectile in packet.projectiles.iter() {
            let mob = Mob {
                id: projectile.id.into(),
                ty: projectile.ty,
                pos: projectile.pos,
                vel: projectile.speed,
                accel: projectile.accel,
                max_speed: projectile.max_speed,
                owner: packet.id.into(),
            };

            if let Some(mob) = self.mobs.insert(mob.id, mob) {
                warn!(
                    "Got PlayerFire projectile created with id {} that was already in use.",
                    mob.id
                );
            }
        }
    }
    fn handle_player_flag(&mut self, packet: &PlayerFlag) {
        if let Some(player) = self.players.get_mut(&packet.id.into()) {
            player.flag = packet.flag;
        } else {
            warn_unknown_player!(PlayerFlag, packet.id);
        }
    }

    fn handle_login(&mut self, packet: &Login) {
        self.me = CurrentPlayer {
            id: packet.id.into(),
            token: packet.token.clone(),
            ..Default::default()
        };
        self.game_ty = packet.ty;
        self.room = packet.room.clone();

        self.players = packet
            .players
            .iter()
            .map(|player| {
                let level = match player.level.into() {
                    0 => None,
                    x => Some(x),
                };

                let details = Player {
                    level,
                    id: player.id.into(),
                    status: player.status.into(),
                    name: player.name.clone(),
                    plane: player.ty,
                    team: player.team,
                    pos: player.pos,
                    rot: player.rot,
                    flag: player.flag,
                    upgrades: player.upgrades,
                    ..Default::default()
                };

                (details.id, details)
            })
            .collect();
    }
    fn handle_score_board(&mut self, packet: &ScoreBoard) {
        for (i, data) in packet.rankings.iter().enumerate() {
            if let Some(player) = self.players.get_mut(&data.id.into()) {
                player.rank = i as u16;
                if let Some(x) = data.pos {
                    if !player.visible {
                        player.pos = x;
                    }
                }

                player.is_spec = data.pos.is_none();
            } else {
                warn_unknown_player!(ScoreBoard, data.id);
            }
        }
    }
    fn handle_score_update(&mut self, packet: &ScoreUpdate) {
        if let Some(player) = self.players.get_mut(&packet.id.into()) {
            player.score = packet.score.into();
            player.earnings = packet.earnings.into();
            player.unused_upgrades = packet.upgrades;
            player.kills = packet.total_kills;
            player.deaths = packet.total_deaths;
        } else {
            warn_unknown_player!(ScoreUpdate, packet.id);
        }
    }

    fn handle_event_boost(&mut self, evt: &EventBoost) {
        self.clock = evt.clock;

        if let Some(player) = self.players.get_mut(&evt.id.into()) {
            player.keystate.boost = evt.boost;
            player.pos = evt.pos;
            player.rot = evt.rot;
            player.vel = evt.speed;
            player.energy = evt.energy;
            player.energy_regen = evt.energy_regen;
        } else {
            warn_unknown_player!(EventBounce, evt.id);
        }
    }
    fn handle_event_bounce(&mut self, evt: &EventBounce) {
        self.clock = evt.clock;

        if let Some(player) = self.players.get_mut(&evt.id.into()) {
            player.keystate = evt.keystate;
            player.pos = evt.pos;
            player.rot = evt.rot;
            player.vel = evt.speed;
        } else {
            warn_unknown_player!(EventBounce, evt.id);
        }
    }
    fn handle_event_leave_horizon(&mut self, evt: &EventLeaveHorizon) {
        use self::LeaveHorizonType::*;

        // FIXME: There's a note in airmash_protocol indicating
        //        that the values for this aren't accurate. This
        //        needs to be resolved.
        match evt.ty {
            Player => {
                if let Some(player) = self.players.get_mut(&evt.id) {
                    player.visible = false;
                } else {
                    warn_unknown_player!(EventLeaveHorizon, (evt.id, ()));
                }
            }
            Mob => {
                if let None = self.mobs.remove(&evt.id) {
                    warn_unknown_mob!(EventLeaveHorizon, (evt.id, ()));
                }
            }
        }
    }
    fn handle_event_repel(&mut self, evt: &EventRepel) {
        self.clock = evt.clock;

        if let Some(player) = self.players.get_mut(&evt.id.into()) {
            player.pos = evt.pos;
            player.rot = evt.rot;
            player.vel = evt.speed;
            player.energy = evt.energy;
            player.energy_regen = evt.energy_regen;
        } else {
            warn_unknown_player!(EventRepel, evt.id);
        }

        for data in evt.players.iter() {
            if let Some(player) = self.players.get_mut(&data.id.into()) {
                player.pos = data.pos;
                player.rot = data.rot;
                player.vel = data.speed;
                player.energy = data.energy;
                player.energy_regen = data.energy_regen;
                player.health = data.health;
                player.health_regen = data.health_regen;
            } else {
                warn_unknown_player!(EventRepel, data.id);
            }
        }

        for data in evt.mobs.iter() {
            if let Some(mob) = self.mobs.get_mut(&data.id.into()) {
                mob.pos = data.pos;
                mob.vel = data.speed;
                mob.accel = data.accel;
                mob.max_speed = data.max_speed;

                if mob.ty != data.ty {
                    warn!(
                        "Received EventRepel packet stating that the mob with id {} was {:?}, but that mob has a type of {:?}",
                        data.id.0,
                        data.ty,
                        mob.ty
                    );
                }
            } else {
                warn_unknown_mob!(EventRepel, data.id);
            }
        }
    }
    fn handle_event_stealth(&mut self, evt: &EventStealth) {
        if let Some(player) = self.players.get_mut(&evt.id.into()) {
            player.energy = evt.energy;
            player.energy_regen = evt.energy_regen;
            player.keystate.stealth = evt.state;
        } else {
            warn_unknown_player!(EventStealth, evt.id);
        }
    }
}
