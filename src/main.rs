use nalgebra::{Matrix3, Vector3};
use std::{f64::consts::PI, thread::sleep, time::Duration};

struct Screen {
    width: usize,
    height: usize,
    pixels: Vec<bool>,
    buffer: String,
}

impl Screen {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![false; width * height],
            buffer: String::with_capacity(width * height),
        }
    }

    fn set(&mut self, x: usize, y: usize, value: bool) {
        if x < self.width && y < self.height {
            let index = x + y * self.width;
            self.pixels[index] = value;
        }
    }

    fn clear(&mut self) {
        self.pixels.fill(false);
    }

    fn project_3d_point(
        &mut self,
        point: Vector3<f64>,
        camera_position: Vector3<f64>,
        display_surface_z: f64,
    ) -> Option<(usize, usize)> {
        let transformed_point = point - camera_position;

        if transformed_point.z > 0.0 {
            let projected_x = (display_surface_z / transformed_point.z) * transformed_point.x;
            let projected_y = (display_surface_z / transformed_point.z) * transformed_point.y;

            let screen_x = ((projected_x + 1.0) * 0.5 * (self.width as f64)) as usize;
            let screen_y = ((1.0 - (projected_y + 1.0) * 0.5) * (self.height as f64)) as usize;

            if screen_x < self.width && screen_y < self.height {
                return Some((screen_x, screen_y));
            }
        }
        None
    }

    fn draw_line(&mut self, start: (usize, usize), end: (usize, usize)) {
        let (x0, y0) = start;
        let (x1, y1) = end;

        let dx = (x1 as isize - x0 as isize).abs();
        let dy = (y1 as isize - y0 as isize).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = if dx > dy { dx } else { -dy } / 2;

        let mut x = x0 as isize;
        let mut y = y0 as isize;

        loop {
            if x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize {
                self.set(x as usize, y as usize, true);
            }
            if x == x1 as isize && y == y1 as isize {
                break;
            }
            let e2 = err;
            if e2 > -dx {
                err -= dy;
                x += sx;
            }
            if e2 < dy {
                err += dx;
                y += sy;
            }
        }
    }

    fn build(&mut self) {
        self.buffer.clear();
        for row in self.pixels.chunks(self.width) {
            for &pixel in row {
                self.buffer.push(if pixel { '.' } else { ' ' });
            }
            self.buffer.push('\n');
        }
    }

    fn render(&self) {
        clearscreen::clear().unwrap();
        println!("{}", self.buffer);
    }
}

fn rotate_point(point: Vector3<f64>, rotation: Vector3<f64>) -> Vector3<f64> {
    let rotation_x = Matrix3::new(
        1.0, 0.0, 0.0,
        0.0, f64::cos(rotation.x), -f64::sin(rotation.x),
        0.0, f64::sin(rotation.x), f64::cos(rotation.x),
    );

    let rotation_y = Matrix3::new(
        f64::cos(rotation.y), 0.0, f64::sin(rotation.y),
        0.0, 1.0, 0.0,
        -f64::sin(rotation.y), 0.0, f64::cos(rotation.y),
    );

    let rotation_z = Matrix3::new(
        f64::cos(rotation.z), -f64::sin(rotation.z), 0.0,
        f64::sin(rotation.z), f64::cos(rotation.z), 0.0,
        0.0, 0.0, 1.0,
    );

    let rotation_matrix = rotation_x * rotation_y * rotation_z;
    rotation_matrix * point
}

fn main() {
    let mut screen = Screen::new(160, 80);
    let camera_position = Vector3::new(0.0, 2.0, -5.0);
    let display_surface_z = 1.0;

    let cube_vertices = vec![
        Vector3::new(-1.0, -1.0, -1.0),
        Vector3::new(1.0, -1.0, -1.0),
        Vector3::new(1.0, 1.0, -1.0),
        Vector3::new(-1.0, 1.0, -1.0),
        Vector3::new(-1.0, -1.0, 1.0),
        Vector3::new(1.0, -1.0, 1.0),
        Vector3::new(1.0, 1.0, 1.0),
        Vector3::new(-1.0, 1.0, 1.0),
    ];

    let cube_edges = vec![
        (0, 1), (1, 2), (2, 3), (3, 0),
        (4, 5), (5, 6), (6, 7), (7, 4),
        (0, 4), (1, 5), (2, 6), (3, 7),
    ];

    let mut angle = 0.0;

    loop {
        screen.clear();

        let rotation = Vector3::new(angle, 0.0, angle);

        let rotated_points: Vec<_> = cube_vertices
            .iter()
            .map(|&point| rotate_point(point, rotation))
            .collect();

        let projected_points: Vec<_> = rotated_points
            .iter()
            .filter_map(|&point| screen.project_3d_point(point, camera_position, display_surface_z))
            .collect();

        for &(start, end) in &cube_edges {
            if let (Some(p0), Some(p1)) = (projected_points.get(start), projected_points.get(end)) {
                screen.draw_line(*p0, *p1);
            }
        }

        screen.build();
        screen.render();

        angle += 0.01;
        if angle >= 2.0 * PI {
            angle -= 2.0 * PI;
        }

        sleep(Duration::from_millis(10));
    }
}
