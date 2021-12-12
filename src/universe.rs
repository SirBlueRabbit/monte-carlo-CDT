use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct Universe {
    triangles: Vec<Triangle>,
    order_four: HashSet<usize>, // keeps a list of order 4 vertices, labelled by the top-left triangle
}

#[derive(Clone, Debug)]
struct Triangle {
    orientation: Orientation,
    time: usize,
    left: usize,
    right: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Orientation {
    Up,
    Down,
}

impl Universe {
    pub fn new(timespan: usize, triangle_count: usize) -> Self {
        assert!(
            triangle_count % (2 * timespan) == 0,
            "triangle count must be an integer multiple of 2 times the timespan"
        );

        let length = triangle_count / timespan;
        let mut triangles = Vec::with_capacity(triangle_count);
        for t in 0..timespan {
            for i in 0..length {
                let (orientation, time) = match i % 2 {
                    0 => (
                        Orientation::Up,
                        (((t + timespan - 1) % timespan) * length + i + 1),
                    ),
                    1 => (
                        Orientation::Down,
                        (((t + timespan + 1) % timespan) * length + i - 1),
                    ),
                    _ => panic!(
                        "input was likely invalid: timespan = {}, triangle_count = {}",
                        timespan, triangle_count
                    ),
                };
                let left = t * length + ((i + length - 1) % length);
                let right = t * length + ((i + length + 1) % length);
                triangles.push(Triangle {
                    orientation,
                    time,
                    left,
                    right,
                })
            }
        }
        let order_four = HashSet::new();
        Universe {
            triangles,
            order_four,
        }
    }

    pub fn mcmc_step(&mut self, move_ratio: f32) {
        if self.order_four.is_empty() || (fastrand::f32() > move_ratio) {
            let left = self.sample_uniform();
            // only flip when possible, do nothing otherwise
            // this is to ensure detailed balance
            if self.is_flippable(left) {
                self.triangle_flip(left);
            }
        } else {
            let shard_up = self.sample_shard();
            let dest_up = self.sample_dest(shard_up);
            self.shard_move(shard_up, dest_up);
        }
    }

    fn sample_uniform(&self) -> usize {
        fastrand::usize(..self.triangles.len())
    }

    fn is_flippable(&self, left: usize) -> bool {
        let right = self.triangles[left].right;
        self.triangles[left].orientation != self.triangles[right].orientation
    }

    fn sample_dest(&self, shard: usize) -> usize {
        loop {
            let index = fastrand::usize(..self.triangles.len());
            let dest = match self.triangles[index].orientation {
                Orientation::Up => index,
                Orientation::Down => self.triangles[index].time,
            };
            if dest != shard {
                return dest;
            }
        }
    }

    fn sample_shard(&self) -> usize {
        let index = fastrand::usize(..self.order_four.len());
        self.triangles[*self.order_four.iter().nth(index).unwrap()].right
    }

    fn triangle_flip(&mut self, left: usize) {
        // identify the relevant triangles
        let right = self.triangles[left].right;
        let left_nbr = self.triangles[left].time;
        let right_nbr = self.triangles[right].time;

        // flip the orientations
        self.swap_orientation(left, right);

        // reassign neighbours
        self.triangles[left_nbr].time = right;
        self.triangles[right_nbr].time = left;
        self.triangles[left].time = right_nbr;
        self.triangles[right].time = left_nbr;

        // update order_four, depending on how the flip was performed
        match self.triangles[left].orientation {
            Orientation::Up => {
                // original orientiation: down-up
                self.add_if_order_four(left_nbr);
                self.add_if_order_four(self.triangles[left].left);
                self.order_four.remove(&right);
                self.order_four.remove(&self.triangles[left_nbr].left);
            }
            Orientation::Down => {
                // original orientiation: up-down
                self.add_if_order_four(right);
                self.add_if_order_four(self.triangles[right_nbr].left);
                self.order_four.remove(&right_nbr);
                self.order_four.remove(&self.triangles[left].left);
            }
        }
    }

    fn swap_orientation(&mut self, left: usize, right: usize) {
        let left_orientation = self.triangles[left].orientation;
        self.triangles[left].orientation = self.triangles[right].orientation;
        self.triangles[right].orientation = left_orientation;
    }

    fn shard_move(&mut self, shard_up: usize, dest_up: usize) {
        // If move is to original position, do nothing
        let shard_nbr_left_up = self.triangles[shard_up].left;
        if dest_up == shard_nbr_left_up {
            return;
        }

        // identify the down triangles of shard and destination
        let shard_down = self.triangles[shard_up].time;
        let dest_down = self.triangles[dest_up].time;

        // identify the shard's neighbours
        let shard_nbr_right_up = self.triangles[shard_up].right;
        let shard_nbr_left_down = self.triangles[shard_down].left;
        let shard_nbr_right_down = self.triangles[shard_down].right;

        // reassign neighbours around original location (close the gap left behind by the shard)
        self.triangles[shard_nbr_left_up].right = shard_nbr_right_up;
        self.triangles[shard_nbr_right_up].left = shard_nbr_left_up;
        self.triangles[shard_nbr_left_down].right = shard_nbr_right_down;
        self.triangles[shard_nbr_right_down].left = shard_nbr_left_down;

        // identify the neighbours near the destination
        let dest_nbr_up = self.triangles[dest_up].right;
        let dest_nbr_down = self.triangles[dest_down].right;

        // reassign neighbours around new shard location (insert the shard in its new location)
        self.triangles[dest_up].right = shard_up;
        self.triangles[dest_nbr_up].left = shard_up;
        self.triangles[dest_down].right = shard_down;
        self.triangles[dest_nbr_down].left = shard_down;

        // update the neighbours of the shard itself
        self.triangles[shard_up].right = dest_nbr_up;
        self.triangles[shard_up].left = dest_up;
        self.triangles[shard_down].right = dest_nbr_down;
        self.triangles[shard_down].left = dest_down;

        // update order_four
        let dest_order4 = !self.order_four.insert(dest_up); // Add dest_up as order 4, and check if it already was
        let shard_order4 = if dest_order4 {
            // Add shard_up as order 4 if dest_up already was or remove if not, and check if itself already was
            !self.order_four.insert(shard_up)
        } else {
            self.order_four.remove(&shard_up)
        };
        if !shard_order4 {
            // Remove shard_nbr_left_up if shard_up was not already order 4
            self.order_four.remove(&shard_nbr_left_up);
        }
    }

    fn add_if_order_four(&mut self, label: usize) {
        if self.is_order_four_at(label) {
            self.order_four.insert(label);
        }
    }

    fn is_order_four_at(&self, label: usize) -> bool {
        self.triangles[label].orientation == Orientation::Up
            && self.triangles[self.triangles[label].right].orientation == Orientation::Up
            && self.triangles[self.triangles[label].time].right
                == self.triangles[self.triangles[label].right].time
    }

    pub fn lengths(&self, origin: usize) -> Vec<usize> {
        // look at the lengths of the timeslices starting from an origin
        // do this by walking through each slice, and thereafter advancing
        // to the next slice until back to starting point

        let triangle_count = self.triangles.len();
        let mut lengths = Vec::with_capacity(triangle_count); // TODO: can't this length be much shorter at least /2

        let mut marker = origin;
        loop {
            lengths.push(0);
            let t = lengths.len() - 1;

            // find a down triangle and mark it as origin of the slice
            let mut slice_origin = marker;
            slice_origin = loop {
                match self.triangles[slice_origin].orientation {
                    Orientation::Down => break slice_origin,
                    Orientation::Up => slice_origin = self.triangles[slice_origin].right,
                }
            }; // TODO: This wastes a few walks for every slice just on finding the marker

            // walk through the slice and count the triangles
            // break loop if slice_origin is found
            // return if origin is found
            if self.triangles[slice_origin].orientation == Orientation::Up {
                lengths[t] += 1;
            }
            let mut slice_walker = self.triangles[slice_origin].right;
            'slice_walk: loop {
                if slice_walker == origin && t > 0 {
                    lengths.pop(); // a slice was counted double, remove it
                    return lengths;
                } else if slice_walker == slice_origin {
                    break 'slice_walk;
                } else {
                    if self.triangles[slice_walker].orientation == Orientation::Up {
                        lengths[t] += 1;
                    }
                    slice_walker = self.triangles[slice_walker].right;
                }
            }

            // move to the next slice
            marker = self.triangles[slice_origin].time;
        }
    }
}
