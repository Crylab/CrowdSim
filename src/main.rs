use std::io::Read;
use three_d::*;
use rand::Rng;

const H_TO_H_COEFF: f64 = 0.4;
const H_TO_O_COEFF: f64 = 0.4;
const H_TO_O_THRESHOLD: f64 = 100.0;
const H_TO_H_THRESHOLD: f64 = 200.0;
const H_RAND_COEFF: f64 = 0.1;
const ATTRAC_COEFF: f64 = 0.1;
const HUMAN_WEIGHT: f64 = 62.0;//62.0

//All distances in centimeters

struct Human {
    position: (f64, f64),
    velocity: (f64, f64),
    acceleration: (f64, f64),
    visual_id: usize,
    color: Color,
}

struct Object {
    position: (f64, f64),
    visual_id: usize,
    color: Color,
}

fn obstacle_line(from: (f64, f64), to: (f64, f64)) -> Vec<(f64, f64)> {
    let mut vec = Vec::new();
    let step = 40.0;
    let dist = ((to.0 - from.0).powi(2) + (to.1 - from.1).powi(2)).sqrt();
    let steps = (dist / step).ceil() as i64;
    let dx = (to.0 - from.0) / steps as f64;
    let dy = (to.1 - from.1) / steps as f64;
    for i in 0..steps {
        vec.push((from.0 + dx * i as f64, from.1 + dy * i as f64));
    }
    vec
}

impl Object {
    fn set_position(x: f64, y: f64) -> Object {
        Object { position: (x, y),
                 visual_id: 0,
                 color: Color::BLUE }
    }
    fn set_position_rand() -> Object {
        let mut rng = rand::thread_rng();
        Object { position: (rand::thread_rng().gen_range(0.0..100.0),
                            rand::thread_rng().gen_range(0.0..100.0)),
                 visual_id: 0,
                    color: Color::BLUE,}
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
                visual_id: 0,
                color: Color::BLUE,}
    }
    fn set_position(x: f64, y: f64) -> Human {
        Human { position: (x, y),
                velocity: (0.0, 0.0),
                acceleration: (0.0, 0.0),
                visual_id: 0,
                color: Color::BLUE,}
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
    fn fluctuation(&mut self) {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(-H_RAND_COEFF..H_RAND_COEFF);
        let y = rng.gen_range(-H_RAND_COEFF..H_RAND_COEFF);
        self.acceleration.0 +=x/HUMAN_WEIGHT;
        self.acceleration.1 +=y/HUMAN_WEIGHT;
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
    }
}

pub fn main() {

    let humans_num = 10;
    let obstacles_num = 10;
    let field_x = 1000.0;
    let field_y = 1000.0;
    let scale = 1.0;

    let mut humans = Vec::new();
    //let mut police = Vec::new();
    let mut obstacles = Vec::new();
    //let mut attractors = Vec::new();

    let mut vec = Vec::new();

    let window = Window::new(WindowSettings {
        title: "Shapes 2D!".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
        .unwrap();
    let context = window.gl();


    /////////////////////////////////////////
    // Human spawn
    /////////////////////////////////////////
    for i in 0..humans_num {
        let x = rand::thread_rng().gen_range(0.0..field_x);
        let y = rand::thread_rng().gen_range(0.0..field_y);
        humans.push(Human { position: (x,y),
                                velocity: (0.0, 0.0),
                                acceleration: (0.0, 0.0),
                                visual_id: i,
                                color: Color::BLACK,});
        vec.push(Gm::new(
            Circle::new(&context, vec2(x as f32, y as f32), 25.0),
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
    lines.extend(line1);
    lines.extend(line2);
    lines.extend(line3);
    lines.extend(line4);

    for point in lines.iter() {
        obstacles.push(Object { position: *point,
                                visual_id: humans_num,
                                color: Color::BLUE,});
        vec.push(Gm::new(
            Circle::new(&context, vec2(point.0 as f32, point.1 as f32), 25.0),
            ColorMaterial { color: Color::BLUE, ..Default::default() }, ));
    }
    window.render_loop(move |frame_input: FrameInput| {
        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
            .render(
                &camera2d(frame_input.viewport),
                vec.iter(),
                &[],
            );
        for i in 0..humans_num {

            humans[i].reset_acceleration();
            for j in 0..humans_num {
                if humans[j].visual_id == humans[i].visual_id {continue;}
                if humans[j].get_distance(humans[i].get_position()) < H_TO_H_THRESHOLD {
                    let other = humans[j].get_position();
                    humans[i].human_to_human(other);
                }
            }
            for j in 0..obstacles.len() {
                if humans[i].get_distance(obstacles[j].get_position()) < H_TO_O_THRESHOLD {
                    humans[i].human_to_object(obstacles[j].get_position());
                }
                humans[i].human_to_object(obstacles[j].get_position());
            }
            humans[i].fluctuation();
            humans[i].kinematics(1.0);

            vec[humans[i].visual_id].set_center(vec2(humans[i].position.0 as f32,
                                                    humans[i].position.1 as f32));
        }
        FrameOutput::default()
    });
}
