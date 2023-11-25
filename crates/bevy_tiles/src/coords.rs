/// Calculate the coordinate of a chunk from a given tile coordinate and chunk size
#[inline]
pub fn calculate_chunk_coordinate<const N: usize>(
    mut tile_c: [isize; N],
    chunk_size: usize,
) -> [isize; N] {
    for i in tile_c.iter_mut() {
        *i = *i / (chunk_size as isize) - if *i < 0 { 1 } else { 0 }
    }
    tile_c
}

/// Calculate the coordinate of a tile relative to the origin of it's chunk.
#[inline]
pub fn calculate_chunk_relative_tile_coordinate<const N: usize>(
    mut tile_c: [isize; N],
    chunk_size: usize,
) -> [isize; N] {
    for i in tile_c.iter_mut() {
        *i %= chunk_size as isize;
        if *i < 0 {
            *i += chunk_size as isize;
        }
    }
    tile_c
}

/// Calculate the index of a tile within it's chunk.
#[inline]
pub fn calculate_tile_index<const N: usize>(tile_c: [isize; N], chunk_size: usize) -> usize {
    let mut index = 0;
    let relative_tile_c = calculate_chunk_relative_tile_coordinate(tile_c, chunk_size);
    for (i, c) in relative_tile_c.iter().enumerate() {
        index += (*c as usize) * chunk_size.pow(i as u32);
    }
    index
}

/// Calculate the coordinate of a tile from it's index in a chunk, and the chunk coordinate.
#[inline]
pub fn calculate_tile_coordinate<const N: usize>(
    chunk_c: [isize; N],
    tile_i: usize,
    chunk_size: usize,
) -> [isize; N] {
    let mut chunk_world_c = chunk_c.map(|c| c * chunk_size as isize);
    for (i, c) in chunk_world_c.iter_mut().enumerate() {
        if i == 0 {
            *c += (tile_i % chunk_size) as isize;
        } else {
            *c += (tile_i / chunk_size.pow(i as u32)) as isize;
        }
    }
    chunk_world_c
}

/// Find the highest index possible in a chunk.
#[inline]
pub fn max_tile_index<const N: usize>(chunk_size: usize) -> usize {
    let mut index = 0;
    for i in 1..=N {
        index += chunk_size.pow(i as u32);
    }
    index - 1
}

/// Calculate the tile coordinate given a world coordinate
/// and the scale_f of the tile coordinates to world coordinates.
/// (For example, if tiles are being represented by 16x16 pixel sprites,
/// the scale factor should be set to 16)
#[inline]
pub fn world_to_tile<const N: usize>(world_c: [f32; N], scale_f: f32) -> [isize; N] {
    world_c.map(|c| (c / scale_f - if c < 0.0 { 1.0 } else { 0.0 }) as isize)
}

/// Allows for iteration between all coordinates in between two corners.
pub struct CoordIterator<const N: usize> {
    corner_1: [isize; N],
    corner_2: [isize; N],
    current: [isize; N],
    complete: bool,
}

impl<const N: usize> CoordIterator<N> {
    /// Create an iterator that iterates through each point created by the bounding of two corners.
    pub fn new(mut corner_1: [isize; N], mut corner_2: [isize; N]) -> Self {
        for i in 0..N {
            if corner_1[i] > corner_2[i] {
                std::mem::swap(&mut corner_1[i], &mut corner_2[i]);
            };
        }

        Self {
            corner_1,
            corner_2,
            current: corner_1,
            complete: false,
        }
    }
}

impl<const N: usize> Iterator for CoordIterator<N> {
    type Item = [isize; N];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.complete {
            return None;
        }

        let ret = self.current;

        if self.current == self.corner_2 {
            self.complete = true;
        } else {
            for i in 0..N {
                if self.current[i] == self.corner_2[i] {
                    self.current[i] = self.corner_1[i];
                    continue;
                }
                self.current[i] += 1;
                break;
            }
        }

        Some(ret)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use std::ops::RangeInclusive;

    use super::*;

    fn make_range_iter(val_1: isize, val_2: isize) -> RangeInclusive<isize> {
        if val_1 < val_2 {
            val_1..=val_2
        } else {
            val_2..=val_1
        }
    }

    #[rstest]
    #[case([0, 0, 0], [3, 3, 3])]
    #[case([3, 3, 3], [0, 0, 0])]
    #[case([0, 3, 0], [3, 0, 3])]
    #[case([0, 3, 0], [3, 3, 3])]
    #[case([0, 3, 0], [0, 0, 3])]
    #[case([3, 3, 3], [3, 3, 3])]
    fn coord_iter(#[case] corner_1: [isize; 3], #[case] corner_2: [isize; 3]) {
        let mut iter = CoordIterator::new(corner_1, corner_2);

        for z in make_range_iter(corner_1[2], corner_2[2]) {
            for y in make_range_iter(corner_1[1], corner_2[1]) {
                for x in make_range_iter(corner_1[0], corner_2[0]) {
                    let next = iter.next();
                    println!("Iter: {:?}", next);
                    assert_eq!(Some([x, y, z]), next);
                }
            }
        }

        let next = iter.next();
        println!("Fin: {:?}", next);
        assert_eq!(None, next);
    }

    #[rstest]
    #[case(16, [15, 0], 15)]
    #[case(16, [0, 15], 240)]
    #[case(16, [15, 15], 255)]
    #[case(16, [-1, -1], 255)]
    #[case(16, [-16, -16], 0)]
    fn tile_index_test(
        #[case] chunk_size: usize,
        #[case] tile_c: [isize; 2],
        #[case] index: usize,
    ) {
        assert_eq!(calculate_tile_index(tile_c, chunk_size), index)
    }
}
