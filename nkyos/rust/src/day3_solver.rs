use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};
use reqwest::header::CONTENT_SECURITY_POLICY_REPORT_ONLY;

use crate::{client::ApiClient, utils::Action, ProblemSetting, _PROBLEMS};
use rand::Rng;

const NUM_QUERY: usize = 1;

pub fn day3_solver() {
    let client = ApiClient::new();
    let problem = &_PROBLEMS[1];
    let problem_name = problem.name;
    let select_result = client.select(problem_name);

    let N = problem.N / problem.layers;
    let N_layer = problem.layers;

    let queries = vec![create_random_query(18 * N); NUM_QUERY];
    let query_results = get_query_results(queries);
    let state = solve(problem, query_results);
}

fn solve(problem: &ProblemSetting, query_resuls: Vec<QueryResult>) -> State {
    let num_query = query_resuls.len();
    let num_action = query_resuls[0].query.len();

    let all_N = problem.N;
    let N = problem.N / problem.layers;

    println!(
        "problem.N: {}, problem.layers: {}",
        problem.N, problem.layers
    );

    let mut state = State {
        N: N,
        N_layers: problem.layers,
        org_labels: vec![None; N],
        layers: vec![
            LayerInfo {
                labels: vec![None; N],
                layer_doors: vec![vec![None; 6]; N],
                vacant_door_num: vec![6; N],
            };
            problem.layers
        ],
        plane_doors: vec![vec![None; 6]; N],
        kashikari_count_plane: vec![vec![0; N]; N],
        kashikari_count: vec![vec![0; all_N]; all_N],
        vacant_door_num: vec![6; N],
        layer_assignments: vec![vec![0; num_action]; num_query],
        room_assignments: vec![vec![0; num_action]; num_query],
    };

    for i in 0..N {
        state.org_labels[i] = Some(i % 4);
        for l in 0..problem.layers {
            state.layers[l].labels[i] = Some(i % 4);
        }
    }

    let mut memo = HashSet::default();
    let result = dfs(&problem, &query_resuls, &mut state, 0, 0, &mut memo);

    println!("result: {result:?}");
    state.print();

    state
}

/// q_idx: いくつ目のクエリ
/// a_idx: いくつ目のアクション
fn dfs(
    problem: &ProblemSetting,
    query_results: &Vec<QueryResult>,
    state: &mut State,
    q_idx: usize,
    a_idx: usize,
    memo: &mut HashSet<State>,
) -> bool {
    // if memo.contains(state) {
    //     return false;
    // }
    // memo.insert(state.clone());

    if q_idx == query_results.len() {
        return true;
    }

    let query = &query_results[q_idx].query[a_idx];
    let prev_result = query_results[q_idx].result[a_idx];
    let result = query_results[q_idx].result[a_idx + 1];

    // クエリの最初は、(0, 0) にいる
    if a_idx == 0 {
        state.room_assignments[q_idx][a_idx] = 0;
        state.layer_assignments[q_idx][a_idx] = 0;
        state.set_label(Room(0, 0), Some(prev_result));
    }

    let current_room = state.room_assignments[q_idx][a_idx];
    let current_layer = state.layer_assignments[q_idx][a_idx];

    // println!("result: {:?}", query_results[q_idx].result);
    // println!(
    //     "q_idx: {q_idx}, a_idx: {a_idx}, current_room: {current_room}, current_layer: {current_layer}: {query:?} {result} {}",
    //     memo.len()
    // );

    if let Some(current_label) = state.layers[current_layer].labels[current_room] {
        if current_label != prev_result {
            return false;
        }
    } else {
        state.set_label(Room(current_room, current_layer), Some(result));
    }

    let (next_q_idx, next_a_idx) = next_idx(q_idx, a_idx, query_results);

    // マーク付け
    match query {
        Action::Mark(label) => {
            assert_eq!(&result, label);
            state.room_assignments[q_idx][a_idx + 1] = current_room;
            state.layer_assignments[q_idx][a_idx + 1] = current_layer;
            state.set_label(Room(current_room, current_layer), Some(result));
            if dfs(problem, query_results, state, next_q_idx, next_a_idx, memo) {
                return true;
            }
            state.set_label(Room(current_room, current_layer), Some(prev_result));
        }
        Action::Door(door) => {
            let candidates =
                state.next_room_candidates(Room(current_room, current_layer), *door, result);

            for candidate in candidates {
                let add_move_result = state.add_move(
                    Room(current_room, current_layer),
                    *door,
                    candidate.clone(),
                    (false, false),
                );
                if a_idx < query_results[q_idx].query.len() - 1 {
                    state.room_assignments[q_idx][a_idx + 1] = candidate.0;
                    state.layer_assignments[q_idx][a_idx + 1] = candidate.1;
                }
                if dfs(problem, query_results, state, next_q_idx, next_a_idx, memo) {
                    return true;
                }
                state.add_move(
                    Room(current_room, current_layer),
                    *door,
                    candidate.clone(),
                    (
                        add_move_result.is_plane_first_door,
                        add_move_result.is_layer_first_door,
                    ),
                );
            }
        }
    }

    let (next_query_idx, next_action_idx) = next_idx(q_idx, a_idx, &query_results);
    dfs(
        problem,
        query_results,
        state,
        next_query_idx,
        next_action_idx,
        memo,
    )
}

fn next_idx(q_idx: usize, a_idx: usize, query_resuls: &Vec<QueryResult>) -> (usize, usize) {
    let next_query_idx = if a_idx == query_resuls[q_idx].query.len() - 1 {
        q_idx + 1
    } else {
        0
    };
    let next_action_idx = if q_idx != next_query_idx {
        0
    } else {
        a_idx + 1
    };

    (next_query_idx, next_action_idx)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Room(usize, usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlaneRoom(usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RoomDoor(Room, usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlaneRoomDoor(PlaneRoom, usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LayerInfo {
    /// その層における各部屋に現在着いているラベル
    pub labels: Vec<Option<usize>>,

    /// その層における各部屋から移動したときの層
    pub layer_doors: Vec<Vec<Option<usize>>>,

    /// その部屋の vacant_door の数
    pub vacant_door_num: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct State {
    pub N: usize,
    pub N_layers: usize,

    /// 元のラベル（1層）
    pub org_labels: Vec<Option<usize>>,

    /// 各層の情報
    pub layers: Vec<LayerInfo>,

    /// 平面で見たとき、各部屋から移動したときの平面上の部屋
    pub plane_doors: Vec<Vec<Option<usize>>>,

    /// kashikari count
    /// 部屋 i から部屋 j に行く方法 - 部屋 j から部屋 i に行く方法
    pub kashikari_count_plane: Vec<Vec<i32>>,
    pub kashikari_count: Vec<Vec<i32>>,
    pub vacant_door_num: Vec<usize>,

    /// 各リザルトでどこにいるかの割当て
    pub room_assignments: Vec<Vec<usize>>,
    pub layer_assignments: Vec<Vec<usize>>,
}

impl State {
    pub fn print(&self) {
        println!("org_labels: {:?}", self.org_labels);
        for l in 0..self.N_layers {
            println!("layer {l}: {:?}", self.layers[l].labels);
        }

        println!("plane_doors");
        for row in self.plane_doors.iter() {
            println!(" {:?}", row);
        }

        for l in 0..self.N_layers {
            println!("Layer {l}:");
            for row in self.layers[l].layer_doors.iter() {
                println!(" {:?}", row);
            }
            println!();
        }
    }

    pub fn set_label(&mut self, room: Room, label: Option<usize>) {
        if self.org_labels[room.0].is_none() {
            self.org_labels[room.0] = label;
            for l in 0..self.N_layers {
                self.layers[l].labels[room.0] = label;
            }
        } else {
            self.layers[room.1].labels[room.0] = label;
        }
    }

    pub fn set_label_plane(&mut self, plane_room: PlaneRoom, label: usize) {
        if self.org_labels[plane_room.0].is_none() {
            self.org_labels[plane_room.0] = Some(label);
        }
    }

    /// 現在の部屋からあるドアを通って、ある部屋に行くという動きをグラフに反映させる
    /// 平面上でその部屋が初めてか、層状でその部屋が初めてかを返す
    /// 平面上でその扉が初めてか、層状でその扉が初めてかを返す
    pub fn add_move(
        &mut self,
        room1: Room,
        door: usize,
        room2: Room,
        delete: (bool, bool),
    ) -> AddMoveResult {
        let room1_idx = self.get_room_idx(&room1);
        let room2_idx = self.get_room_idx(&room2);

        let mut result = AddMoveResult {
            is_plane_first_room: false,
            is_layer_first_room: false,
            is_plane_first_door: false,
            is_layer_first_door: false,
        };

        // 平面上の処理
        if self.plane_doors[room1.0][door].is_none() || delete.0 {
            self.plane_doors[room1.0][door] = if delete.0 { None } else { Some(room2.0) };
            self.layers[room1.1].layer_doors[room1.0][door] =
                if delete.0 { None } else { Some(room2.1) };
            let coeff = if delete.0 { -1 } else { 1 };
            self.kashikari_count_plane[room1.0][room2.0] += coeff;
            self.kashikari_count_plane[room2.0][room1.0] -= coeff;
            self.kashikari_count[room1_idx][room2_idx] += coeff;
            self.kashikari_count[room2_idx][room1_idx] -= coeff;
            self.vacant_door_num[room1.0] -= coeff as usize;

            result.is_plane_first_door = true;
            result.is_layer_first_door = true;

            return result;
        }

        // 層状の処理
        if self.layers[room1.1].layer_doors[room1.0][door].is_none() || delete.1 {
            // delete が true の場合、層状の扉を削除する
            let coeff = if delete.1 { -1 } else { 1 };

            self.layers[room1.1].layer_doors[room1.0][door] =
                if delete.1 { None } else { Some(room2.1) };
            let room1_idx = self.get_room_idx(&room1);
            let room2_idx = self.get_room_idx(&room2);
            self.kashikari_count[room1_idx][room2_idx] += coeff;
            self.kashikari_count[room2_idx][room1_idx] -= coeff;
            self.layers[room1.1].vacant_door_num[room1.0] -= coeff as usize;

            result.is_layer_first_door = true;

            return result;
        }

        result
    }

    pub fn next_room_candidates(&self, room: Room, door: usize, next_label: usize) -> Vec<Room> {
        let mut candidates = vec![];

        let possible_destinations = self.possible_destination(room, door);

        for destination in possible_destinations.iter() {
            if self.label_validity(destination, next_label) {
                candidates.push(destination.clone());
            }
        }

        candidates
    }

    /// ある部屋からあるドアを通ったときの行き先としてあり得るものを列挙
    fn possible_destination(&self, room: Room, door: usize) -> Vec<Room> {
        let mut result = vec![];

        // 既に平面では行く部屋を決めている場合
        if let Some(plane_room) = self.plane_doors[room.0][door] {
            // 更に、今のレイヤーから行くレイヤーが決まっている場合、その部屋に行く
            if let Some(layer) = self.layers[room.1].layer_doors[plane_room][door] {
                return vec![Room(plane_room, layer)];

            // レイヤーが決まってなければ、どのレイヤーにもいけるはず
            } else {
                // 他のレイヤーが行くレイヤーはいけない
                let mut not_dest_layer: HashSet<usize> = Default::default();
                for l in 0..self.N_layers {
                    if l != room.1 {
                        not_dest_layer.insert(l);
                    }
                }
                for l in 0..self.N_layers {
                    if not_dest_layer.contains(&l) {
                        continue;
                    }
                    if let Some(layer_room) = self.layers[l].layer_doors[plane_room][door] {
                        result.push(Room(layer_room, l));
                    }
                }
                return result;
            }
        }

        // 決まっていない場合は全部屋全レイヤーが一度候補になる
        for i in 0..self.N {
            for l in 0..self.N_layers {
                // 返報性原理チェック
                // こっちから (i, l) に行くようにしてもOKか確認したいときは？
                if self.check_henposei_principle(&room, door, &Room(i, l)) {
                    result.push(Room(i, l));
                }
            }
        }

        result
    }

    /// (Room, Door) から (next_room) に行くようにしたとき、返報性原理が矛盾しないかチェック
    /// 一個余裕がないといけない
    fn check_henposei_principle(&self, room: &Room, door: usize, next_room: &Room) -> bool {
        if self.layers[room.1].layer_doors[room.0][door].is_some() {
            return true;
        }

        // 平面での返報性原理と、立体での返報性原理がある
        if !self.check_henposei_principle_plane(room, door, next_room) {
            return false;
        }
        if !self.check_henposei_principle_layer(room, door, next_room) {
            return false;
        }

        true
    }

    fn check_henposei_principle_plane(&self, room: &Room, door: usize, next_room: &Room) -> bool {
        let vacant_door_num = self.vacant_door_num[room.0];

        // room から next_room に行く方法がいくつ多いか
        let kashikari = self.kashikari_count_plane[room.0][next_room.0] + 1;
        // 貸しがあるが、それよりも向こう側の空きドアが少ない
        if kashikari > self.vacant_door_num[next_room.0] as i32 {
            return false;
        }
        // 借りを返しきれない
        if kashikari + (vacant_door_num as i32) < 0 {
            return false;
        }

        // 他の部屋に対する借りを返せるか
        for i in 0..self.N {
            if i == next_room.0 {
                continue;
            }

            let kashikari = self.kashikari_count_plane[room.0][i];
            if kashikari + (vacant_door_num as i32) < 0 {
                return false;
            }
        }

        true
    }

    fn check_henposei_principle_layer(&self, room: &Room, door: usize, next_room: &Room) -> bool {
        let vacant_door_num = self.layers[room.1].vacant_door_num[room.0];

        let kashikari =
            self.kashikari_count[self.get_room_idx(room)][self.get_room_idx(next_room)] + 1;
        if kashikari > self.vacant_door_num[next_room.0] as i32 {
            return false;
        }
        if kashikari + (vacant_door_num as i32) < 0 {
            return false;
        }

        for i in 0..self.N * self.N_layers {
            if i == self.get_room_idx(room) {
                continue;
            }

            let kashikari = self.kashikari_count[self.get_room_idx(room)][i];
            if kashikari + (vacant_door_num as i32) < 0 {
                return false;
            }
        }

        true
    }

    /// ある部屋が label のラベルになっていることがあるかチェック
    fn label_validity(&self, room: &Room, label: usize) -> bool {
        if self.org_labels[room.0].is_none() {
            true
        } else if self.layers[room.1].labels[room.0].is_none() {
            self.org_labels[room.0] == Some(label)
        } else {
            self.layers[room.1].labels[room.0] == Some(label)
        }
    }

    fn get_room_idx(&self, room: &Room) -> usize {
        room.0 + room.1 * self.N
    }
}

/// N回ドアを開けるランダムなクエリを生成する
fn create_random_query(N: usize) -> Vec<Action> {
    let mut rng = rand::thread_rng();
    let mut query = vec![];
    for _ in 0..N {
        let label = rng.gen_range(0..4);
        query.push(Action::Mark(label));
        let door = rng.gen_range(0..6);
        query.push(Action::Door(door));
    }

    query
}

pub struct QueryResult {
    pub query: Vec<Action>,
    pub result: Vec<usize>,
}

fn get_query_results(queries: Vec<Vec<Action>>) -> Vec<QueryResult> {
    let client = ApiClient::new();
    let queries_str = queries
        .iter()
        .map(|query| Action::vec_to_str(query))
        .collect::<Vec<String>>();
    let query_results = client.explore(&queries_str).unwrap().results;
    let query_results = query_results
        .iter()
        .enumerate()
        .map(|(i, result)| QueryResult {
            query: queries[i].clone(),
            result: result.clone(),
        })
        .collect();
    query_results
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AddMoveResult {
    pub is_plane_first_room: bool,
    pub is_layer_first_room: bool,
    pub is_plane_first_door: bool,
    pub is_layer_first_door: bool,
}
