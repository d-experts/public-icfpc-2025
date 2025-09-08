use std::collections::{HashMap, HashSet};

use serde::Serialize;

use crate::{
    _PROBLEMS,
    client::{ApiClient, Connection, Door, Map},
    utils::get_ith_label,
};

const MAX_SIGNATURE_LEN: usize = 12;

#[derive(Debug, Clone, Serialize)]
pub struct Room {
    /// この部屋のラベル 2bit
    pub label: usize,

    /// この部屋から、key で移動したときの result
    /// 同じ key で異なる result になっていたらアウト
    pub sign: HashMap<String, String>,

    /// この部屋から、door で移動したときの部屋
    pub doors: HashMap<usize, usize>,

    /// この部屋に移動してきた部屋と、そのときに通った扉
    /// 返報性原理のためのチェック用
    pub from_room_doors: HashSet<(usize, usize)>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Problem {
    pub N: usize,
    pub query: String,
    pub result: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct State {
    pub room_history: Vec<usize>,
    pub rooms: Vec<Room>,
}

pub fn ganba_dfs_solver() {
    let client = ApiClient::new();
    let problem = &_PROBLEMS[2];
    let problem_name = problem.name;
    let _select_result = client.select(problem_name);
    let v = problem.N;
    let query = problem.query;

    let result = client.explore(&vec![query.to_string()]).unwrap().results[0].clone();
    let result = result
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join("");

    let problem = Problem {
        N: v,
        query: query.to_string(),
        result: result.clone(),
    };

    let state = solve(&problem);

    let table = convert_to_table(&state);
    for (i, row) in table.iter().enumerate() {
        println!("room {i}: {row:?}");
        println!("  from_room_doors: {:?}", state.rooms[i].from_room_doors);
    }

    let state = fill_blank_in_dfs(&problem, state.clone());

    let rooms = state.rooms.clone();
    println!("The number of rooms: {}", rooms.len());

    let table = convert_to_table(&state);
    for (i, row) in table.iter().enumerate() {
        println!("room {i}: {row:?}");
    }

    let map = state.to_map();
    let guess_result = client.guess(map.rooms, map.starting_room, map.connections);
    println!("guess_result: {guess_result:?}");
}

pub fn solve(problem: &Problem) -> State {
    let rooms = vec![get_room_info(&problem, 0)];
    let room_history = vec![0];

    let state = dfs(
        &problem,
        State {
            rooms,
            room_history,
        },
        0,
    )
    .unwrap();

    let state = fill_blank_in_dfs(problem, state.clone());

    state
}

fn convert_to_table(state: &State) -> Vec<Vec<Option<usize>>> {
    let mut table = vec![vec![None; 6]; state.rooms.len()];
    for (i, room) in state.rooms.iter().enumerate() {
        for (door, room_id) in room.doors.iter() {
            table[i][*door] = Some(*room_id);
        }
    }
    table
}

fn get_room_info(problem: &Problem, idx: usize) -> Room {
    Room {
        label: get_ith_label(&problem.result, idx),
        sign: get_signatures(&problem, idx),
        doors: HashMap::new(),
        from_room_doors: HashSet::new(),
    }
}

fn get_signatures(problem: &Problem, idx: usize) -> HashMap<String, String> {
    let mut signatures = HashMap::new();
    for i in 0..MAX_SIGNATURE_LEN {
        if idx + i >= problem.query.len() {
            break;
        }
        let q = problem.query[idx..idx + i].to_string();
        let r = problem.result[idx..idx + i + 1].to_string();
        signatures.insert(q, r);
    }
    signatures
}

pub fn fill_blank_in_dfs(problem: &Problem, state: State) -> State {
    let mut new_state = state.clone();
    for i in 0..problem.N {
        if new_state.rooms[i].doors.is_empty() {
            new_state.rooms[i].doors = HashMap::new();
        }
    }

    let mut blanks = vec![];
    for room_id in 0..problem.N {
        for d in 0..6 {
            if !new_state.rooms[room_id].doors.contains_key(&d) {
                blanks.push((room_id, d));
            }
        }
    }

    println!("blanks: {blanks:?}");

    let result = fill_dfs(problem, new_state, &blanks, 0);

    result.expect("No solution found")
}

pub fn fill_dfs(
    problem: &Problem,
    state: State,
    blanks: &Vec<(usize, usize)>,
    idx: usize,
) -> Option<State> {
    if idx == blanks.len() {
        return Some(state);
    }

    let (room_id, door_id) = blanks[idx];

    // 部屋 room_id の door_id を通ったら target_room_id に移動するようにしてもいいか確認
    for (target_room_id, room) in state.rooms.iter().enumerate() {
        let mut new_state = state.clone();
        new_state.rooms[room_id]
            .doors
            .insert(door_id, target_room_id);
        new_state.rooms[target_room_id]
            .from_room_doors
            .insert((room_id, door_id));

        if !new_state.rooms[room_id].is_valid(problem) {
            println!(" - Invalid: idx: {idx}, room_id: {room_id}, door_id: {door_id}");
            continue;
        }

        if !new_state.rooms[target_room_id].is_valid(problem) {
            println!(" - Invalid: idx: {idx}, room_id: {target_room_id}, door_id: {door_id}");
            continue;
        }

        println!(
            " - Valid: idx: {idx}, room_id: {room_id}, door_id: {door_id}, target_room_id: {target_room_id}"
        );

        if let Some(result) = fill_dfs(problem, new_state, blanks, idx + 1) {
            return Some(result);
        }
    }

    None
}

pub fn dfs(problem: &Problem, state: State, idx: usize) -> Option<State> {
    println!("idx: {idx}");
    if idx + 1 == problem.query.len() {
        if state.rooms.len() == problem.N {
            return Some(state);
        }
        return None;
    }

    // DFSでは、次の部屋をどうするかを考えて、現在の部屋もそれに伴って変更する
    let current_room_id = state.room_history[state.room_history.len() - 1];
    let next_room = get_room_info(&problem, idx + 1);

    let door_id = get_ith_label(&problem.query, idx);

    // 既存の部屋とまとめられるかチェック
    for (room_id, room) in state.rooms.iter().enumerate() {
        let merged_room = room.clone().merge(&next_room, problem);
        if merged_room.is_none() {
            continue;
        }
        let mut merged_room = merged_room.unwrap();
        let mut new_state = state.clone();

        new_state.rooms[room_id] = merged_room;

        new_state.rooms[room_id]
            .from_room_doors
            .insert((current_room_id, door_id));
        new_state.rooms[current_room_id]
            .doors
            .insert(door_id, room_id);

        if !new_state.rooms[room_id].is_valid(problem) {
            continue;
        }
        if !new_state.rooms[current_room_id].is_valid(problem) {
            continue;
        }

        new_state.room_history.push(room_id);

        if let Some(result) = dfs(problem, new_state, idx + 1) {
            return Some(result);
        }
    }

    if state.rooms.len() == problem.N {
        return None;
    }

    // 新しい部屋を作る
    let mut new_state = state.clone();
    let mut next_room_with_from = next_room.clone();
    next_room_with_from
        .from_room_doors
        .insert((current_room_id, door_id));
    new_state.rooms.push(next_room_with_from);
    new_state.room_history.push(new_state.rooms.len() - 1);

    let mut current_room = state.rooms[current_room_id].clone();
    current_room
        .doors
        .insert(door_id, new_state.rooms.len() - 1);
    new_state.rooms[current_room_id] = current_room;

    if let Some(result) = dfs(problem, new_state, idx + 1) {
        return Some(result);
    }

    None
}

impl Room {
    fn merge_doors(&self, other: &Room, problem: &Problem) -> Option<HashMap<usize, usize>> {
        let mut merged_doors = HashMap::new();

        for (door, room) in self.doors.iter() {
            merged_doors.insert(*door, *room);
        }
        for (door, room) in other.doors.iter() {
            if let Some(merged_room) = merged_doors.get(door) {
                if *merged_room != *room {
                    return None;
                }
            } else {
                merged_doors.insert(*door, *room);
            }
        }

        Some(merged_doors)
    }

    fn merge_from_room_doors(&self, other: &Room) -> HashSet<(usize, usize)> {
        let mut merged_from_room_doors = self.from_room_doors.clone();
        for (room, door) in &other.from_room_doors {
            merged_from_room_doors.insert((*room, *door));
        }
        merged_from_room_doors
    }

    fn merge_signatures(&self, other: &Room) -> Option<HashMap<String, String>> {
        let mut merged_signatures = self.sign.clone();
        for (q, r) in &other.sign {
            if let Some(r2) = merged_signatures.get(q) {
                if *r != *r2 {
                    return None;
                }
            } else {
                merged_signatures.insert(q.clone(), r.clone());
            }
        }
        Some(merged_signatures)
    }

    fn merge(&self, other: &Room, problem: &Problem) -> Option<Room> {
        if self.label != other.label {
            return None;
        }

        let mut merged_doors = self.merge_doors(other, problem)?;
        let merged_from_room_doors = self.merge_from_room_doors(other);
        let merged_signatures = self.merge_signatures(other)?;

        if merged_from_room_doors.len() > 6 {
            return None;
        }

        Some(Room {
            label: self.label,
            sign: merged_signatures,
            doors: merged_doors,
            from_room_doors: merged_from_room_doors,
        })
    }

    fn rooms_are_mergeable(&self, other: &Room, problem: &Problem) -> bool {
        if !self.compare_signatures(&other.sign) {
            return false;
        }

        let mut count_outgoing = [0; 6];
        let mut count_incoming = [0; 6];

        let merged_from_room_doors = self.merge_from_room_doors(other);
        for (source_room, _door) in &merged_from_room_doors {
            count_incoming[*source_room] += 1;
        }

        if merged_from_room_doors.len() > problem.N {
            return false;
        }
        let merged_doors = self.merge_doors(other, problem);
        if merged_doors.is_none() {
            return false;
        }

        for (_door, target_room) in merged_doors.clone().unwrap() {
            count_outgoing[target_room] += 1;
        }

        // 返報性原理のチェック
        let vacant_doors = 6 - merged_doors.unwrap().len();
        for i in 0..6 {
            if count_incoming[i] > count_outgoing[i] + vacant_doors {
                return false;
            }
        }

        true
    }

    fn is_valid(&self, problem: &Problem) -> bool {
        let mut count_outgoing = vec![0; problem.N]; // この部屋から各部屋への出口の数
        let mut count_incoming = vec![0; problem.N]; // 各部屋からこの部屋への入口の数

        for (door, target_room) in self.doors.iter() {
            count_outgoing[*target_room] += 1;
        }
        for (source_room, door) in self.from_room_doors.iter() {
            count_incoming[*source_room] += 1;
        }

        let vacant_doors = 6 - self.doors.len(); // 6つの扉のうち、まだ決まっていない扉の数
        for i in 0..problem.N {
            // 返報性原理：部屋iからこの部屋への入口の数 <= この部屋から部屋iへの出口の数 + 空き扉
            if count_incoming[i] > count_outgoing[i] + vacant_doors {
                let ci = count_incoming[i];
                let co = count_outgoing[i];
                return false;
            }
        }
        true
    }

    fn compare_signatures(&self, signs2: &HashMap<String, String>) -> bool {
        for (q, r) in self.sign.iter() {
            if let Some(r2) = signs2.get(q) {
                if r != r2 {
                    return false;
                }
            }
        }
        for (q, r) in signs2.iter() {
            if let Some(r2) = self.sign.get(q) {
                if r != r2 {
                    return false;
                }
            }
        }
        return true;
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Answer {
    pub table: Vec<Vec<Option<usize>>>,
}

impl Answer {
    pub fn new(problem: &Problem, state: &State) -> Self {
        let mut table = vec![vec![None; 6]; problem.N];
        for (i, room) in state.rooms.iter().enumerate() {
            for (door, room_id) in room.doors.iter() {
                table[i][*door] = Some(*room_id);
            }
        }

        Self { table }
    }
}

impl State {
    fn to_map(&self) -> Map {
        let rooms = self.rooms.iter().map(|room| room.label).collect::<Vec<_>>();
        let starting_room = self.room_history[0];

        let connections = self.matrix_to_connections();

        let mut map = Map {
            rooms,
            starting_room,
            connections,
        };

        map
    }

    fn matrix_to_connections(&self) -> Vec<Connection> {
        let mut result = vec![];

        let mut doors = vec![vec![vec![]; self.rooms.len()]; self.rooms.len()];

        for (i, room) in self.rooms.iter().enumerate() {
            for (door, room_id) in room.doors.iter() {
                doors[i][*room_id].push(*door);
            }
        }

        for i in 0..self.rooms.len() {
            for j in i..self.rooms.len() {
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
        result
    }
}
