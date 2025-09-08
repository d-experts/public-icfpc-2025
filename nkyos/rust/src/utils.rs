use rand::Rng;
use serde::Serialize;

use crate::client::{Connection, Door};

#[derive(Debug, Clone, Serialize)]
pub struct OutEdges {
    pub out_edges: Vec<[i32; 6]>,
}

#[derive(Debug, Clone, Serialize)]
pub enum Action {
    Door(usize),
    Mark(usize),
}

pub fn all_actions() -> Vec<Action> {
    [all_doors(), all_marks()].concat()
}

pub fn all_doors() -> Vec<Action> {
    vec![
        Action::Door(0),
        Action::Door(1),
        Action::Door(2),
        Action::Door(3),
        Action::Door(4),
        Action::Door(5),
    ]
}

pub fn all_marks() -> Vec<Action> {
    vec![
        Action::Mark(0),
        Action::Mark(1),
        Action::Mark(2),
        Action::Mark(3),
    ]
}

pub fn get_ith_label(s: &String, i: usize) -> usize {
    let ch = s.chars().nth(i).unwrap();
    return ch as usize - '0' as usize;
}

pub fn create_random_route(v: usize) -> String {
    let mut route = String::new();
    let mut rng = rand::thread_rng();
    for _ in 0..v {
        route.push_str(&format!("{}", rng.gen_range(0..6)));
    }
    return route;
}

pub fn query_result_to_string(query_result: &Vec<usize>) -> String {
    query_result
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join("")
}

impl Action {
    pub fn to_string(&self) -> String {
        match self {
            Action::Door(d) => format!("{}", d),
            Action::Mark(m) => format!("[{}]", m),
        }
    }

    pub fn vec_to_str(actions: &[Action]) -> String {
        actions
            .iter()
            .map(|a| a.to_string())
            .collect::<Vec<String>>()
            .join("")
    }

    pub fn from_string(s: &String) -> Vec<Action> {
        s.chars()
            .map(|c| Action::Door(c.to_digit(10).unwrap() as usize))
            .collect()
    }
}

pub fn matrix_to_connections(matrix: &Vec<Vec<Option<usize>>>) -> Option<Vec<Connection>> {
    let mut result = vec![];
    let N = matrix.len();

    let mut doors = vec![vec![vec![]; N]; N];

    for i in 0..N {
        for k in 0..6 {
            if matrix[i][k].is_some() {
                doors[i][matrix[i][k].unwrap()].push(k);
            }
        }
    }

    for i in 0..N {
        for j in i..N {
            println!(
                "i: {i}, j: {j}, doors[i][j]: {:?}, doors[j][i]: {:?}",
                doors[i][j], doors[j][i]
            );
            if doors[i][j].len() != doors[j][i].len() {
                return None;
            }
            for k in 0..doors[i][j].len() {
                result.push(Connection {
                    from: Door {
                        room: i,
                        door: doors[i][j][k],
                    },
                    to: Door {
                        room: j,
                        door: doors[j][i][k],
                    },
                });
            }
        }
    }
    Some(result)
}
