use once_cell::sync::OnceCell;
use roaring::RoaringBitmap;

mod map_data;
mod vis_data;

const BOUND: usize = 10;
static MAP: OnceCell<MapData> = OnceCell::new();

pub struct MapData {
  data: RoaringBitmap,
}

impl MapData {
  pub fn get(&self, x: usize, y: usize) -> bool {
    if x >= self.width() || y >= self.height() {
      return false;
    }

    self.data.contains((y * 512 + x) as _)
  }

  pub const fn width(&self) -> usize {
    512
  }

  pub const fn height(&self) -> usize {
    256
  }

  pub fn walls<'a>(&'a self) -> impl Iterator<Item = [usize; 2]> + 'a {
    self
      .data
      .iter()
      .map(|v| v as usize)
      .map(move |v| [v % self.width(), v / self.width()])
  }
}

pub fn map() -> &'static MapData {
  MAP.get_or_init(|| MapData {
    data: RoaringBitmap::from_sorted_iter(self::map_data::MAP_DATA.iter().map(|x| *x)),
  })
}

pub struct VisData {

}