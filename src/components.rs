use raylib::color::Color;
use specs::prelude::*;
use specs_derive::*;



#[derive(Component)]
pub struct Renderable {
    pub index: u8,
    pub color: Color,
}

#[derive(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}


#[derive(Component, Debug)]
pub struct Player {
    pub health : i32,
}


#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles : Vec<(i32,i32)>,
    pub range : i32,
    pub dirty : bool
}

#[derive(Component)]
pub struct Monster {
    pub seen_player : bool,
    pub known_player_location: (i32, i32)
}

#[derive(Component, Debug)]
pub struct Name {
    pub name : String
}


#[derive(Component, Debug)]
pub struct BlocksTile {}


#[derive(Component, Debug)]
pub struct CombatStats {
    pub max_hp : i32,
    pub hp : i32,
    pub defense : i32,
    pub power : i32
}

#[derive(Component, Debug, Clone)]
pub struct WantsToMelee {
    pub target : Entity
}


#[derive(Component, Debug)]
pub struct SufferDamage {
    pub amount : Vec<i32>
}

impl SufferDamage {
    pub fn new_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: i32) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amount.push(amount);
        } else {
            let dmg = SufferDamage { amount : vec![amount] };
            store.insert(victim, dmg).expect("Unable to insert damage");
        }
    }
}
