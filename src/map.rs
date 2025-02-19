use raylib::prelude::*;
use specs::{Entity, Join, World, WorldExt};
use std::cmp::{min,max};
use rand::Rng;
use crate::Viewshed;

use super::{SCALE,TILE_SIZE,Rect, Player};

pub const MAPWIDTH : usize = 80;
pub const MAPHEIGHT : usize = 50;
pub const MAPCOUNT : usize = MAPHEIGHT * MAPWIDTH;


#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall, Floor
}


pub struct Map {
    pub tiles : Vec<TileType>,
    pub rooms : Vec<Rect>,
    pub width : i32,
    pub height : i32,
    pub revealed_tiles : Vec<bool>,
    pub visible_tiles : Vec<bool>,
    pub blocked : Vec<bool>,
    pub tile_content : Vec<Vec<Entity>>
}


impl Map {
    ///translates an x and y coordinate to an index in the map vector
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    pub fn is_opaque(&self, idx:usize) -> bool {
        self.tiles[idx as usize] == TileType::Wall
    }

    pub fn in_bounds(&self,x: i32, y: i32) -> bool {
        x >= 0 && x < self.width && y >= 0 && y < self.height
    }



    pub fn populate_blocked(&mut self) {
        for (i,tile) in self.tiles.iter_mut().enumerate() {
            self.blocked[i] = *tile == TileType::Wall;
        }
    }


    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    pub fn new_map_rooms_and_corridors() -> Map {
        let mut map = Map{
            tiles: vec![TileType::Wall;MAPCOUNT],
            rooms: Vec::new(),
            width: 80,
            height: 50,
            revealed_tiles: vec![false;MAPCOUNT],
            visible_tiles: vec![false;MAPCOUNT],
            blocked: vec![false;MAPCOUNT],
            tile_content : vec![Vec::new(); MAPCOUNT]
        };

        const MAX_ROOMS : i32 = 30;
        const MIN_SIZE : i32 = 6;
        const MAX_SIZE : i32 = 10;

        let mut rng = rand::thread_rng();

        for _ in 0..MAX_ROOMS {
            let w = rng.gen_range(MIN_SIZE..MAX_SIZE);
            let h = rng.gen_range(MIN_SIZE..MAX_SIZE);
            let x = rng.gen_range(1..(80 - w - 1)) - 1;
            let y = rng.gen_range(1..(50 - h - 1)) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) { ok = false }
            }
            if ok {
                map.apply_room_to_map(&new_room);

                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len()-1].center();
                    if rng.gen_range(0..2) == 1 {
                        map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                        map.apply_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        map.apply_vertical_tunnel(prev_y, new_y, prev_x);
                        map.apply_horizontal_tunnel( prev_x, new_x, new_y);
                    }
                }

                map.rooms.push(new_room);         
            }
        }

        map
    }


    
    fn is_exit_valid(&self, x:i32, y:i32) -> bool {
        if x < 1 || x > self.width-1 || y < 1 || y > self.height-1 { return false; }
        let idx = self.xy_idx(x, y);
        !self.blocked[idx]
    }

    pub fn get_available_exits(&self, idx:usize) -> Vec<(usize, f32)> {
        let mut exits = Vec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;

        // Cardinal directions
        if self.is_exit_valid(x-1, y) { exits.push((idx-1, 1.0)) };
        if self.is_exit_valid(x+1, y) { exits.push((idx+1, 1.0)) };
        if self.is_exit_valid(x, y-1) { exits.push((idx-w, 1.0)) };
        if self.is_exit_valid(x, y+1) { exits.push((idx+w, 1.0)) };

        // Diagonals
        if self.is_exit_valid(x-1, y-1) { exits.push(((idx-w)-1, 1.45)); }
        if self.is_exit_valid(x+1, y-1) { exits.push(((idx-w)+1, 1.45)); }
        if self.is_exit_valid(x-1, y+1) { exits.push(((idx+w)-1, 1.45)); }
        if self.is_exit_valid(x+1, y+1) { exits.push(((idx+w)+1, 1.45)); }

        exits
    }

    pub fn get_available_cardinal_exits(&self, idx:usize) -> Vec<(usize, f32)> {
        let mut exits = Vec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;

        // Cardinal directions
        if self.is_exit_valid(x-1, y) { exits.push((idx-1, 1.0)) };
        if self.is_exit_valid(x+1, y) { exits.push((idx+1, 1.0)) };
        if self.is_exit_valid(x, y-1) { exits.push((idx-w, 1.0)) };
        if self.is_exit_valid(x, y+1) { exits.push((idx+w, 1.0)) };
        exits
    }

    pub fn get_pathing_distance(&self, idx1:usize, idx2:usize) -> f32 {
        let w = self.width as usize;
        let p1 = (idx1 % w, idx1 / w);
        let p2 = (idx2 % w, idx2 / w);
        //pythagorus distance crap
        Self::distance2d_pythagoras(p1.0 as i32,p1.1 as i32,p2.0 as i32,p2.1 as i32)
    }


    pub fn distance2d_pythagoras(start_x: i32, start_y: i32, end_x: i32, end_y: i32) -> f32 {
        let dsq = Self::distance2d_pythagoras_squared(start_x,start_y,end_x, end_y);
        f32::sqrt(dsq)
    }

    fn distance2d_pythagoras_squared(start_x: i32, start_y: i32, end_x: i32, end_y: i32) -> f32 {
        let dx = (max(start_x, end_x) - min(start_x, end_x)) as f32;
        let dy = (max(start_y, end_y) - min(start_y, end_y)) as f32;
        (dx * dx) + (dy * dy)
    }



    fn apply_room_to_map(&mut self,room : &Rect) {
        for y in room.y1 +1 ..= room.y2 {
            for x in room.x1 + 1 ..= room.x2 {
                let idx = self.xy_idx(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }


    fn apply_horizontal_tunnel(&mut self, x1:i32, x2:i32, y:i32) {
        for x in min(x1,x2) ..= max(x1,x2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < MAPCOUNT {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_tunnel(&mut self, y1:i32, y2:i32, x:i32) {
        for y in min(y1,y2) ..= max(y1,y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < MAPCOUNT {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }
}

pub fn draw_map(ecs: &World, draw: &mut RaylibDrawHandle, tileset: &Vec<Texture2D>) {
    
    let map = ecs.fetch::<Map>();
    let mut y = 0;
    let mut x = 0;
    for (idx,tile) in map.tiles.iter().enumerate() {
        let sprite;
        let mut fg = Color::WHITE;
        // Render a tile depending upon the tile type
        if map.revealed_tiles[idx] { 
            match tile {
                TileType::Floor => {
                    //draw.draw_texture_ex(&tileset[17],Vector2::new((x * TILE_SIZE) as f32 * SCALE ,(y * TILE_SIZE) as f32 * SCALE),0.0,SCALE,Color::WHITE);
                    sprite = &tileset[17];
                }
                TileType::Wall => {
                    //draw.draw_texture_ex(&tileset[1],Vector2::new((x * TILE_SIZE) as f32 * SCALE ,(y * TILE_SIZE) as f32 * SCALE),0.0,SCALE,Color::WHITE);
                    sprite = &tileset[1];
                }
            }
            if !map.visible_tiles[idx] { fg = Color::GRAY};
            draw.draw_texture_ex(sprite, Vector2::new((x * TILE_SIZE) as f32 * SCALE ,(y * TILE_SIZE) as f32 * SCALE), 0.0, SCALE, fg);
            //if map.blocked[idx] {
            //    draw.draw_pixel_v(Vector2::new((x * TILE_SIZE) as f32 * SCALE ,(y * TILE_SIZE) as f32 * SCALE), Color::RED);
            //}
        }
        // Move the coordinates
        x += 1;
        if x > (MAPWIDTH - 1) as i32 {
            x = 0;
            y += 1;
        }
    }
}

