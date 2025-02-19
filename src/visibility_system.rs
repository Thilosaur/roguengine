use specs::prelude::*;
use super::{Viewshed, Position, Map, Player};
use std::collections::HashSet;

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = ( WriteExpect<'a, Map>,
                        Entities<'a>,
                        WriteStorage<'a, Viewshed>, 
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, Player>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, entities, mut viewshed, pos, player) = data;

        for (ent,viewshed,pos) in (&entities, &mut viewshed, &pos).join() {
            if viewshed.dirty {
                viewshed.dirty = false;
                viewshed.visible_tiles.clear();
                viewshed.visible_tiles = field_of_view(Point::new(pos.x,pos.y), viewshed.range, &*map);
                viewshed.visible_tiles.retain(|p| p.0 >= 0 && p.0 < map.width && p.1 >= 0 && p.1 < map.height );

                let p : Option<&Player> = player.get(ent);
                if let Some(p) = p {
                    for t in map.visible_tiles.iter_mut() { *t = false };
                    for vis in viewshed.visible_tiles.iter() {
                        let idx = map.xy_idx(vis.0, vis.1);
                        map.revealed_tiles[idx] = true;
                        map.visible_tiles[idx] = true;
                    }
                }
            }
        }
    }
}
#[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}
impl Point {
    pub fn new(x: i32, y: i32) -> Self{
        Point{x,y}
    }
}











struct ScanFovData<'a> {
    center: Point,
    dimensions: Point,
    range_2: i32,
    map: &'a Map,
    visible_points: &'a mut HashSet<Point>,
}

impl ScanFovData<'_> {
    fn is_transparent(&self, idx: usize, point: Point) -> bool {
        if self.map.in_bounds(point.x, point.y) {
            !self.map.is_opaque(idx)
        } else {
            false
        }
    }

    fn distance_to_center(&self, point: Point) -> f32 {
        let dx = i32::abs(point.x - self.center.x) as f32 - 0.5;
        let dy = i32::abs(point.y - self.center.y) as f32 - 0.5;
        dx * dx + dy * dy
    }

    fn insert_visible_for_vertical(&mut self, point: Point) -> bool {
        let idx = self.map.xy_idx(point.x, point.y);
        let mut is_visible = self.is_transparent(idx, point);

        if self.distance_to_center(point) <= self.range_2 as f32 {
            if point.x != self.center.x {
                self.visible_points.insert(point);
            }
        } else {
            is_visible = false;
        }
        is_visible
    }

    fn insert_visible_for_horizontal(&mut self, point: Point) -> bool {
        let idx = self.map.xy_idx(point.x,point.y);
        let mut is_visible = self.is_transparent(idx, point);

        if self.distance_to_center(point) <= self.range_2 as f32 {
            if self.center.y != point.y {
                self.visible_points.insert(point);
            }
        } else {
            is_visible = false;
        }
        is_visible
    }

    fn scan_N2NE(&mut self, distance: i32, start_slope: f32, end_slope: f32) {
        let mut start_slope = start_slope;

        if distance * distance > self.range_2 {
            return;
        }

        let mut current = Point::new(0, self.center.y - distance);
        if current.y < 0 {
            return;
        }

        current.x = self.center.x + (start_slope * distance as f32 + 0.5) as i32;
        if current.x >= self.dimensions.x {
            return;
        }

        let mut end_x = self.center.x + (end_slope * distance as f32 + 0.5) as i32;
        if end_x >= self.dimensions.x {
            end_x = self.dimensions.x - 1;
        }

        let idx = self.map.xy_idx(current.x,current.y);
        let mut last_visible = self.is_transparent(idx, current);
        for current_x in current.x..=end_x {
            current.x = current_x;

            let is_visible = self.insert_visible_for_vertical(current);

            if last_visible && !is_visible {
                self.scan_N2NE(
                    distance + 1,
                    start_slope,
                    ((current.x - self.center.x) as f32 - 0.5) / (distance as f32 + 0.5),
                );
            } else if !last_visible && is_visible {
                start_slope = ((current.x - self.center.x) as f32 - 0.5) / (distance as f32 - 0.5);
            }
            last_visible = is_visible;
        }
        if last_visible {
            self.scan_N2NE(distance + 1, start_slope, end_slope);
        }
    }

    fn scan_N2NW(&mut self, distance: i32, start_slope: f32, end_slope: f32) {
        let mut start_slope = start_slope;

        if distance * distance > self.range_2 {
            return;
        }

        let mut current = Point::new(0, self.center.y - distance);
        if current.y < 0 {
            return;
        }

        current.x = self.center.x - (start_slope * distance as f32 + 0.5) as i32;
        if current.x < 0 {
            return;
        }

        let mut end_x = self.center.x - (end_slope * distance as f32 + 0.5) as i32;
        if end_x < 0 {
            end_x = 0;
        }

        let idx = self.map.xy_idx(current.x,current.y);
        let mut last_visible = self.is_transparent(idx, current);
        while current.x >= end_x {
            let is_visible = self.insert_visible_for_vertical(current);

            if last_visible && !is_visible {
                self.scan_N2NW(
                    distance + 1,
                    start_slope,
                    ((self.center.x - current.x) as f32 - 0.5) / (distance as f32 + 0.5),
                );
            } else if !last_visible && is_visible {
                start_slope = ((self.center.x - current.x) as f32 - 0.5) / (distance as f32 - 0.5);
            }
            last_visible = is_visible;
            current.x -= 1;
        }
        if last_visible {
            self.scan_N2NW(distance + 1, start_slope, end_slope);
        }
    }

    fn scan_S2SE(&mut self, distance: i32, start_slope: f32, end_slope: f32) {
        let mut start_slope = start_slope;

        if distance * distance > self.range_2 {
            return;
        }

        let mut current = Point::new(0, self.center.y + distance);
        if current.y >= self.dimensions.y {
            return;
        }

        current.x = self.center.x + (start_slope * distance as f32 + 0.5) as i32;
        if current.x >= self.dimensions.x {
            return;
        }

        let mut end_x = self.center.x + (end_slope * distance as f32 + 0.5) as i32;
        if end_x >= self.dimensions.x {
            end_x = self.dimensions.x - 1;
        }

        let idx = self.map.xy_idx(current.x,current.y);
        let mut last_visible = self.is_transparent(idx, current);
        for current_x in current.x..=end_x {
            current.x = current_x;

            let is_visible = self.insert_visible_for_vertical(current);

            if last_visible && !is_visible {
                self.scan_S2SE(
                    distance + 1,
                    start_slope,
                    ((current.x - self.center.x) as f32 - 0.5) / (distance as f32 + 0.5),
                );
            } else if !last_visible && is_visible {
                start_slope = ((current.x - self.center.x) as f32 - 0.5) / (distance as f32 - 0.5);
            }
            last_visible = is_visible;
        }
        if last_visible {
            self.scan_S2SE(distance + 1, start_slope, end_slope);
        }
    }

    fn scan_S2SW(&mut self, distance: i32, start_slope: f32, end_slope: f32) {
        let mut start_slope = start_slope;

        if distance * distance > self.range_2 {
            return;
        }

        let mut current = Point::new(0, self.center.y + distance);
        if current.y >= self.dimensions.y {
            return;
        }

        current.x = self.center.x - (start_slope * distance as f32 + 0.5) as i32;
        if current.x < 0 {
            return;
        }

        let mut end_x = self.center.x - (end_slope * distance as f32 + 0.5) as i32;
        if end_x < 0 {
            end_x = 0;
        }

        let idx = self.map.xy_idx(current.x,current.y);
        let mut last_visible = self.is_transparent(idx, current);
        while current.x >= end_x {
            let is_visible = self.insert_visible_for_vertical(current);

            if last_visible && !is_visible {
                self.scan_S2SW(
                    distance + 1,
                    start_slope,
                    ((self.center.x - current.x) as f32 - 0.5) / (distance as f32 + 0.5),
                );
            } else if !last_visible && is_visible {
                start_slope = ((self.center.x - current.x) as f32 - 0.5) / (distance as f32 - 0.5);
            }
            last_visible = is_visible;
            current.x -= 1;
        }
        if last_visible {
            self.scan_S2SW(distance + 1, start_slope, end_slope);
        }
    }

    fn scan_E2SE(&mut self, distance: i32, start_slope: f32, end_slope: f32) {
        let mut start_slope = start_slope;

        if distance * distance > self.range_2 {
            return;
        }

        let mut current = Point::new(self.center.x + distance, 0);
        if current.x >= self.dimensions.x {
            return;
        }

        current.y = self.center.y + (start_slope * distance as f32 + 0.5) as i32;
        if current.y >= self.dimensions.y {
            return;
        }

        let mut end_y = self.center.y + (end_slope * distance as f32 + 0.5) as i32;
        if end_y >= self.dimensions.y {
            end_y = self.dimensions.y - 1;
        }

        let idx = self.map.xy_idx(current.x,current.y);
        let mut last_visible = self.is_transparent(idx, current);
        for current_y in current.y..=end_y {
            current.y = current_y;

            let is_visible = self.insert_visible_for_horizontal(current);

            if last_visible && !is_visible {
                self.scan_E2SE(
                    distance + 1,
                    start_slope,
                    ((current.y - self.center.y) as f32 - 0.5) / (distance as f32 + 0.5),
                );
            } else if !last_visible && is_visible {
                start_slope = ((current.y - self.center.y) as f32 - 0.5) / (distance as f32 - 0.5);
            }
            last_visible = is_visible;
        }
        if last_visible {
            self.scan_E2SE(distance + 1, start_slope, end_slope);
        }
    }

    fn scan_E2NE(&mut self, distance: i32, start_slope: f32, end_slope: f32) {
        let mut start_slope = start_slope;

        if distance * distance > self.range_2 {
            return;
        }

        let mut current = Point::new(self.center.x + distance, 0);
        if current.x >= self.dimensions.x {
            return;
        }

        current.y = self.center.y - (start_slope * distance as f32 + 0.5) as i32;
        if current.y < 0 {
            return;
        }

        let mut end_y = self.center.y - (end_slope * distance as f32 + 0.5) as i32;
        if end_y < 0 {
            end_y = 0;
        }

        let idx = self.map.xy_idx(current.x,current.y);
        let mut last_visible = self.is_transparent(idx, current);
        while current.y >= end_y {
            let is_visible = self.insert_visible_for_horizontal(current);

            if last_visible && !is_visible {
                self.scan_E2NE(
                    distance + 1,
                    start_slope,
                    ((self.center.y - current.y) as f32 - 0.5) / (distance as f32 + 0.5),
                );
            } else if !last_visible && is_visible {
                start_slope = ((self.center.y - current.y) as f32 - 0.5) / (distance as f32 - 0.5);
            }
            last_visible = is_visible;
            current.y -= 1;
        }
        if last_visible {
            self.scan_E2NE(distance + 1, start_slope, end_slope);
        }
    }

    fn scan_W2SW(&mut self, distance: i32, start_slope: f32, end_slope: f32) {
        let mut start_slope = start_slope;

        if distance * distance > self.range_2 {
            return;
        }

        let mut current = Point::new(self.center.x - distance, 0);
        if current.x < 0 {
            return;
        }

        current.y = self.center.y + (start_slope * distance as f32 + 0.5) as i32;
        if current.y >= self.dimensions.y {
            return;
        }

        let mut end_y = self.center.y + (end_slope * distance as f32 + 0.5) as i32;
        if end_y >= self.dimensions.y {
            end_y = self.dimensions.y - 1;
        }

        let idx = self.map.xy_idx(current.x,current.y);
        let mut last_visible = self.is_transparent(idx, current);
        for current_y in current.y..=end_y {
            current.y = current_y;

            let is_visible = self.insert_visible_for_horizontal(current);

            if last_visible && !is_visible {
                self.scan_W2SW(
                    distance + 1,
                    start_slope,
                    ((current.y - self.center.y) as f32 - 0.5) / (distance as f32 + 0.5),
                );
            } else if !last_visible && is_visible {
                start_slope = ((current.y - self.center.y) as f32 - 0.5) / (distance as f32 - 0.5);
            }
            last_visible = is_visible;
        }
        if last_visible {
            self.scan_W2SW(distance + 1, start_slope, end_slope);
        }
    }

    fn scan_W2NW(&mut self, distance: i32, start_slope: f32, end_slope: f32) {
        let mut start_slope = start_slope;

        if distance * distance > self.range_2 {
            return;
        }

        let mut current = Point::new(self.center.x - distance, 0);
        if current.x < 0 {
            return;
        }

        current.y = self.center.y - (start_slope * distance as f32 + 0.5) as i32;
        if current.y < 0 {
            return;
        }

        let mut end_y = self.center.y - (end_slope * distance as f32 + 0.5) as i32;
        if end_y < 0 {
            end_y = 0;
        }

        let idx = self.map.xy_idx(current.x,current.y);
        let mut last_visible = self.is_transparent(idx, current);
        while current.y >= end_y {
            let is_visible = self.insert_visible_for_horizontal(current);

            if last_visible && !is_visible {
                self.scan_W2NW(
                    distance + 1,
                    start_slope,
                    ((self.center.y - current.y) as f32 - 0.5) / (distance as f32 + 0.5),
                );
            } else if !last_visible && is_visible {
                start_slope = ((self.center.y - current.y) as f32 - 0.5) / (distance as f32 - 0.5);
            }
            last_visible = is_visible;
            current.y -= 1;
        }
        if last_visible {
            self.scan_W2NW(distance + 1, start_slope, end_slope);
        }
    }
}

pub fn field_of_view_set(center: Point, range: i32, map: &Map) -> HashSet<Point> {
    let mut visible_points: HashSet<Point> =
        HashSet::with_capacity(((range * 2) * (range * 2)) as usize);

    visible_points.insert(center);

    /* N, NE, E, SE, S, SW, W, NW */
    const SECTORS: [(i32, i32); 8] = [ (0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), ];

    let r2 = range * range;
    

    // Add visibility for every 45 degree line:
    let mut visibility_per_sector = [false; 8];
    for (i, (dx, dy)) in SECTORS.iter().enumerate() {
        let mut current = center;
        loop {
            current = Point::new(current.x + dx, current.y + dy);
            if current.x < 0 || current.x >= map.width
                || current.y < 0 || current.y >= map.height
            {
                break;
            }
            let x2 = current.x - center.x;
            let x2 = x2 * x2;
            let y2 = current.y - center.y;
            let y2 = y2 * y2;
            if x2 + y2 > r2 {
                break;
            }

            let idx = map.xy_idx(current.x, current.y);
            visible_points.insert(current);
            if map.is_opaque(idx) {
                break;
            }
            visibility_per_sector[i] = true;
        }
    }

    let mut scanner = ScanFovData {
        center,
        dimensions: Point::new(map.width,map.height),
        range_2: r2,
        map: &map,
        visible_points: &mut visible_points,
    };
    if visibility_per_sector[0] {
        scanner.scan_N2NW(1, 0., 1.);
        scanner.scan_N2NE(1, 0., 1.);
    }

    if visibility_per_sector[2] {
        scanner.scan_E2NE(1, 0., 1.);
        scanner.scan_E2SE(1, 0., 1.);
    }

    if visibility_per_sector[4] {
        scanner.scan_S2SE(1, 0., 1.);
        scanner.scan_S2SW(1, 0., 1.);
    }

    if visibility_per_sector[6] {
        scanner.scan_W2SW(1, 0., 1.);
        scanner.scan_W2NW(1, 0., 1.);
    }

    visible_points
        .iter()
        .copied()
        .filter(|p| map.in_bounds(p.x, p.y))
        .collect()
}

/// Calculates field-of-view for a map that supports Algorithm2D.
pub fn field_of_view(start: Point, range: i32, map: &Map) -> Vec<(i32,i32)> {
    let mut visi_points: Vec<(i32,i32)> = Vec::new();

    for i in field_of_view_set(start, range, map).into_iter() {
        visi_points.push((i.x,i.y));
    };

    visi_points
}
