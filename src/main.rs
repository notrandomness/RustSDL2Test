extern crate sdl2;

use sdl2::{event::Event, rect::Point};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::time::SystemTime;
use rand::Rng;


/**
 * This is a simple demo of the performance of SDL2 in Rust.
 */


static SCREEN_WIDTH: u32 = 1920;
static SCREEN_HEIGHT: u32 = 1080;
static G: f64 = 0.000000011; // gravitational constant
static SUN_MASS: f64 = 32000000.0; // mass of the sun
static SPEED: f64 = 100.0;

// handle the annoying Rect i32
macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

struct Orb {
    x: f64,
    y: f64,
    r: i32,
    vx: f64,
    vy: f64,
    circle: Vec<(i32, i32)>,
}

fn rand_i32() -> i32 {
    let mut rng = rand::thread_rng();
    rng.gen()
}


pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let window = video_subsystem
        .window("rust-sdl2 performance demo", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().software().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();

    // Load a font
    let mut font = ttf_context.load_font("seguisym.ttf", 100)?;
    font.set_style(sdl2::ttf::FontStyle::BOLD);

    canvas.set_draw_color(Color::RGB(0, 0, 0)); //set color for clearing screen
    canvas.clear();
    canvas.present();

    //create initial Orb states
    let sun_pos = (SCREEN_WIDTH as f64 / 2.0, SCREEN_HEIGHT as f64 / 2.0);
    let mut orbs = Vec::new();
    for i in 0..5000 {
        let mut orb = Orb {
            x: (rand_i32() % SCREEN_WIDTH as i32) as f64 ,
            y: (rand_i32() % (SCREEN_HEIGHT * 2) as i32) as f64 - (SCREEN_HEIGHT as f64/2.0) ,
            r: 10,
            vx: 0.0,
            vy: 0.0,
            circle: Vec::new(),
        };
        let dx = orb.x - sun_pos.0;
        let dy = orb.y - sun_pos.1;
        let r = (dx * dx + dy * dy).sqrt();
        let V = (G * SUN_MASS / r).sqrt();
        let theta = dy.atan2(dx) + (3.141592 / 2.0);
        orb.vx = V * theta.cos();
        orb.vy = V * theta.sin();

        //Generate circle points
        orb.circle = get_circle(0 as i32, 0 as i32, orb.r);
        orbs.push(orb);
    }

    let mut start = SystemTime::now();
    let mut vector = Vec::new();
    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        //calculate average FPS
        let end = SystemTime::now();
        let since_the_last_frame = end
            .duration_since(start)
            .expect("Time went backwards");
        vector.insert(0, since_the_last_frame);
        if vector.len() > 20 {
            vector.pop();
        }
        let mut sum = 0;
        for i in &vector {
            sum += i.as_nanos();
        }
        let average_fps =  1_000_000_000 as f64/(sum as f64 / vector.len() as f64);
        start = SystemTime::now();

        canvas.set_draw_color(Color::RGB(0, 0, 0)); //set color for clearing screen
        canvas.clear(); //clear screen to black
        canvas.set_draw_color(Color::RGB(0, 255, 0)); //set color for drawing orbs


        //calculate new orb positions
        for orb in orbs.iter_mut() {
            let dx = orb.x - sun_pos.0;
            let dy = orb.y - sun_pos.1;
            let mut r = (dx * dx + dy * dy).sqrt();
            if r == 0.0 {
                r = 0.0000000001;
            }
            let g = ((G * SUN_MASS * SPEED as f64) / (r*r));
            let theta = dy.atan2(dx);
            orb.vx += -g * theta.cos();
            orb.vy += -g * theta.sin();
            orb.x += orb.vx * SPEED as f64;
            orb.y += orb.vy * SPEED as f64;

            //generate circle points with correct offset
            let mut points = Vec::new();
            for pixel in orb.circle.iter_mut() {
                points.push(Point::new((orb.x + pixel.0 as f64) as i32, (orb.y + pixel.1 as f64) as i32));
            }
            canvas.draw_points(points.as_slice()).unwrap(); //draw orb
        }


        canvas.set_draw_color(Color::RGB(255, 255, 255)); //set color for FPS
        // render a surface, and convert it to a texture bound to the canvas
        let surface = font
            .render(average_fps.floor().to_string().as_str())
            .blended(Color::RGBA(255, 0, 0, 255))
            .map_err(|e| e.to_string())?;
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;

        let target = rect!(0, 0, 80, 40); //set position of FPS counter

        canvas.copy(&texture, None, Some(target))?;
        canvas.present(); //draw to screen
        
    }

    Ok(())
}


fn get_circle(center_x: i32, center_y: i32, r: i32) -> Vec<(i32,i32)> {
    let mut x = r - 1;
    let mut y = 0;
    let mut tx = 1;
    let mut ty = 1;
    let mut error = tx - (r << 1); // shifting bits left by 1 effectively
                                   // doubles the value. == tx - diameter
    let mut points = Vec::new();
    
    while x >= y {
        points.push((center_x + x, center_y - y));
        points.push((center_x + x, center_y + y));
        points.push((center_x - x, center_y - y));
        points.push((center_x - x, center_y + y));
        points.push((center_x + y, center_y - x));
        points.push((center_x + y, center_y + x));
        points.push((center_x - y, center_y - x));
        points.push((center_x - y, center_y + x));

        if error <= 0 {
            y += 1;
            error += ty;
            ty += 2;
        }
        if error > 0 {
            x -= 1;
            tx += 2;
            error += tx - (r << 1);
        }
    }
    return points;
}