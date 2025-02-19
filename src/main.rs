mod map;
mod player;
mod components;
mod rect;
mod visibility_system;
mod monster_ai_system;
mod event_log;
mod state;
mod astar;
mod map_indexing_system;
mod melee_combat_system;
mod damage_system;


pub use map::*;
pub use player::*;
pub use components::*;
pub use rect::*;
pub use visibility_system::*;
pub use monster_ai_system::*;
pub use event_log::*;
pub use state::*;
pub use astar::*;
pub use map_indexing_system::*;
pub use melee_combat_system::*;
pub use damage_system::*;

use rand::{thread_rng, Rng};
use specs::prelude::*;
use raylib::prelude::*;


pub const TILE_SIZE : i32 = 8;
pub const SCALE : f32 = 1.5;


fn main() {
    let mut gs = State {
        ecs: World::new(),
    };

    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<SufferDamage>();

    
    gs.ecs.insert(RunState::PreRun);

    let map = map::Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();

    let mut rng = thread_rng();
    for (i,room) in map.rooms.iter().skip(1).enumerate() {
        let (x,y) = room.center();
        let sprite : u8;
        let name : String;
        let roll = rng.gen_range(1..3);
        match roll {
            1 => { sprite = 13; name = "beholder".to_string()}
            _ => { sprite = 23; name = "motherfuckingcrab".to_string()}
        }
        gs.ecs.create_entity()
            .with(Position{ x, y })
            .with(Renderable{
                index: sprite,
                color: Color::WHITE,
            })
            .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
            .with(Monster{ seen_player: false, known_player_location : (0,0)})
            .with(Name{ name: format!("{} #{}", &name, i)})
            .with(BlocksTile{})
            .with(CombatStats{ max_hp: 16, hp: 16, defense: 1, power: 4 })
            .build();
    }
    
    gs.ecs.insert(Point::new(player_x, player_y));
    gs.ecs.insert(map);

    let log = EventLog::new();
    gs.ecs.insert(log);

    
    let player_entity = gs.ecs
        .create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            index: 8,
            color: Color::WHITE,
        })
        .with(Player{health : 100})
        .with(Viewshed{visible_tiles: Vec::new(), range : 8, dirty: true})
        .with(Name{name: "Player".to_string()})
        .with(CombatStats{max_hp: 30, hp: 30, defense: 2, power: 5})
        .build();

    gs.ecs.insert(player_entity);

    

    let (mut rl, thread) = raylib::init()
        .size(MAPWIDTH as i32 * (TILE_SIZE as f32 * SCALE) as i32,  MAPHEIGHT as i32 * (TILE_SIZE as f32 * SCALE) as i32)
        .title("RogueLike")
        .build();

    
    let (_image,tileset,mut rl,thread) = load_tile_set(rl,thread, "tilemap2.png");

    while !rl.window_should_close() {
        
        gs.tick(&mut rl,thread.clone(), &tileset)
        
    }
}




///loads the tile set vector from a specified path
fn load_tile_set(mut rl:RaylibHandle, thread: RaylibThread, path: &str) -> (Image,Vec<Texture2D>,RaylibHandle, RaylibThread) {
	  let image: Image = Image::load_image(path).unwrap();
	  let mut tileset: Vec<Texture2D> = Vec::new();
    
    let width = image.width()/TILE_SIZE;
    let height = image.height()/TILE_SIZE;

	  let mut y:u16 = 0;
	  let mut x:u16 = 0;
	  //add tiles to tilemap vector
	  for _i in 0..width*height {
        if x >(width-1) as u16{
    	  x = 0;
    	  y += 1
        }
        let mut temp: Image = image.clone();
        //crops the source image to the current tile that we want to source
        temp.crop(Rectangle::new( (x*TILE_SIZE as u16) as f32,(y*TILE_SIZE as u16) as f32,TILE_SIZE as f32, TILE_SIZE  as f32));
        //adds this to the tilemap vector
        tileset.push(rl.load_texture_from_image(&thread,&temp).unwrap());
        x += 1;
	  }
	  return (image, tileset,rl,thread)
}

