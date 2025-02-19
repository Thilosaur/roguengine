use raylib::prelude::*;
use specs::World;


pub struct EventLog {
    pub log: Vec<String>
}

impl EventLog {
    pub fn new() -> EventLog{
        EventLog{log : Vec::new()}
    }

    pub fn message(&mut self, string:String) {
        self.log.push(string);
    }
}

pub fn draw_log(ecs: &World, draw: &mut RaylibDrawHandle) {
    let log = ecs.fetch::<EventLog>();
    let mut temp = log.log.clone();
    for i in 0..4 {
        let item = temp.pop();
        match item {
            Some(message) => {
                draw.draw_text(&message, 0, i*10, 8, match i {0 => {Color::WHITE},_=>{Color::GRAY}});
            }
            None => {}
        }
    }
}
