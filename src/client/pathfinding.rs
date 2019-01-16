use std::f32::consts::PI;
use std::time::{Duration, Instant};
//use hashbrown::HashSet;
//use std::collections::BinaryHeap;
use airmash_protocol::*;

use super::*;
use crate::consts::BASE_DIR;
use crate::Handler;
//use crate::map::MAP;

use protocol::Position;

impl<H: Handler> Client<H> {
    fn calc_angle(&mut self, pos: Position) -> f32 {
        let rel = (pos - self.world.get_me().pos).normalized();
        let mut angle = Vector2::dot(rel, BASE_DIR).acos();

        if rel.x < 0.0.into() {
            angle = 2.0 * PI - angle;
        }

        angle
    }

    pub async fn run_straight_at(&mut self, pos: Position) -> ClientResult<()> {
        r#await!(self.point_at(pos))?;
        r#await!(self.press_key(KeyCode::Up))?;
        r#await!(self.wait(Duration::from_millis(self.world.ping as u64 * 2)))?;

        while let Some(_) = r#await!(self.next())? {
            let dist = (pos - self.world.get_me().pos).length();
            let angle = self.calc_angle(pos);

            if angle > 1.0 {
                if dist.inner() < 500.0 {
                    r#await!(self.release_key(KeyCode::Up))?;
                }

                r#await!(self.point_at(pos))?;

                if dist.inner() < 500.0 || !self.world.get_me().keystate.up {
                    r#await!(self.press_key(KeyCode::Up))?;
                }

                r#await!(self.wait(Duration::from_millis(self.world.ping.into())))?;
            }

            if dist.inner() < 100.0 {
                break;
            }
        }

        r#await!(self.press_key(KeyCode::Down))?;
        r#await!(self.wait(Duration::from_millis(100)))?;
        r#await!(self.release_key(KeyCode::Down))?;

        r#await!(self.release_key(KeyCode::Up))
    }

    pub async fn follow(&mut self, player: u16) -> ClientResult<()> {
        let mut pos;
        let mut prev = Instant::now();
        r#await!(self.press_key(KeyCode::Up))?;
        while let Some(_) = r#await!(self.next())? {
            if let Some(p) = self.world.players.get(&player) {
                pos = p.pos;

                let mypos = self.world.get_me().pos;
                if (pos - mypos).length() < 200.0.into() {
                    //break;
                }
            } else {
                break;
            }
            if Instant::now() - prev > Duration::from_millis(500) {
                r#await!(self.press_key(KeyCode::Up))?;
                prev = Instant::now();
            }

            r#await!(self.point_at(pos))?;
            r#await!(self.wait(Duration::from_millis(
                (self.world.ping * 2).min(1000).max(10) as u64
            )))?;
        }

        r#await!(self.release_key(KeyCode::Up))
    }
}
