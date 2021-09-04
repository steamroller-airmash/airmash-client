#![feature(core_intrinsics)]

use std::sync::atomic::{AtomicUsize, Ordering};

use airmash_client::map::map;
use airmash_client::protocol::Vector2;
use rayon::prelude::*;
use roaring::RoaringBitmap;

fn transform_octant(r: f32, c: f32, oct: u8) -> Vector2<f32> {
  let [x, y] = match oct {
    0 => [c, -r],
    1 => [r, -c],
    2 => [r, c],
    3 => [c, r],
    4 => [-c, r],
    5 => [-r, c],
    6 => [-r, -c],
    7 => [-c, -r],
    _ => unimplemented!(),
  };

  Vector2::new(x, y)
}

#[derive(Clone, Copy)]
struct Shadow {
  start: f32,
  end: f32,
}

impl Shadow {
  pub fn new(start: f32, end: f32) -> Self {
    Self { start, end }
  }

  pub fn project(r: f32, c: f32) -> Self {
    let tl = c / (r + 2.0);
    let br = (c + 1.0) / (r + 1.0);
    Self::new(tl, br)
  }

  pub fn contains(&self, other: Self) -> bool {
    (self.start <= other.start && other.start <= self.end)
      || (other.start <= self.end && other.end <= self.end)
  }
}

fn out_of_bounds(pos: Vector2<f32>) -> bool {
  pos.x < 0.0 || pos.x > 511.0 || pos.y < 0.0 || pos.y > 255.0
}

#[derive(Default)]
struct ShadowLine {
  list: Vec<Shadow>,
}

impl ShadowLine {
  pub fn shadowed(&self, shadow: Shadow) -> bool {
    for s in self.list.iter() {
      if s.contains(shadow) {
        return true;
      }
    }

    false
  }

  pub fn push(&mut self, shadow: Shadow) {
    if self.list.is_empty() {
      self.list.push(shadow);
      return;
    }

    let mut index = self.list.len() - 1;
    for (i, s) in self.list.iter().enumerate() {
      if s.start >= shadow.start {
        index = i;
        break;
      }
    }

    let mut o_prev = None;
    let mut o_next = None;

    if index > 0 && self.list[index - 1].end >= shadow.start {
      o_prev = Some(index - 1);
    }

    if index < self.list.len() && self.list[index].start <= shadow.end {
      o_next = Some(index);
    }

    match (o_prev, o_next) {
      (Some(prev), Some(next)) => {
        self.list[prev].end = self.list[next].end;
        self.list.remove(index);
      }
      (None, Some(next)) => self.list[next].start = shadow.start,
      (Some(prev), None) => self.list[prev].end = shadow.end,
      (None, None) => self.list.insert(index, shadow),
    }
  }
}

const BOUND: usize = 50;

fn make_index(src: [usize; 2], dst: [usize; 2]) -> u64 {
  assert!(src[0] < 512, "{} >= {}", src[0], 512);
  assert!(src[1] < 256, "{} >= {}", src[1], 512);
  assert!(dst[0] < 2 * BOUND, "{} >= {}", dst[0], 2 * BOUND);
  assert!(dst[1] < 2 * BOUND, "{} >= {}", dst[1], 2 * BOUND);

  let [x0, y0] = [src[0] as u64, src[1] as u64];
  let [x1, y1] = [dst[0] as u64, dst[1] as u64];

  let main = x0 | (y0 * 512);
  let rest = x1 * 2 * (BOUND as u64) + y1;

  (rest * 256 * 512) | main
}

fn main() {
  let map = map();
  static COUNT: AtomicUsize = AtomicUsize::new(0);
  static LINE: AtomicUsize = AtomicUsize::new(0);

  let output = (0..map.width())
    .into_par_iter()
    .map(|sx| {
      let mut values = Vec::new();

      for sy in 0..map.height() {
        let start = Vector2::new(sx as f32, sy as f32);

        for oct in 0..8 {
          let mut line = ShadowLine::default();

          for r in 1..BOUND {
            let pos = start + transform_octant(r as _, 0.0, oct);
            if out_of_bounds(pos) {
              break;
            }

            for c in 0..r {
              let pos = start + transform_octant(r as _, c as _, oct);
              let rpos = [(pos.x + 0.5) as usize, (pos.y + 0.5) as usize];
              let [rx, ry] = [rpos[0] + BOUND - sx, rpos[1] + BOUND - sy];

              if out_of_bounds(pos) {
                break;
              }

              let proj = Shadow::project(r as _, c as _);

              if map.get(rpos[0], rpos[1]) {
                line.push(proj);
              } else if !line.shadowed(proj) {
                values.push(make_index([sx, sy], [rx, ry]));
                COUNT.fetch_add(1, Ordering::Relaxed);
              }
            }
          }
        }
      }

      values.sort_unstable();
      values.dedup();

      eprintln!(
        "status: {: >3}/{}",
        LINE.fetch_add(1, Ordering::Relaxed) + 1,
        map.width()
      );

      RoaringBitmap::from_sorted_iter(values.into_iter().map(|x| x as _))
    })
    .reduce(
      || RoaringBitmap::new(),
      |mut a, b| {
        a |= b;
        a
      },
    );
  //   }
  // }

  let stdout = std::io::stdout();
  let mut stdout = stdout.lock();

  let _ = output.serialize_into(&mut stdout);

  eprintln!(
    "occupancy: {}",
    COUNT.load(Ordering::Relaxed) as f64 / (512usize * 512 * 4 * BOUND * BOUND) as f64
  );
}
