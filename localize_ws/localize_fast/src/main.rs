use std::fs::File;
use std::io::{Read, Write};
use three_d::*;
use rand::Rng;
use std::time::Instant;

const H_TO_H_COEFF: f64 = 2219.0;
const H_TO_O_COEFF: f64 = 2219.0;
const H_TO_A_COEFF: f64 = 0.0;
const H_TO_O_THRESHOLD: f64 = 0.6;
const H_TO_H_THRESHOLD: f64 = 0.6;
const H_TO_A_THRESHOLD: f64 = 10.0;
const H_RAND_COEFF: f64 = 10.0;
const H_RAND_PERIOD: usize = 3;
const ATTRAC_COEFF: f64 = 0.1;
const HUMAN_WEIGHT: f64 = 62.0;//62.0
const HUMAN_VISCOS: f64 = 50.0;//0.05

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
    observ: Observation,
}

struct Object {
    position: (f64, f64),
    id: usize,
}

struct Observation {
    position: (f64, f64),
    radius: f64,
    id: usize,
    expired: bool,
}

impl Observation {
    fn new(position: (f64, f64), radius: f64, id: usize) -> Observation {
        Observation {
            position,
            radius,
            id,
            expired: false,
        }
    }
    fn empty() -> Observation {
        Observation {
            position: (0.0, 0.0),
            radius: 0.0,
            id: 0,
            expired: true,
        }
    }
    fn clone(&self) -> Observation {
        Observation {
            position: self.position,
            radius: self.radius,
            id: self.id,
            expired: self.expired,
        }
    }

    fn is_valid(&self) -> bool {
        !self.expired
    }
    fn is_exact(&self) -> bool {
        return if self.radius < 0.001 { true } else { false };
    }
    fn overlays(&self, other: &Observation) -> Observation {
        if self.id != other.id {
            println!("Error: Observations do not have the same id");
            return Observation::new((0.0, 0.0), 0.0, 0);
        }
        let (x1, y1) = self.position;
        let (x2, y2) = other.position;
        let a = ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt();
        let p = (a + self.radius + other.radius) / 2.0;
        let h = (2.0 / a) * ((p * (p - a) * (p - self.radius) * (p - other.radius)).sqrt());
        if a>(self.radius+other.radius) {
            return Observation::empty();
        }
        if a<=((self.radius-other.radius).abs()) {
            return if self.radius > other.radius {
                Observation::new(other.position, other.radius, other.id)
            } else {
                Observation::new(self.position, self.radius, self.id)
            }
        }
        let proportion = self.radius/other.radius;
        let x3 = ((x2*proportion)+x1)/(1.0+proportion);
        let y3 = ((y2*proportion)+y1)/(1.0+proportion);
        Observation::new((x3, y3), h, self.id)
    }
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
            id: 0,}
    }
    fn change_position(&mut self, x: f64, y: f64) {
        self.position = (x, y);
    }
    fn get_position(&self) -> (f64, f64) {
        self.position
    }
}

impl Human {
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
    fn localization_distance(&self) -> f64 {
        let p1 = self.observ.position;
        let p2 = self.position;
        ((p1.0-p2.0).powi(2)+(p1.1-p2.1).powi(2)).sqrt()
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
    let humans_app = 10;
    let discoverable_range = 1.5;
    let field_x = 10.0;
    let field_y = 10.0;
    let scale = 1.0;

    let mut humans = Vec::new();
    let mut obstacles = Vec::new();
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
            observ: Observation::empty(),});
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
            id: humans_num,});
    }
    /////////////////////////////////////////
    // Atractor's spawn
    /////////////////////////////////////////
    let attractor = (field_x*0.5, field_y*0.5);

    let mut time = 0.0;
    let dt = 0.1;
    let mut fluctuation_timer = 0;
    let mut measures = Vec::new();
    let mut cloud_observations = Vec::new();
    let cutoff = 200.0;
    loop {
        for i in 0..humans_num {
            /////////////////////////////////////////
            // Mechanical interactions
            /////////////////////////////////////////
            for j in 0..humans_num {
                if j == i { continue; }
                if humans[j].get_distance(humans[i].get_position()) < H_TO_H_THRESHOLD {
                    let other = humans[j].get_position();
                    humans[i].human_to_human(other);
                }
                if humans[i].get_distance(humans[j].get_position()) < discoverable_range && humans[j].discoverable && !humans[j].app {
                    if humans[i].app {
                        cloud_observations.push(Observation::new(humans[j].get_position(), discoverable_range, humans[j].id));
                    } else if humans[i].discoverable && humans[i].observ.is_valid() {
                        let local = humans[i].observ.clone();
                        cloud_observations.push(Observation::new(local.position, local.radius+discoverable_range, humans[j].id));
                    }
                }
            }
            for j in 0..obstacles.len() {
                if humans[i].get_distance(obstacles[j].get_position()) < H_TO_O_THRESHOLD {
                    humans[i].human_to_object(obstacles[j].get_position());
                }
                humans[i].human_to_object(obstacles[j].get_position());
            }
            if humans[i].get_distance(attractor) < H_TO_A_THRESHOLD {
                humans[i].human_to_attraction(attractor);
            }
            humans[i].kinematics(dt);
            if fluctuation_timer > H_RAND_PERIOD {
                humans[i].fluctuation();
            }
        }
        /////////////////////////////////////////
        // People's fluctuations
        /////////////////////////////////////////
        if fluctuation_timer > H_RAND_PERIOD {
            fluctuation_timer = 0;
        }
        fluctuation_timer += 1;
        /////////////////////////////////////////
        // Cloud computing
        /////////////////////////////////////////
        for i in humans_app..humans_num {
            humans[i].observ = Observation::empty();
        }
        for i in (0..cloud_observations.len()).rev() {
            if !cloud_observations[i].is_valid() {continue;}
            let id = cloud_observations[i].id;
            if humans[id].app {
                cloud_observations[i].expired = true;
                continue;
            }
            if humans[id].observ.is_valid() {
                let updated = humans[id].observ.overlays(&cloud_observations[i]);
                if updated.is_valid() {
                    humans[id].observ = updated.clone();
                } else {
                    cloud_observations[i].expired = true;
                }
            } else {
                humans[id].observ = cloud_observations[i].clone();
            }
        }
        for i in (0..cloud_observations.len()).rev() {
            if cloud_observations[i].expired {
                cloud_observations.remove(i);
            }
        }
        for i in humans_app..humans_num {
            let dist = humans[i].localization_distance();
            measures.push(dist);
        }
        if time>cutoff {
            let mut file = File::create("estimations.csv").unwrap();
            let mut average = 0.0;
            let mut variation = 0.0;
            for estimation in measures.iter() {
                file.write_all(format!("{}\n", estimation).as_bytes()).unwrap();
                average += estimation;
            }
            average /= measures.len() as f64;
            for estimation in measures.iter() {
                variation += (estimation - average).powi(2);
            }
            variation /= measures.len() as f64;
            println!("Average: {} Variation: {} Samples: {}", average, variation, measures.len());
            break
        }
        time += dt;
        println!("Time: {}", time);
    }
    println!("Done!");
}
