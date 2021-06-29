use std::f32::consts::PI;
use std::time::{Duration, Instant};
//use hashbrown::HashSet;
//use std::collections::BinaryHeap;
use airmash_protocol::*;

use super::*;
use crate::consts::BASE_DIR;
//use crate::map::MAP;

use protocol::Position;

impl Client {
    fn calc_angle(&mut self, pos: Position) -> f32 {
        let rel = (pos - self.world.get_me().pos).normalize();
        let mut angle = Vector2::dot(&rel, &BASE_DIR).acos();

        if rel.x < 0.0.into() {
            angle = 2.0 * PI - angle;
        }

        angle
    }

    pub async fn run_straight_at(&mut self, pos: Position) -> ClientResult<()> {
        (self.point_at(pos)).await?;
        (self.press_key(KeyCode::Up)).await?;
        (self.wait(Duration::from_millis(self.world.ping as u64 * 2))).await?;

        while let Some(_) = self.next().await? {
            let dist = (pos - self.world.get_me().pos).norm();
            let angle = self.calc_angle(pos);

            if angle > 1.0 {
                if dist < 500.0 {
                    self.release_key(KeyCode::Up).await?;
                }

                self.point_at(pos).await?;

                if dist < 500.0 || !self.world.get_me().keystate.up {
                    self.press_key(KeyCode::Up).await?;
                }

                self.wait(Duration::from_millis(self.world.ping.into())).await?;
            }

            if dist < 100.0 {
                break;
            }
        }

        self.press_key(KeyCode::Down).await?;
        self.wait(Duration::from_millis(100)).await?;
        self.release_key(KeyCode::Down).await?;

        self.release_key(KeyCode::Up).await
    }

    pub async fn follow(&mut self, player: u16) -> ClientResult<()> {
        let mut pos;
        let mut prev = Instant::now();
        self.press_key(KeyCode::Up).await?;
        while let Some(_) = self.next().await? {
            if let Some(p) = self.world.players.get(&player) {
                pos = p.pos;

                let mypos = self.world.get_me().pos;
                if (pos - mypos).norm() < 200.0 {
                    //break;
                }
            } else {
                break;
            }
            if Instant::now() - prev > Duration::from_millis(500) {
                self.press_key(KeyCode::Up).await?;
                prev = Instant::now();
            }

            self.point_at(pos).await?;
            self.wait(Duration::from_millis(
                (self.world.ping * 2).min(1000).max(10) as u64
            )).await?;
        }

        self.release_key(KeyCode::Up).await
    }
}
