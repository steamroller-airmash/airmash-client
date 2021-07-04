use bstr::BString;
use protocol::Vector2;

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use crate::{Config, protocol::server::*};
use crate::protocol::*;

#[derive(Default, Debug, Clone)]
pub struct Player {
    pub name: BString,
    pub flag: FlagCode,
    pub id: u16,
    pub team: Team,
    pub level: Option<u8>,
    pub plane: PlaneType,
    pub status: PlayerStatus,
    pub muted: bool,
    pub visible: bool,
    pub rank: u16,
    pub score: u32,
    pub earnings: u32,
    pub is_spec: bool,

    pub kills: u32,
    pub deaths: u32,
    pub captures: u32,

    pub pos: Position,
    pub rot: Rotation,
    pub vel: Velocity,
    pub health: Health,
    pub energy: Energy,
    pub health_regen: HealthRegen,
    pub energy_regen: EnergyRegen,

    pub flagspeed: bool,
    pub keystate: ServerKeyState,
    pub upgrades: Upgrades,
    pub unused_upgrades: u16,
}

impl Player {
    pub fn update(&mut self, packet: &PlayerUpdate) {
        self.pos = packet.pos;
        self.rot = packet.rot;
        self.vel = packet.speed;
        self.keystate = packet.keystate;
        self.upgrades = packet.upgrades;
        self.status = PlayerStatus::Alive;
    }

    pub fn update_time(&mut self, delta: f32, config: &Config) {
        if self.status == PlayerStatus::Dead {
            self.pos = Vector2::zeros();
            return;
        }

        let mut movement_angle = None;
        let info = &config.planes[self.plane];
        let boost_factor = match self.plane == PlaneType::Predator && self.keystate.boost {
            true => info.boost_factor,
            false => 1.0,
        };

        if self.keystate.strafe {
            if self.keystate.left {
                movement_angle = Some(self.rot - FRAC_PI_2);
            }
            if self.keystate.right {
                movement_angle = Some(self.rot + FRAC_PI_2);
            }
        } else {
            if self.keystate.left {
                self.rot -= delta * info.turn_factor;
            }
            if self.keystate.right {
                self.rot += delta * info.turn_factor;
            }
        }

        if self.keystate.up {
            if let Some(angle) = movement_angle {
                if self.keystate.right {
                    movement_angle = Some(angle - PI * 0.25);
                } else if self.keystate.left {
                    movement_angle = Some(angle + PI * 0.25);
                }
            } else {
                movement_angle = Some(self.rot);
            }
        } else if self.keystate.down {
            if let Some(angle) = movement_angle {
                if self.keystate.right {
                    movement_angle = Some(angle + PI * 0.25);
                } else if self.keystate.left {
                    movement_angle = Some(angle - PI * 0.25);
                }
            } else {
                movement_angle = Some(self.rot + PI);
            }
        }

        if let Some(angle) = movement_angle {
            let mult = info.accel_factor * delta * boost_factor;
            self.vel += Vector2::new(mult * angle.sin(), mult * -angle.cos());
        }

        let old_vel = self.vel;
        let speed = self.vel.norm();
        let mut max_speed = info.max_speed * boost_factor;
        let min_speed = info.min_speed;

        if self.upgrades.speed != 0 {
            max_speed *= config.upgrades.speed.factor[self.upgrades.speed as usize];
        }

        if self.upgrades.inferno {
            max_speed *= info.inferno_factor;
        }

        if self.keystate.flagspeed {
            max_speed = info.flag_speed;
        }

        if speed > max_speed {
            self.vel *= max_speed / speed;
        } else {
            if self.vel.x.abs() > min_speed || self.vel.y.abs() > min_speed {
                self.vel *= 1.0 - info.brake_factor * delta;
            } else {
                self.vel = Vector2::default();
            }
        }

        self.pos += old_vel * delta + (self.vel - old_vel) * delta * 0.5;
        self.rot = (self.rot % TAU + TAU) % TAU;

        let bound = Vector2::new(16352.0, 8160.0);
        if self.pos.x.abs() > bound.x {
            self.pos.x = self.pos.x.signum() * bound.x;
        }
        if self.pos.y.abs() > bound.y {
            self.pos.y = self.pos.y.signum() * bound.y;
        }
    }
}
