use actor;
use spaceship;
use bullet;
use asteroid;
use kamikaze;
use explosion;
use token;
use spaceship_agent;
use messages::PlayerInstructions;
use messages::GameInstructions;
use rand;
use rand::Rng;
//use std::num::Float;
//use std::num::FloatMath;

#[derive(Clone, Debug, PartialEq)]
pub struct ActorManager{
    spaceships: Vec<spaceship::Spaceship>,
    bullets: Vec<bullet::Bullet>,
    asteroids: Vec<asteroid::Asteroid>,
    kamikaze: Vec<kamikaze::Kamikaze>,
    explosions: Vec<explosion::Explosion>,
    tokens: Vec<token::Token>,
    count:i32,
    px: f32,
    py: f32
}

impl ActorManager {
    pub fn new() -> ActorManager {
        ActorManager {
            spaceships : vec!(),
            bullets: vec!(),
            asteroids: vec!(),
            kamikaze: vec!(),
            explosions: vec!(),
            tokens: vec!(),
            count: 1,
            px: 0.0,
            py: 0.0
        }
    }

    pub fn get(&self) -> Vec<actor::ActorView> {
        let mut all_views = ActorManager::get_views(&self.spaceships.clone());
        all_views.extend(ActorManager::get_views(&self.bullets.clone()));
        all_views.extend(ActorManager::get_views(&self.asteroids.clone()));
        all_views.extend(ActorManager::get_views(&self.kamikaze.clone()));
        all_views.extend(ActorManager::get_views(&self.explosions.clone()));
        all_views.extend(ActorManager::get_views(&self.tokens.clone()));
        all_views
    }

    pub fn get_collectables(&self) -> Vec<actor::ActorView> {
        ActorManager::get_views(&self.tokens.clone())
    }

    pub fn update(&mut self, messages:Vec<(i32, PlayerInstructions)>, output_messages:&mut Vec<(GameInstructions, actor::ActorView)>){
        let mut player_messages = messages;//messages.clone();

        for ref actor in self.get().iter(){
            if actor.id == 1 {
                self.px = actor.x;
                self.py = actor.y;
                break;
            }
        }


        for ship in ActorManager::get_views(&self.spaceships.clone()).iter(){
            if ship.id == 1 {
                // forget about the player
                continue;
            }
            let nearest = self.get_nearest(ship);
            spaceship_agent::set_instructions(ship.clone(), nearest, &mut player_messages);
        }

        self.spaceships = ActorManager::update_actor_list(self.px, self.py, &mut self.spaceships, &player_messages, output_messages);
        self.bullets    = ActorManager::update_actor_list(self.px, self.py, &mut self.bullets, &player_messages, output_messages);
        self.asteroids  = ActorManager::update_actor_list(self.px, self.py, &mut self.asteroids, &player_messages, output_messages);
        self.kamikaze  = ActorManager::update_actor_list(self.px, self.py, &mut self.kamikaze, &player_messages, output_messages);
        self.explosions  = ActorManager::update_actor_list(self.px, self.py, &mut self.explosions, &player_messages, output_messages);
        self.tokens  = ActorManager::update_actor_list(self.px, self.py, &mut self.tokens, &player_messages, output_messages);
    }

    pub fn process_messages(&mut self, output_messages: &Vec<(GameInstructions, actor::ActorView)>){

        for &(ref msg, ref v) in output_messages.iter(){
            //println!("{} : {}", msg, v);
            match msg{
                &GameInstructions::Fire          => self.add_bullet(v.id, v.x as i32, v.y as i32, v.rotation * 180.0 / 3.14159265359),
                &GameInstructions::Explode       => self.add_explosion(v.x as i32, v.y as i32, (v.width + v.height) as i32 / 2, v.rotation),
                &GameInstructions::Trail         => self.add_explosion(v.x as i32 - v.height as i32 * 2 * v.rotation.sin() as i32, v.y as i32 - v.height as i32 * 2 * v.rotation.cos() as i32, 10, v.rotation),
                &GameInstructions::NewAsteroid  => self.split_asteroid(v),
                &GameInstructions::Collect       => {
                    if v.id == 1{
                        self.new_token();
                    }
                }
            }
        }

    }

    pub fn new_player(&mut self){
        let mut p = spaceship::Spaceship::new(1, 0, 0, 0.0);
        p.set_color(vec!(0.7, 0.7, 0.77));
        self.spaceships.push(p);
    }

    pub fn new_token(&mut self){
        self.count += 1;
        let id = self.count;
        let x = rand::thread_rng().gen_range(-10000i32, 10000);
        let y = rand::thread_rng().gen_range(-10000i32, 10000);
        self.tokens = vec!(token::Token::new(id, x, y));
    }

    pub fn restart(&mut self){
        self.spaceships = vec!();
        self.bullets = vec!();
        self.asteroids = vec!();
        self.kamikaze = vec!();
        self.explosions = vec!();
        self.new_player();
        self.new_token();
    }

    pub fn new_spaceship(&mut self, x: i32, y:i32){
        self.count += 1;
        let id = self.count;
        let r = rand::thread_rng().gen_range(0.0f32, 360.0);
        let ship = spaceship::Spaceship::new(id, x, y, r);
        self.spaceships.push(ship);
    }

    pub fn new_asteroid(&mut self, x: i32, y:i32){
        self.count += 1;
        let id = self.count;
        let ast = asteroid::Asteroid::new(id, x, y);
        self.asteroids.push(ast);
    }

    fn split_asteroid(&mut self, original: &actor::ActorView){
        self.count += 1;
        let id = self.count;
        let d = original.width / 2.0;
        let ast = asteroid::Asteroid::new_with_d(id, original.x as i32, original.y as i32, d, 0);
        self.asteroids.push(ast);
        self.count += 1;
        let id2 = self.count;
        let ast = asteroid::Asteroid::new_with_d(id2, original.x as i32, original.y as i32, d, id);
        self.asteroids.push(ast);
    }

    pub fn new_kamikaze(&mut self, x: i32, y:i32, target:(f32, f32)){
        self.count += 1;
        let id = self.count;
        let kam = kamikaze::Kamikaze::new(id, x, y, target);
        self.kamikaze.push(kam);
    }

    fn add_bullet(&mut self, parent:i32, x:i32, y:i32, r:f32){
        self.count += 1;
        let id = self.count;
        let bullet = bullet::Bullet::new(id, parent, x, y, r);
        self.bullets.push(bullet);
    }

    fn add_explosion(&mut self, x:i32, y:i32, d:i32, r:f32){
        let expl = explosion::Explosion::new(x, y,  d, r);
        self.explosions.push(expl);
    }

    fn get_nearest(&self, actor: &actor::ActorView) -> Vec<actor::ActorView>{
        let mut nearest = vec!();

        for enemy in self.get().iter(){
            if enemy.id == actor.id {
                continue;
            }

            let max_distance = 2000.0;
            let dx = enemy.x - actor.x;
            let dy = enemy.y - actor.y;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance < max_distance{
                nearest.push(enemy.clone());
            }
        }
        nearest
    }

    fn update_actor_list<T: actor::Actor>(px:f32, py:f32, list:&mut Vec<T>,
                                messages:&Vec<(i32, PlayerInstructions)>,
                                output_messages:&mut Vec<(GameInstructions, actor::ActorView)>) -> Vec<T>{

        let threshold = 4000.0 * 4000.0;
        let mut ac:Vec<T> = vec!();

        for actor in list.iter_mut() {
            let a_pos = actor.get_view();
            if actor.get_id() != 1 && a_pos.collision_type != actor::CollisionType::Collect{
                let x_distance = a_pos.x - px;
                let y_distance = a_pos.y - py;
                let distance = x_distance * x_distance + y_distance * y_distance;
                if distance > threshold{
                    actor.kill();
                    continue;
                }
            }

            for &(id, ref message) in messages.iter(){
                if id == actor.get_id() {
                    actor.execute(message, output_messages);
                }
            }

            actor.update(output_messages);

            if actor.is_alive(){
                ac.push(actor.clone());
            }
        }

        ac

    }

    fn get_views<T: actor::Actor>(list: &Vec<T>) -> Vec<actor::ActorView>{
        let mut v: Vec< actor::ActorView > = vec!();
        for actor in list.iter() {
            v.push(actor.get_view());
        }
        v
    }
}
