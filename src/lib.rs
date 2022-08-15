use bio::alphabets::dna::complement;
use gfa::gfa::{Orientation, GFA};
use std::collections::{hash_map::Entry, VecDeque};

type Graph = GFA<Vec<u8>, ()>;
type Text = [u8];
// vertex id in graph
type V = usize;
// distance
type D = usize;
// queue
type Q = VecDeque<(State, D)>;
// distance storage
// TODO: Change to array?
type Map = rustc_hash::FxHashMap<State, D>;

#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
struct State {
    // Text index.
    i: usize,
    // The vertex id.
    v: V,
    // The position in the vertex, 0 <= j <= |v|.
    j: usize,
}

pub fn bfs(graph: &Graph, text: &Text, s: V) -> D {
    let mut queue = Q::default();
    let mut g = Map::default();

    // Duplicate vertices for the forward and reverse direction.
    let vertices: Vec<_> = graph
        .segments
        .iter()
        .flat_map(|s| {
            let mut rev = s.sequence.clone();
            // Make the reverse complement sequence.
            rev.reverse();
            for c in &mut rev {
                *c = complement(*c);
            }
            [s.sequence.clone(), rev]
        })
        .collect();
    let id: rustc_hash::FxHashMap<Vec<u8>, V> = graph
        .segments
        .iter()
        .enumerate()
        .map(|(i, vertex)| (vertex.name.clone(), i))
        .collect();
    let mut edges = vec![vec![]; 2 * graph.segments.len()];
    for link in &graph.links {
        let from =
            2 * id[&link.from_segment] + (link.from_orient == Orientation::Backward) as usize;
        let to = 2 * id[&link.to_segment] + (link.to_orient == Orientation::Backward) as usize;
        edges[from].push(to);
        let from =
            2 * id[&link.from_segment] + (link.from_orient == Orientation::Backward) as usize;
        let to = 2 * id[&link.to_segment] + (link.to_orient == Orientation::Backward) as usize;
        edges[to ^ 1].push(from ^ 1);
    }

    let extend = |st: &mut State| {
        // Greedy extend
        while st.i < text.len() && st.j < vertices[st.v].len() {
            if text[st.i] == vertices[st.v][st.j] {
                st.i += 1;
                st.j += 1;
            } else {
                break;
            }
        }
    };

    {
        let mut start = State { i: 0, v: s, j: 0 };
        extend(&mut start);
        g.insert(start, 0);
        queue.push_back((start, 0));
    }

    let mut last_d = 0;
    let mut maxi = 0;
    while let Some((st, qd)) = queue.pop_front() {
        let d = g[&st];
        assert!(d <= qd, "dist {d} queue has {qd}");
        if qd != d {
            continue;
        }
        if d > last_d {
            last_d = d;
        }

        if st.i > maxi {
            maxi = st.i;
        }

        if st.i == text.len() {
            return d;
        }

        let mut explore = |mut st: State, cost| {
            let d = d + cost;
            extend(&mut st);
            match g.entry(st) {
                Entry::Occupied(mut e) => {
                    if d < *e.get() {
                        e.insert(d);
                        if cost == 0 {
                            queue.push_front((st, d));
                        } else {
                            queue.push_back((st, d));
                        }
                    }
                }
                Entry::Vacant(e) => {
                    e.insert(d);
                    if cost == 0 {
                        queue.push_front((st, d));
                    } else {
                        queue.push_back((st, d));
                    }
                }
            }
        };

        // Match or substitution.
        if st.i < text.len() && st.j < vertices[st.v].len() {
            let is_match = text[st.i] == vertices[st.v][st.j];
            explore(
                State {
                    i: st.i + 1,
                    j: st.j + 1,
                    ..st
                },
                if is_match { 0 } else { 1 },
            );
            if is_match {
                continue;
            }
        }

        // Insertion
        if st.j < vertices[st.v].len() {
            explore(State { j: st.j + 1, ..st }, 1);
        }
        // Deletion
        if st.i < text.len() {
            explore(State { i: st.i + 1, ..st }, 1);
        }
        // Outgoing neighbours.
        if st.j == vertices[st.v].len() {
            for &v in &edges[st.v] {
                explore(State { i: st.i, j: 0, v }, 0);
            }
        }
    }
    unreachable!()
}
