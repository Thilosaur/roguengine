use specs::prelude::*;
use super::{a_star_search,a_star_search_cardinal};

use super::{Viewshed, Position, Map, Monster, Point, RunState, WantsToMelee};

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadExpect<'a, Point>,
                        WriteStorage<'a, Viewshed>,
                        WriteStorage<'a, Monster>,
                        WriteStorage<'a, Position>,
                        ReadExpect<'a, Entity>,
                        Entities<'a>,
                        WriteStorage<'a, WantsToMelee>,
                        ReadExpect<'a, RunState>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map,player_pos ,mut viewshed, mut monster, mut position,   player_entity, entities, mut wants_to_melee, runstate) = data;
        
        if *runstate != RunState::MonsterTurn { return; }

        for (entity,mut viewshed,mut monster,  mut pos) in (&entities, &mut viewshed, &mut monster,  &mut position).join() {
            let distance = Map::distance2d_pythagoras(pos.x, pos.y, player_pos.x, player_pos.y);
            if distance < 1.5 {
                wants_to_melee.insert(entity, WantsToMelee{ target: *player_entity }).expect("Unable to insert attack");
                return;
            }
            if viewshed.visible_tiles.contains(&(player_pos.x,player_pos.y)) {
                monster.seen_player = true;
                monster.known_player_location = (player_pos.x.clone(), player_pos.y.clone());
            }

            if monster.seen_player {
                if (pos.x, pos.y) == monster.known_player_location {
                    monster.seen_player = false;
                    return;
                }
                let path = a_star_search(
                map.xy_idx(pos.x, pos.y) as i32,
                map.xy_idx(monster.known_player_location.0, monster.known_player_location.1) as i32,
                &mut *map
                );
                

                if path.success && path.steps.len()>1{
                    pos.x = path.steps[1] as i32 % map.width;
                    pos.y = path.steps[1] as i32 / map.width;
                    viewshed.dirty = true;
                }
                
            }
        }
    }
}
