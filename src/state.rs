use super::*;


#[derive(PartialEq, Copy, Clone)]
pub enum RunState { AwaitingInput, PreRun, PlayerTurn, MonsterTurn }




pub struct State {
    pub ecs: World,
}

impl State {
    pub fn tick(&mut self, handle : &mut RaylibHandle, thread: RaylibThread, tileset : &Vec<Texture2D>) {


        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate
        }

        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, handle);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
        }
        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }
        damage_system::delete_the_dead(&mut self.ecs);


        
        let mut draw = handle.begin_drawing(&thread);

        
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        
        let map = self.ecs.fetch::<Map>();

        draw.clear_background(Color::BLACK);
        

        draw_map(&self.ecs, &mut draw, &tileset);

        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                draw.draw_texture_ex(&tileset[render.index as usize],Vector2::new((pos.x * TILE_SIZE) as f32 * SCALE ,(pos.y * TILE_SIZE) as f32 * SCALE),0.0,SCALE,Color::WHITE);
            }
        }
        draw_log(&self.ecs, &mut draw);
    }

    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        let mut mov = MonsterAI{};
        mov.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem{};
        mapindex.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem{};
        melee.run_now(&self.ecs);
        let mut damage = DamageSystem{};
        damage.run_now(&self.ecs);
        self.ecs.maintain();
    }
}
