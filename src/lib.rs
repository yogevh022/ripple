use glam::{IVec3, U8Vec3};

fn init_sorted_sphere_octant(radius: u32) -> Vec<U8Vec3> {
    if radius > u8::MAX as u32 {
        panic!("radius must be less than or equal to u8::MAX ({})", u8::MAX);
    }
    // visit points in a sphere, k = dist from 0
    let d2 = (radius * radius) as usize;
    let each = |f: &mut dyn FnMut(usize, U8Vec3)| {
        for x in 0..radius as usize {
            for y in 0..radius as usize {
                let xy = x * x + y * y;
                if xy > d2 {
                    break;
                }
                for z in 0..radius as usize {
                    let k = xy + z * z;
                    if k > d2 {
                        break;
                    }
                    f(k, U8Vec3::new(x as u8, y as u8, z as u8));
                }
            }
        }
    };
    let mut start = vec![0usize; d2 + 2];
    each(&mut |k, _| start[k + 1] += 1);
    for i in 1..start.len() {
        start[i] += start[i - 1];
    }
    let mut out = vec![U8Vec3::ZERO; start[d2 + 1]];
    each(&mut |k, p| {
        out[start[k]] = p;
        start[k] += 1;
    });
    out
}

fn for_each_octant<F: FnMut(IVec3)>(mut point: IVec3, mut f: F) {
    for _x in 0..2 {
        for _y in 0..2 {
            for _z in 0..2 {
                f(point);
                point.z = -point.z - 1;
            }
            point.y = -point.y - 1;
        }
        point.x = -point.x - 1;
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Ripple3DCursor {
    cursor: usize,
}

pub struct Ripple3D {
    points: Vec<U8Vec3>,
}

impl Ripple3D {
    pub fn new(radius: u32) -> Self {
        Ripple3D {
            points: init_sorted_sphere_octant(radius),
        }
    }

    pub fn step<F: FnMut(IVec3)>(&self, c: &mut Ripple3DCursor, n_points: usize, mut f: F) -> bool {
        let end = (c.cursor + n_points).min(self.points.len());
        for p in &self.points[c.cursor..end] {
            for_each_octant(p.as_ivec3(), &mut f);
        }
        c.cursor = end;
        end < self.points.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hint::black_box;
    use std::time::Instant;

    // empty benchmark. cargo test --release timing -- --nocapture
    #[test]
    fn timing() {
        let t = Instant::now();
        let r = Ripple3D::new(255);
        println!("setup: {:?}", t.elapsed());

        let bytes = r.points.len() * size_of::<U8Vec3>();
        println!("points: {} ({:.2} MB)", r.points.len(), bytes as f64 / (1024.0 * 1024.0));

        let count = 100;
        let t = Instant::now();
        for _ in 0..count {
            let mut c = Ripple3DCursor::default();
            while r.step(&mut c, 64, |p| { black_box(p); }) {}
        }
        println!("full pass: {:?}", t.elapsed() / count);
    }
}
