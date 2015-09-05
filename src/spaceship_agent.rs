use actor::ActorView;
use messages::PlayerInstructions;
use rand;
use rand::Rng;

static PI : f32 = 3.14159265359;

enum Activity {
    Player(ActorView),
    Nothing
}

pub fn set_instructions(actor: ActorView,
                        nearbys: Vec<ActorView>,
                        player_messages:&mut Vec<(i32, PlayerInstructions)>){


    // figure out priority

    // 1. look for player
    // 2. can go forward
    // 3. fire

    let mut priority = Activity::Nothing;

    for enemy in nearbys.into_iter() {
        if enemy.id == 1 {
            priority = Activity::Player(enemy);
            break;
        }


    }

    match priority{
        Activity::Player(enemy)   => attack_player(actor, enemy, player_messages),
        _               => random_behaviour(actor.id, player_messages)
    }
}

fn random_behaviour(id: i32, player_messages: &mut Vec<(i32, PlayerInstructions)>){
    let rand = rand::thread_rng().gen_range(0u32, 100);
    match rand {
        0...50  => {
            player_messages.push((id, PlayerInstructions::StopRotateRight));
            player_messages.push((id, PlayerInstructions::StopRotateLeft));
            player_messages.push((id, PlayerInstructions::BeginIncreaseThrottle));
        },
        51...70 => {
            player_messages.push((id, PlayerInstructions::BeginRotateLeft));
            player_messages.push((id, PlayerInstructions::StopRotateRight));
            player_messages.push((id, PlayerInstructions::StopIncreaseThrottle));
        },
        71...90 => {
            player_messages.push((id, PlayerInstructions::BeginRotateRight));
            player_messages.push((id, PlayerInstructions::StopRotateLeft));
            player_messages.push((id, PlayerInstructions::StopIncreaseThrottle));
        }
        _      => {
            player_messages.push((id, PlayerInstructions::StopRotateRight));
            player_messages.push((id, PlayerInstructions::StopRotateLeft));
            player_messages.push((id, PlayerInstructions::StopIncreaseThrottle));
        }
    }
}

fn attack_player(player: ActorView, enemy: ActorView, player_messages: &mut Vec<(i32, PlayerInstructions)>){

    let dx = enemy.x - player.x;
    let dy = enemy.y - player.y;
    let mut ideal_rotation = dx.atan2(dy) * 180.0 / PI;
    let mut player_rotation = player.rotation * 180.0 / PI;


    while ideal_rotation > 360.0 {
        ideal_rotation -= 360.0;
    }

    while ideal_rotation < -360.0 {
        ideal_rotation += 360.0;
    }

    while player_rotation > 360.0 {
        player_rotation -= 360.0;
    }

    while player_rotation < -360.0 {
        player_rotation += 360.0;
    }

    let d_rotation = ideal_rotation - player_rotation;

    if d_rotation < 20.0 && d_rotation > -20.0 {
        player_messages.push((player.id, PlayerInstructions::BeginIncreaseThrottle));
        player_messages.push((player.id, PlayerInstructions::Fire));
        player_messages.push((player.id, PlayerInstructions::StopRotateLeft));
        player_messages.push((player.id, PlayerInstructions::StopRotateRight));
    } else if d_rotation < 0.0 {
        player_messages.push((player.id, PlayerInstructions::BeginRotateLeft));
        player_messages.push((player.id, PlayerInstructions::StopRotateRight));
    } else {
        player_messages.push((player.id, PlayerInstructions::StopRotateLeft));
        player_messages.push((player.id, PlayerInstructions::BeginRotateRight));
    }
}

