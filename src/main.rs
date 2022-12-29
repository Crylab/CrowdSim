use three_d::*;
use rand::Rng;

const H_TO_H_COEFF: f64 = 0.5;
const H_TO_O_COEFF: f64 = 0.5;
const H_RAND_COEFF: f64 = 0.1;
const ATTRAC_COEFF: f64 = 0.1;

struct Human {
    position: (f64, f64),
    velocity: (f64, f64),
    acceleration: (f64, f64),
}

struct Object {
    position: (f64, f64),
}

impl Object {
    fn set_position(x: f64, y: f64) -> Object {
        Object { position: (x, y) }
    }
    fn set_position_rand() -> Object {
        let mut rng = rand::thread_rng();
        Object { position: (rand::thread_rng().gen_range(0.0..100.0),
                            rand::thread_rng().gen_range(0.0..100.0)) }
    }
    fn get_position(&self) -> (f64, f64) {
        self.position
    }
}

impl Human {
    fn set_position_rand() -> Human {
        let mut rng = rand::thread_rng();
        Human { position: (rand::thread_rng().gen_range(0.0..100.0),
                           rand::thread_rng().gen_range(0.0..100.0)),
                velocity: (0.0, 0.0),
                acceleration: (0.0, 0.0) }
    }
    fn set_position(x: f64, y: f64) -> Human {
        Human { position: (x, y),
                velocity: (0.0, 0.0),
                acceleration: (0.0, 0.0) }
    }
    fn get_position(&self) -> (f64, f64) {
        self.position
    }
    fn get_distance(&self, other: (f64, f64)) -> f64 {
        let x = self.position.0 - other.0;
        let y = self.position.1 - other.1;
        (x.powi(2) + y.powi(2)).sqrt()
    }
    fn human_to_human(&self, other: (f64, f64)) -> (f64, f64) {
        let x = (other.0 - self.get_position().0)*H_TO_H_COEFF;
        let y = (other.1 - self.get_position().1)*H_TO_H_COEFF;
        (x, y)
    }
    fn human_to_object(&self, other: (f64, f64)) -> (f64, f64) {
        let x = H_TO_O_COEFF/(other.0 - self.get_position().0).powi(2);
        let y = H_TO_O_COEFF/(other.1 - self.get_position().1).powi(2);
        (x, y)
    }
    fn fluctuation(&self) -> (f64, f64) {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(-H_RAND_COEFF..H_RAND_COEFF);
        let y = rng.gen_range(-H_RAND_COEFF..H_RAND_COEFF);
        (x, y)
    }
    fn attraction(&self, other: (f64, f64)) -> (f64, f64) {
        let x = (other.0 - self.get_position().0)*ATTRAC_COEFF;
        let y = (other.1 - self.get_position().1)*ATTRAC_COEFF;
        (x, y)
    }
}

struct Scene {
    humans: Vec<Human>,
    police: Vec<Human>,
    obstacles: Vec<Object>,
    attractors: Vec<Object>,
}

pub fn main() {
    let window = Window::new(WindowSettings {
        title: "Shapes 2D!".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
        .unwrap();
    let context = window.gl();

    let mut rectangle = Gm::new(
        Rectangle::new(&context, vec2(200.0, 200.0), degrees(45.0), 100.0, 200.0),
        ColorMaterial {
            color: Color::RED,
            ..Default::default()
        },
    );
    let mut circle = Gm::new(
        Circle::new(&context, vec2(500.0, 500.0), 200.0),
        ColorMaterial {
            color: Color::BLUE,
            ..Default::default()
        },
    );
    let mut line = Gm::new(
        Line::new(
            &context,
            vec2(0.0, 0.0),
            vec2(
                window.viewport().width as f32,
                window.viewport().height as f32,
            ),
            5.0,
        ),
        ColorMaterial {
            color: Color::GREEN,
            ..Default::default()
        },
    );

    window.render_loop(move |frame_input: FrameInput| {
        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
            .render(
                &camera2d(frame_input.viewport),
                line.into_iter().chain(&rectangle),
                &[],
            );
        frame_input.render(&camera2d(frame_input.viewport),
                           circle,
                           &[],);
        FrameOutput::default()
    });
}
