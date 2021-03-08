use serde_derive::{Serialize, Deserialize};

use crate::coord::{Coord};

#[derive(Clone, Debug, Copy, PartialEq, Serialize, Deserialize)]
pub struct Cuboid {
    pub start: Coord,
    pub end: Coord,
}

impl Cuboid {
    // unneeded allocation
    pub fn corners(&self) -> Vec<Coord> {
        let mut result = Vec::with_capacity(8);
        let minx = self.start.x.min(self.end.x);
        let maxx = self.start.x.max(self.end.x);
        let miny = self.start.y.min(self.end.y);
        let maxy = self.start.y.max(self.end.y);
        let minz = self.start.z.min(self.end.z);
        let maxz = self.start.z.max(self.end.z);

        result.push(Coord { x: minx, y: miny, z: minz });
        result.push(Coord { x: minx, y: miny, z: maxz });
        result.push(Coord { x: minx, y: maxy, z: minz });
        result.push(Coord { x: minx, y: maxy, z: maxz });
        result.push(Coord { x: maxx, y: miny, z: minz });
        result.push(Coord { x: maxx, y: miny, z: maxz });
        result.push(Coord { x: maxx, y: maxy, z: minz });
        result.push(Coord { x: maxx, y: maxy, z: maxz });

        result
    }

    pub fn avg_dist_to_point(&self, center: Coord) -> f32 {
        let corners = self.corners();
        let dists: Vec<f32> = corners.into_iter().map(|corn| {
            let dist = center.dist_to(&corn);
            dist
        }).collect();

        dists.iter().sum::<f32>() / dists.len() as f32
    }

    // pub fn sphere_intersection(&self, center: Coord, radius: f32) -> f32 {
    //     let corners = self.corners();
    //     let dists: Vec<f32> = corners.into_iter().map(|corn| {
    //         let dist = center.dist_to(&corn);
    //         radius - dist
    //     }).collect();

    //     dists.iter().sum::<f32>() / dists.len() as f32
    // }
}
