use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub struct Universe {
    slices: Vec<Vec<Vertex>>,
    order_four: Vec<VertexPos>,
}

#[derive(Debug)]
struct Vertex {
    neighbours_next: Vec<VertexPos>,
    neighbours_prev: Vec<VertexPos>,
}

#[derive(Copy, Clone, Debug)]
struct VertexPos {
    time: usize,
    space: usize,
}

impl Index<VertexPos> for Universe {
    type Output = Vertex;

    fn index(&self, pos: VertexPos) -> &Self::Output {
        &self.slices[pos.time][pos.space]
    }
}

impl IndexMut<VertexPos> for Universe {
    fn index_mut(&mut self, pos: VertexPos) -> &mut Self::Output {
        &mut self.slices[pos.time][pos.space]
    }
}

impl Universe {
    pub fn new(n_vertex: usize, max_time: usize) -> Self {
        assert!(
            n_vertex >= max_time,
            "given vertex number ({}) is too small, must be at least as big as the max time ({})",
            n_vertex,
            max_time
        );

        let mut universe = Universe {
            slices: vec![],
            order_four: vec![],
        };
        for t in 0..max_time {
            let pos_next = VertexPos::new((t + 1) % max_time, 0);
            let pos_prev = VertexPos::new((t + max_time - 1) % max_time, 0);
            let vertex = Vertex {
                neighbours_next: vec![pos_next, pos_next],
                neighbours_prev: vec![pos_prev, pos_prev],
            };
            let slice = vec![vertex];
            universe.slices.push(slice);
        }

        let n_free = n_vertex - max_time;
        let pos_next = VertexPos::new(1, 0);
        let pos_prev = VertexPos::new(max_time - 1, 0);
        for s in 0..n_free {
            let vertex = Vertex {
                neighbours_next: vec![pos_next],
                neighbours_prev: vec![pos_prev],
            };
            universe.slices[0].push(vertex);

            let pos = VertexPos::new(0, s + 1);
            universe.order_four.push(pos);
            universe[pos_next].neighbours_prev.push(pos);
            universe[pos_prev].neighbours_next.push(pos);
        }
        universe
    }
}

impl VertexPos {
    fn new(time: usize, space: usize) -> Self {
        VertexPos { time, space }
    }
}
