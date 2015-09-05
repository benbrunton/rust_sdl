//#![feature(globs)]

extern crate gl;
extern crate glfw;
extern crate time;
extern crate rand;

use gl::types::*;
use glfw::Context;
use rand::Rng;
use std::mem;
use std::ptr;
use std::str;
//use std::num::Float;
//use std::num::FloatMath;
use std::iter::repeat;

mod actor;
mod actor_manager;
mod spaceship;
mod spaceship_agent;
mod bullet;
mod asteroid;
mod kamikaze;
mod explosion;
mod token;
mod game;
mod background;
mod messages;

// Shader sources
// vertex shader
static VS_SRC: &'static str =
   "#version 150\n\
    in vec4 shape;\n\
    uniform vec3 position;\n\
    uniform vec2 camera;\n\
    uniform float z;\n\
    void main() {\n\
        float x = shape[0];\n\
        float y = shape[1];\n\
        float x_pos = position[0];\n\
        float y_pos = position[1];\n\
        float angle = position[2];\n\
        float c_x   = camera[0];\n\
        float c_y   = camera[1];\n\
        float xx = (x * cos(angle) + y * sin(angle)) + x_pos - c_x;\n\
        float yy = (-x * sin(angle) + y * cos(angle)) + y_pos - c_y;\n\
        gl_Position = vec4(xx, yy, 0.0, z);\n\
    }";


// fragment shader
static FS_SRC: &'static str =
   "#version 150\n\
    out vec4 out_color;\n\
    uniform vec3 color;\n\
    void main() {\n\
       out_color = vec4(color, 0.8);\n\
    }";


fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    unsafe {
        let shader = gl::CreateShader(ty);

        // Attempt to compile the shader
        glfw::with_c_str(src, |ptr| gl::ShaderSource(shader, 1, &ptr, ptr::null()));
        gl::CompileShader(shader);

        // Get the compile status
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf:Vec<u8> = repeat(0u8).take(len as usize -1).collect();  // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            panic!("{:?}", str::from_utf8(&buf[..]));
        }
        shader
    }
}

fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);

        // Get the link status
        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len: GLint = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf:Vec<u8> = repeat(0u8).take(len as usize - 1).collect();     // subtract 1 to skip the trailing null character
            gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            panic!("{:?}", str::from_utf8(&buf[..]));
        }
        program
    }
}

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    // Choose a GL profile that is compatible with OS X 10.7+
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 2));
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    let (mut window, events) = glfw.create_window(800, 600, "rusteroids", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);

    // It is essential to make the context current before calling `gl::load_with`.
    window.make_current();

    // Load the OpenGL function pointers
//    gl::load_with(|s| glfw.get_proc_address(s));
    gl::load_with(|s| window.get_proc_address(s));

    // Create GLSL shaders
    let vs = compile_shader(VS_SRC, gl::VERTEX_SHADER);
    let fs = compile_shader(FS_SRC, gl::FRAGMENT_SHADER);
    let program = link_program(vs, fs);

    let mut vao = 0;
    let mut vbo = 0;
    let mut loc;
    let mut cam;
    let mut color;
    let mut z;

    unsafe {
        // Create Vertex Array Object
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        // Create a Vertex Buffer Object and copy the vertex data to it
        gl::GenBuffers(1, &mut vbo);

        // Generate a buffer for the indices
 // GLuint elementbuffer;
 // glGenBuffers(1, &elementbuffer);
 // glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, elementbuffer);


        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        // gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, vbo);

        // Use shader program
        gl::UseProgram(program);
        glfw::with_c_str("out_color", |ptr| gl::BindFragDataLocation(program, 0, ptr));

        // Specify the layout of the vertex data
        let pos_attr = glfw::with_c_str("shape",|ptr| gl::GetAttribLocation(program, ptr));
        gl::EnableVertexAttribArray(pos_attr as GLuint);
        gl::VertexAttribPointer(pos_attr as GLuint,     // must match the layout in the shader.
                                2,                      // size
                                gl::FLOAT,              // type
                                gl::FALSE as GLboolean, // normalized?
                                0,                      // stride
                                ptr::null());           // array buffer offset

        loc = glfw::with_c_str("position",|ptr| gl::GetUniformLocation(program, ptr));
        cam = glfw::with_c_str("camera",|ptr| gl::GetUniformLocation(program, ptr));
        color = glfw::with_c_str("color",|ptr| gl::GetUniformLocation(program, ptr));
        z = glfw::with_c_str("z",|ptr| gl::GetUniformLocation(program, ptr));
    }

    let output_on = false;
    let global_time = time::get_time();
    let mut t = time::get_time();
    let mut inner_t = time::get_time();
    let fr:i32 = 100000000 / 60;

    // camera position
    let mut cam_pos = (0.0, 0.0);
    let mut game = game::Game::new();

    let mut reset_countdown:usize = 3;
    let mut actors = actor_manager::ActorManager::new();
    actors.restart();

    let mut background = background::Background::new();
    background.generate(cam_pos.clone());

    while !window.should_close() {

        // Poll events
        glfw.poll_events();

        let mut messages = vec!();

        for event in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event, &mut messages);
        }

        let t2 = time::get_time();


        //////////////////////////////////////
        //
        //     INNER LOOP
        //
        //////////////////////////////////////


        // switch to 1 frame a second
        //let fr = 1000000000;


        if t2.nsec - fr > t.nsec || t2.sec > t.sec {


            t = t2;


            calculate_collisions(&actors, &mut messages);


            let mut output_messages = vec!();
            actors.update(messages, &mut output_messages);

            cam_pos = get_camera(&actors, cam_pos.clone());

            draw_scene(&actors, loc, cam, color, z, cam_pos.clone(), &mut window, &background);

            actors.process_messages(&mut output_messages);
            game.process_messages(output_messages);

            generate_actors(&mut actors, cam_pos.clone(), game.max_players());
            background.cleanup(cam_pos.clone());
            background.offscreen_generate(cam_pos.clone());

            window.set_title(&format!("rusteroids - score [{}] - highscore [{}]", game.score, game.highscore)[..]);

            // every second
            let t3 = time::get_time();
            if t3.sec > inner_t.sec {
                inner_t = t3;

                if output_on {
                    println!("::  {}s  ::::::::::::::::::::::::::::::", t3.sec - global_time.sec);
                    for actor in actors.get().iter(){
                        if actor.id == 1 {
                            println!("> x  :: {}", actor.x);
                            println!("> y  :: {}", actor.y);
                        }

                        if actor.collision_type == actor::CollisionType::Collect {
                            println!("-- collect --");
                            println!("> x  :: {}", actor.x);
                            println!("> y  :: {}", actor.y);
                        }
                    }

                    println!(":: SCORE : {}", game.score);
                    println!(":: HIGHSCORE : {}", game.highscore);

                    println!(":::::::::::::::::::::::::::::::::::::::\n");
                }

                if check_restart(&actors){

                    if reset_countdown > 0 {
                        reset_countdown -= 1;
                    } else {
                        restart(&mut actors, &mut game);
                        reset_countdown = 3;
                    }
                }


            }

        }



        //////////////////////////////////////
        //
        // END OF INNER LOOP
        //
        ///////////////////////////////////////


        //window.set_should_close(true);

    }

    unsafe {
        // Cleanup
        gl::DeleteProgram(program);
        gl::DeleteShader(fs);
        gl::DeleteShader(vs);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteVertexArrays(1, &vao);
    }
}

fn generate_actors(actors: &mut actor_manager::ActorManager, (cx, cy): (f32, f32), max_actors: usize){

    let min_x = cx as i32 - 4000;
    let max_x = cx as i32 + 4000;
    let min_y = cy as i32 - 4000;
    let max_y = cy as i32 + 4000;
    let min_distance = 2600 * 2600; // square instead of sqrt on distance

    while actors.get().len() < max_actors {
        let x = rand::thread_rng().gen_range(min_x, max_x);
        let y = rand::thread_rng().gen_range(min_y, max_y);

        let x_dis = x - cx as i32;
        let y_dis = y - cy as i32;
        let distance = x_dis * x_dis + y_dis * y_dis;

        if distance > min_distance {
            let rand = rand::thread_rng().gen_range(0u32, 100);
            match rand {
                0...75  => actors.new_asteroid(x, y),
                76...82 => actors.new_spaceship(x, y),
                83...85 => actors.new_kamikaze(x, y, (cx, cy)),
                _      => ()
            }
        }
    }
}

fn check_restart(actors: &actor_manager::ActorManager) -> bool{
    let mut player_exists = false;
    for actor in actors.get().iter(){
        if actor.id == 1 {
            player_exists = true;
            break;
        }
    }
    !player_exists
}

fn restart(actors: &mut actor_manager::ActorManager, game: &mut game::Game){
    game.restart();
    actors.restart();
}


fn calculate_collisions(actor_manager: &actor_manager::ActorManager, messages: &mut Vec<(i32, messages::PlayerInstructions)>){

    let actors = actor_manager.get();

    for a1 in actors.iter(){

        // todo - if we count the top iter(), we can slice this one
        //      - to only match remaining actors
        //      - would then need to push a2.id messages on collision

        let actors2 = actor_manager.get();

        for a2 in actors2.iter(){

            if     a1.id    == 0
                || a2.id    == 0
                || a1.id    == a2.id
                || a1.id    == a2.parent
                || a2.id    == a1.parent
                || a1.collision_type == actor::CollisionType::Ignore
                || a2.collision_type == actor::CollisionType::Ignore {
                continue;
            }

            if (a1.x - a2.x).abs() as usize > 1000 || (a1.y - a2.y).abs() as usize > 1000 {
                continue;
            }

            if a1.x + a1.width > a2.x - a2.width && a1.x - a1.width < a2.x + a2.width
              && a1.y + a1.height > a2.y - a2.height && a1.y - a1.height < a2.y + a2.height {

                // within bounding box - pixel perfect that shit
                // ...

                match a2.collision_type{
                    actor::CollisionType::Collide => messages.push((a1.id,
                        messages::PlayerInstructions::Collide)),
                    actor::CollisionType::Collect => messages.push((a1.id,
                    messages::PlayerInstructions::Collect)),
                    _              => ()
                }

            }
        }
    }

}

fn handle_window_event(window: &mut glfw::Window, (_/*time*/, event): (f64, glfw::WindowEvent), messages : &mut Vec<(i32, messages::PlayerInstructions)>) {
    match event {
        // glfw::PosEvent(x, y)                => window.set_title(format!("Time: {}, Window pos: ({}, {})", time, x, y).as_slice()),
        // glfw::SizeEvent(w, h)               => window.set_title(format!("Time: {}, Window size: ({}, {})", time, w, h).as_slice()),
        // glfw::CloseEvent                    => println!("Time: {}, Window close requested.", time),
        // glfw::RefreshEvent                  => println!("Time: {}, Window refresh callback triggered.", time),
        // glfw::FocusEvent(true)              => println!("Time: {}, Window focus gained.", time),
        // glfw::FocusEvent(false)             => println!("Time: {}, Window focus lost.", time),
        // glfw::IconifyEvent(true)            => println!("Time: {}, Window was minimised", time),
        // glfw::IconifyEvent(false)           => println!("Time: {}, Window was maximised.", time),
        // glfw::FramebufferSizeEvent(w, h)    => println!("Time: {}, Framebuffer size: ({}, {})", time, w, h),
        // glfw::CharEvent(character)          => println!("Time: {}, Character: {}", time, character),
        // glfw::MouseButtonEvent(btn, action, mods) => println!("Time: {}, Button: {}, Action: {}, Modifiers: [{}]", time, glfw::DebugAliases(btn), action, mods),
        // glfw::CursorPosEvent(xpos, ypos)    => window.set_title(format!("Time: {}, Cursor position: ({}, {})", time, xpos, ypos).as_slice()),
        // glfw::CursorEnterEvent(true)        => println!("Time: {}, Cursor entered window.", time),
        // glfw::CursorEnterEvent(false)       => println!("Time: {}, Cursor left window.", time),
        // glfw::ScrollEvent(x, y)             => window.set_title(format!("Time: {}, Scroll offset: ({}, {})", time, x, y).as_slice()),
        glfw::WindowEvent::Key(key, /* scancode */ _, action, /* mods */ _ ) => {
            // println!("Time: {}, Key: {}, ScanCode: {}, Action: {}, Modifiers: [{}]", time, key, scancode, action, mods);
            match (key, action) {
                (glfw::Key::Escape, glfw::Action::Press) => window.set_should_close(true),
                (glfw::Key::Up, glfw::Action::Press) => messages.push((1, messages::PlayerInstructions::BeginIncreaseThrottle)),
                (glfw::Key::Down, glfw::Action::Press) => messages.push((1, messages::PlayerInstructions::BeginDecreaseThrottle)),
                (glfw::Key::Up, glfw::Action::Release) => messages.push((1, messages::PlayerInstructions::StopIncreaseThrottle)),
                (glfw::Key::Down, glfw::Action::Release) => messages.push((1, messages::PlayerInstructions::StopDecreaseThrottle)),
                (glfw::Key::Right, glfw::Action::Press) => messages.push((1, messages::PlayerInstructions::BeginRotateRight)),
                (glfw::Key::Left, glfw::Action::Press) => messages.push((1, messages::PlayerInstructions::BeginRotateLeft)),
                (glfw::Key::Right, glfw::Action::Release) => messages.push((1, messages::PlayerInstructions::StopRotateRight)),
                (glfw::Key::Left, glfw::Action::Release) => messages.push((1, messages::PlayerInstructions::StopRotateLeft)),
                (glfw::Key::Space, glfw::Action::Release) => messages.push((1, messages::PlayerInstructions::Fire)),
                (glfw::Key::LeftShift, glfw::Action::Press) => messages.push((1, messages::PlayerInstructions::ShieldUp)),
                (glfw::Key::LeftShift, glfw::Action::Release) => messages.push((1, messages::PlayerInstructions::ShieldDown)),
                // (glfw::KeyR, glfw::Press) => {
                //     // Resize should cause the window to "refresh"
                //     let (window_width, window_height) = window.get_size();
                //     window.set_size(window_width + 1, window_height);
                //     window.set_size(window_width, window_height);
                // }
                _ => {}
            }
        },

        _ => ()
    }
}

fn get_camera(actor_manager:&actor_manager::ActorManager, (cx, cy):(f32,f32)) -> (f32, f32){
    let actors = actor_manager.get();
    for v in actors.iter() {
        if v.id == 1 {
            return (v.x, v.y);
        }
    }

    (cx, cy)
}

fn draw_scene(actor_manager:&actor_manager::ActorManager,
        loc:i32,
        cam:i32,
        color:i32,
        z: i32,
        (cx, cy):(f32, f32),
        window: &mut glfw::Window,
        background: &background::Background){

    let actors = actor_manager.get();
    let bg = background.get();

    unsafe {
        gl::ClearColor(0.1, 0.1, 0.2, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
    for st in bg.iter(){
        draw(&st.shape, loc, cam, color, z, st.x, st.y, 0.0, cx, cy, &st.color, 1.9);
    }

    let mut meter = 0.0;
    for v in actors.iter() {
        if v.id == 1 {
            meter = v.meter;
        }

        draw_actor(v, loc, cam, color, z, cx, cy);
    }

    let collectables = actor_manager.get_collectables();

    draw_hud(loc, cam, color, z, (cx, cy), collectables, meter);

    window.swap_buffers();
}

fn draw_actor(p: &actor::ActorView, loc:i32, cam:i32, color:i32, z:i32, cx: f32, cy: f32){

    draw(&p.shape, loc, cam, color, z, p.x, p.y, p.rotation, cx, cy, &p.color, 1.0);
    if p.show_secondary {
        match (p.secondary_shape.clone(), p.secondary_color.clone()) {
            (Some(shape), Some(second_color)) => draw(&shape, loc, cam, color, z, p.x, p.y, p.rotation, cx, cy, &second_color, 1.0),
            _                        => ()
        }

    }
}

fn draw_hud(loc:i32, cam:i32, color:i32, z:i32, (cx, cy) : (f32, f32), collectables : Vec<actor::ActorView>, meter: f32){
    let v = vec!(
        0.0, 0.0,
        0.04, -0.04,
        0.0, -0.02,

        0.0, -0.02,
        -0.04, -0.04,
        0.0, 0.0
    );

    let col = vec!(
        0.9, 0.9, 0.4
    );

    for token in collectables.iter(){
        let dx = token.x - cx;
        let dy = token.y - cy;
        let rotation = dx.atan2(dy);

        let player_distance = (dx * dx + dy * dy).sqrt() as i32;

        let dx = rotation.sin();
        let dy = rotation.cos();

        let mut distance = 1800;

        while distance > player_distance - 100 {
            distance -= 5;
        }

        let x = dx * (distance as f32);
        let y = dy * (distance as f32);

        draw(&v, loc, cam, color, z, x, y, rotation, 0.0, 0.0, &col, 1.0);
    }

    let m = meter/5.0;
    let h = 0.02;
    let v = vec!(
        0.0, 0.0,
        0.0, -h,
        m, 0.0,

        m, 0.0,
        m, -h,
        0.0, -h
    );

    let col = vec!(0.4, 0.6, 0.2);

    draw(&v, loc, cam, color, z, 1500.0, -1850.0, 0.0, 0.0, 0.0, &col, 1.0);

}

fn draw(v: &Vec<f32>, loc:i32, cam:i32, color:i32, z:i32, x:f32, y:f32, rotation:f32, cx:f32, cy:f32, col:&Vec<f32>, z_val:f32){
    unsafe{

        gl::BufferData(gl::ARRAY_BUFFER,
               (v.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
               mem::transmute(&v[0]),
               gl::DYNAMIC_DRAW); // STATIC | DYNAMIC | STREAM


        gl::Uniform3f(loc, x / 2000.0, y / 2000.0, rotation);
        gl::Uniform2f(cam, cx / 2000.0, cy / 2000.0);
        gl::Uniform3f(color, col[0], col[1], col[2]);
        gl::Uniform1f(z, z_val);

        // LINE_LOOP / TRIANGLES
        gl::DrawArrays(gl::TRIANGLES, 0, v.len() as i32 / 2);
    }
    // unsafe {
    //     gl::DrawElements(gl::TRIANGLES, v.len() as GLsizei,
    //                                gl::UNSIGNED_INT, ptr::null());
    // }
}
