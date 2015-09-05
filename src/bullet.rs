use actor::Actor;
use actor::ActorView;
use actor;
use messages::PlayerInstructions;
use messages::GameInstructions;

static PI : f32 = 3.14159265359;

#[derive(Debug, Clone, PartialEq)]
pub struct Bullet{
    id: i32,
    x: f32,
    y: f32,
    acc_x: f32,
    acc_y: f32,
    rotation: f32,
    shape: Vec<f32>,
    is_alive:bool,
    parent: i32,
    color: Vec<f32>
}


impl Bullet{
    pub fn new(id: i32, parent: i32, x: i32, y: i32, rotation: f32) -> Bullet {
        let shape = vec!(
            0.0,  0.005,
            0.005, -0.005,
            -0.005, -0.005,
        );

        let color = vec!(0.2, 0.8, 0.2);

        let acc = 100.0;
        let (dirx, diry) = Bullet::get_rotate_vec(rotation);
        let acc_x = acc * dirx;
        let acc_y = acc * diry;

        Bullet{
            id: id, parent: parent, x: x as f32, y: y as f32,
            rotation: rotation, acc_x: acc_x, acc_y: acc_y,
            shape: shape,
            is_alive: true,
            color: color
        }
    }

    fn get_rotate_vec(rotation:f32) -> (f32, f32){
        let r = (rotation * PI) / 180.0;
        (r.sin(), r.cos())
    }
}


impl Actor for Bullet{

    fn update(&mut self, _:&mut Vec<(GameInstructions, ActorView)>){
        self.y += self.acc_y;
        self.x += self.acc_x;

    }

    fn get_view(&self) -> ActorView {
        ActorView {
            id: self.id,
            parent: self.parent,
            x: self.x,
            y: self.y,
            width: 10.0,
            height: 10.0,
            rotation: (self.rotation * PI) / 180.0,
            shape: self.shape.clone(),
            color: self.color.clone(),
            collision_type: actor::CollisionType::Collide,
            show_secondary: false,
            secondary_shape: None,
            secondary_color: None,
            meter: 0.0
        }
    }

    fn execute(&mut self, message: &PlayerInstructions, _:&mut Vec<(GameInstructions, ActorView)>){
        match message {
            &PlayerInstructions::Collide   => self.is_alive = false,
            _                           => ()
        };
    }

    fn kill(&mut self){
        self.is_alive = false;
    }

    fn get_id(&self) -> i32{
        self.id
    }

    fn is_alive(&self) -> bool{
        self.is_alive
    }

}
