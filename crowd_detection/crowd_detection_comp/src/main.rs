use std::fs::File;
use std::io::{Read, Write};
use three_d::*;
use rand::Rng;
use std::time::Instant;

const H_TO_H_COEFF: f64 = 2219.0;
const H_TO_O_COEFF: f64 = 2219.0;
const H_TO_A_COEFF: f64 = 100.0;
const H_TO_O_THRESHOLD: f64 = 0.6;
const H_TO_H_THRESHOLD: f64 = 0.6;
const H_TO_A_THRESHOLD: f64 = 6.0;
const H_RAND_COEFF: f64 = 10.0;
const H_RAND_PERIOD: usize = 3;
const ATTRAC_COEFF: f64 = 0.1;
const HUMAN_WEIGHT: f64 = 62.0;//62.0
const HUMAN_VISCOS: f64 = 50.0;//0.05

const SCHMITT_TRIGGER: i8 = 3;
const NEIGHBORHOOD_THRESHOLD: usize = 1;
const ETALON_THRESHOLD: usize = 4;

//All distances in centimeters
const CM_TO_M: f64 = 100.0;

struct Human {
    position: (f64, f64),
    velocity: (f64, f64),
    acceleration: (f64, f64),
    desire: (f64, f64),
    id: usize,
    app: bool,
    discoverable: bool,
    schmitt_val: i8,
}

struct Object {
    position: (f64, f64),
    id: usize,
    color: Color,
}

fn obstacle_line(from: (f64, f64), to: (f64, f64)) -> Vec<(f64, f64)> {
    let mut vec = Vec::new();
    let step = 0.2;
    let dist = ((to.0 - from.0).powi(2) + (to.1 - from.1).powi(2)).sqrt();
    let steps = (dist / step).ceil() as i64;
    let dx = (to.0 - from.0) / steps as f64;
    let dy = (to.1 - from.1) / steps as f64;
    for i in 0..steps {
        vec.push((from.0 + dx * i as f64, from.1 + dy * i as f64));
    }
    vec
}

fn distance(a: (f64, f64), b: (f64, f64)) -> f64 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

impl Object {
    fn set_position(x: f64, y: f64) -> Object {
        Object { position: (x, y),
            id: 0,
            color: Color::BLUE }
    }
    fn change_position(&mut self, x: f64, y: f64) {
        self.position = (x, y);
    }
    fn set_position_rand() -> Object {
        let mut rng = rand::thread_rng();
        Object { position: (rand::thread_rng().gen_range(0.0..100.0),
                            rand::thread_rng().gen_range(0.0..100.0)),
            id: 0,
            color: Color::BLUE,
        }
    }
    fn get_position(&self) -> (f64, f64) {
        self.position
    }
}

impl Human {
    fn human() -> Human {
        let mut rng = rand::thread_rng();
        Human { position: (rand::thread_rng().gen_range(0.0..100.0),
                           rand::thread_rng().gen_range(0.0..100.0)),
            velocity: (0.0, 0.0),
            acceleration: (0.0, 0.0),
            id: 0,
            desire: (0.0, 0.0),
            app: false,
            discoverable: false,
            schmitt_val: 0,}
    }
    fn set_position(x: f64, y: f64) -> Human {
        Human { position: (x, y),
            velocity: (0.0, 0.0),
            acceleration: (0.0, 0.0),
            id: 0,
            desire: (0.0, 0.0),
            app: false,
            discoverable: false,
            schmitt_val: 0,}
    }
    fn set_crowded(&mut self) -> bool {
        self.schmitt_val += 1;
        return if self.schmitt_val > SCHMITT_TRIGGER {
            self.schmitt_val = SCHMITT_TRIGGER;
            true
        } else {
            false
        }
    }
    fn reset_crowded(&mut self) -> bool {
        self.schmitt_val -= 1;
        return if self.schmitt_val < 0 {
            self.schmitt_val = 0;
            true
        } else {
            false
        }
    }
    fn get_position(&self) -> (f64, f64) {
        self.position
    }
    fn get_distance(&self, other: (f64, f64)) -> f64 {
        let x = self.position.0 - other.0;
        let y = self.position.1 - other.1;
        (x.powi(2) + y.powi(2)).sqrt()
    }
    fn human_to_human(&mut self, other: (f64, f64)) {
        let dist = self.get_distance(other);
        if dist < H_TO_H_THRESHOLD {
            let x = ((-other.0 + self.get_position().0)/dist)*(H_TO_H_THRESHOLD-dist)*H_TO_H_COEFF;
            let y = ((-other.1 + self.get_position().1)/dist)*(H_TO_H_THRESHOLD-dist)*H_TO_H_COEFF;
            self.acceleration.0 +=x/HUMAN_WEIGHT;
            self.acceleration.1 +=y/HUMAN_WEIGHT;
        }
    }
    fn human_to_object(&mut self, other: (f64, f64)) {
        let dist = self.get_distance(other);
        if dist < H_TO_O_THRESHOLD {
            let x = ((-other.0 + self.get_position().0)/dist)*(H_TO_O_THRESHOLD-dist)*H_TO_O_COEFF;
            let y = ((-other.1 + self.get_position().1)/dist)*(H_TO_O_THRESHOLD-dist)*H_TO_O_COEFF;
            self.acceleration.0 +=x/HUMAN_WEIGHT;
            self.acceleration.1 +=y/HUMAN_WEIGHT;
        }
    }
    fn human_to_attraction(&mut self, other: (f64, f64)) {
        let dist = self.get_distance(other);
        if dist < H_TO_A_THRESHOLD {
            let x = ((other.0 - self.get_position().0)/dist)*H_TO_A_COEFF;
            let y = ((other.1 - self.get_position().1)/dist)*H_TO_A_COEFF;
            self.acceleration.0 +=x/HUMAN_WEIGHT;
            self.acceleration.1 +=y/HUMAN_WEIGHT;
        }
    }

    fn fluctuation(&mut self) {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(-H_RAND_COEFF..H_RAND_COEFF);
        let y = rng.gen_range(-H_RAND_COEFF..H_RAND_COEFF);
        self.desire.0 +=x/HUMAN_WEIGHT;
        self.desire.1 +=y/HUMAN_WEIGHT;
    }
    fn attraction(&self, other: (f64, f64)) -> (f64, f64) {
        let x = (other.0 - self.get_position().0)*ATTRAC_COEFF;
        let y = (other.1 - self.get_position().1)*ATTRAC_COEFF;
        (x, y)
    }
    fn reset_acceleration(&mut self) {
        self.acceleration.0 = 0.0;
        self.acceleration.1 = 0.0;
    }
    fn kinematics(&mut self, dt: f64) {
        self.velocity.0 += self.acceleration.0*dt;
        self.velocity.1 += self.acceleration.1*dt;
        self.position.0 += self.velocity.0*dt;
        self.position.1 += self.velocity.1*dt;
        self.acceleration.0 = -self.velocity.0*HUMAN_VISCOS*dt + self.desire.0;
        self.acceleration.1 = -self.velocity.1*HUMAN_VISCOS*dt + self.desire.1;
    }
}

pub fn main() {
    let humans_num = 100;
    let humans_app = 75;
    let field_x = 10.0;
    let field_y = 10.0;
    let scale = 1.0;

    let mut humans = Vec::new();
    let mut obstacles = Vec::new();
    let mut vec = Vec::new();

    let window = Window::new(WindowSettings {
        title: "Crowd Simulation".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
        .unwrap();
    let context = window.gl();


    /////////////////////////////////////////
    // Human spawn
    /////////////////////////////////////////
    for i in 0..humans_num {
        let x = rand::thread_rng().gen_range(0.25..field_x-0.25);
        let y = rand::thread_rng().gen_range(0.25..field_y-0.25);
        let app = if i < humans_app { true } else { false };
        humans.push(Human { position: (x,y),
            velocity: (0.0, 0.0),
            acceleration: (0.0, 0.0),
            id: i,
            desire: (0.0, 0.0),
            app,
            discoverable: true,
            schmitt_val: 0,});
        vec.push(Gm::new(
            Circle::new(&context, vec2((x*CM_TO_M) as f32, (y*CM_TO_M) as f32), 25.0),
            ColorMaterial { color: Color::BLACK, ..Default::default() }, ));

    }
    /////////////////////////////////////////
    // Object spawn
    /////////////////////////////////////////
    let line1 = obstacle_line((0.0, 0.0), (0.0, field_y));
    let line2 = obstacle_line((field_x, 0.0), (0.0, 0.0));
    let line3 = obstacle_line((field_x, field_y), (field_x, 0.0));
    let line4 = obstacle_line((0.0, field_y), (field_x, field_y));
    let mut lines = Vec::new();
    let line_len = line1.len();
    lines.extend(line1);
    lines.extend(line2);
    lines.extend(line3);
    lines.extend(line4);
    for point in lines.iter() {
        obstacles.push(Object { position: *point,
            id: humans_num,
            color: Color::BLUE,});
        vec.push(Gm::new(
            Circle::new(&context, vec2((point.0 * CM_TO_M) as f32, (point.1 * CM_TO_M) as f32), 25.0),
            ColorMaterial { color: Color::BLUE, ..Default::default() }, ));
    }
    /////////////////////////////////////////
    // Atractor's spawn
    /////////////////////////////////////////
    vec.push(Gm::new(
        Circle::new(&context, vec2((field_x*CM_TO_M*0.5) as f32, (field_y*CM_TO_M*0.5) as f32), 25.0),
        ColorMaterial { color: Color::GREEN, ..Default::default() }, ));
    let attractor = (field_x*0.5, field_y*0.5);

    let mut time = 0.0;
    let dt = 0.1;
    let mut fluctuation_timer = 0;
    let cutoff = 200.0;
    window.render_loop(move |frame_input: FrameInput| unsafe {
        let now = Instant::now();
        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
            .render(
                &camera2d(frame_input.viewport),
                vec.iter(),
                &[],
            );
        for i in 0..humans_num {
            /////////////////////////////////////////
            // Mechanical interactions
            /////////////////////////////////////////
            let mut counter = 0;
            for j in 0..humans_num {
                if j == i { continue; }
                if humans[j].get_distance(humans[i].get_position()) < H_TO_H_THRESHOLD {
                    let other = humans[j].get_position();
                    humans[i].human_to_human(other);
                    counter += 1;
                }
            }
            if counter>4 {
                vec[i].material.color = Color::RED;
            }
            else {
                vec[i].material.color = Color::BLACK;
            }
            for j in 0..obstacles.len() {
                if humans[i].get_distance(obstacles[j].get_position()) < H_TO_O_THRESHOLD {
                    humans[i].human_to_object(obstacles[j].get_position());
                }
            }
            if humans[i].get_distance(attractor) < H_TO_A_THRESHOLD {
                humans[i].human_to_attraction(attractor);
            }
            humans[i].kinematics(dt);
            if fluctuation_timer > H_RAND_PERIOD {
                humans[i].fluctuation();
            }
            vec[humans[i].id].set_center(vec2((humans[i].position.0 * CM_TO_M) as f32,
                                              (humans[i].position.1 * CM_TO_M) as f32));
        }
        /////////////////////////////////////////
        // People's fluctuations
        /////////////////////////////////////////
        if fluctuation_timer > H_RAND_PERIOD {
            fluctuation_timer = 0;
        }
        fluctuation_timer += 1;
        /////////////////////////////////////////
        // End of the simulation
        /////////////////////////////////////////
        //if time>cutoff {
        //    let mut file = File::create("estimations.csv").unwrap();
        //    let mut average = 0.0;
        //    let mut variation = 0.0;
        //    for estimation in estimations.iter() {
        //        file.write_all(format!("{}\n", estimation).as_bytes()).unwrap();
        //        average += estimation;
        //    }
        //    average /= estimations.len() as f64;
        //    for estimation in estimations.iter() {
        //        variation += (estimation - average).powi(2);
        //    }
        //    variation /= estimations.len() as f64;
        //    println!("Average: {} Variation: {} Samples: {}", average, variation, estimations.len());
        //    println!("Alpha: {} Beta: {} Base: {}", alpha_errors, beta_errors, base_errors);
        //    std::process::exit(0);
        //}
        /////////////////////////////////////////
        // Performance metrics
        /////////////////////////////////////////
        let elapsed = now.elapsed();
        let fps = 1.0 / elapsed.as_secs_f64();
        let sim_fps = 1.0 / dt;
        time += dt;
        println!("FPS: {:.2?} SimTime: {:.2?}", fps, time);
        FrameOutput::default()
    });
}
