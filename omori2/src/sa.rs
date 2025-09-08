use std::sync::Arc;
use std::sync::Mutex;

use fxhash::FxHashMap as HashMap;
use fxhash::FxHashSet as HashSet;
use rand::Rng;
use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::api::BaseMap;
use crate::api::PlanStep;
use crate::dfs::DfsSolver;
// --- 焼きなましパラメータ ---
const INITIAL_TEMPERATURE: f64 = 100.0;
const COOLING_RATE: f64 = 0.99999;
const MAX_ITERATIONS: usize = 10000000;
const MAX_ROOMS: usize = 30;
const MAX_DOORS: usize = 6;

pub struct SimulatedAnnealingSolver {
    // 問題定義
    pub observed_labels: Vec<usize>,
    pub transitions: Vec<(usize, usize)>, // (from_observation_idx, door)
    pub num_rooms: usize,                 // 現在の部屋数

    // 探索中の状態
    pub assignment: Vec<usize>, // assignment[observation_idx] = room_id

    // コスト計算用の補助データ構造
    cost: i32,
    // graph[from_room][door][to_room] = 遷移の回数
    graph: Vec<Vec<Vec<usize>>>,
    filled_in_future: Vec<i32>,    // room -> count of filled doors
    kasikari_count: Vec<Vec<i32>>, // kasikari_count[from_room][to_room] = count

    known_inequalities: Vec<(usize, usize)>, // (obs_idx1, obs_idx2) という形で、obs_idx1 != obs_idx2 であるべきことを示す
    known_inequalities_by_obs_idx: Vec<Vec<usize>>,
}

impl SimulatedAnnealingSolver {
    pub fn new(plan_str: &str, results_str: &str, num_rooms: usize) -> Self {
        let plan: Vec<usize> = plan_str
            .chars()
            .map(|c| c.to_digit(10).unwrap() as usize)
            .collect();
        let observed_labels: Vec<usize> = results_str
            .chars()
            .map(|c| c.to_digit(10).unwrap() as usize)
            .collect();

        let known_inequalities = find_signatures_ineqs(plan_str, &observed_labels);

        let num_observations = observed_labels.len();

        let mut transitions = Vec::new();
        for i in 0..plan.len() {
            // transition[i] は、観測点 i から ドア plan[i] を通って 観測点 i+1 に行くことを示す
            transitions.push((i, plan[i]));
        }
        let mut assignment = vec![0; num_observations];
        for i in 0..num_observations {
            let label = observed_labels[i] as usize;
            assignment[i] = label;
        }
        let mut known_inequalities_by_obs_idx = vec![vec![]; num_observations];
        for (r1, r2) in &known_inequalities {
            known_inequalities_by_obs_idx[*r1].push(*r2);
            known_inequalities_by_obs_idx[*r2].push(*r1);
        }
        let mut solver = Self {
            observed_labels,
            transitions,
            assignment,
            num_rooms,
            cost: 0,
            graph: vec![vec![vec![0; MAX_ROOMS]; MAX_DOORS]; MAX_ROOMS],
            filled_in_future: vec![0; MAX_ROOMS],
            kasikari_count: vec![vec![0; MAX_ROOMS]; MAX_ROOMS],
            known_inequalities,
            known_inequalities_by_obs_idx,
        };

        solver.recalculate_cost();
        solver
    }

    const HENPOU_WEIGHT: i32 = 1;
    const INEQ_WEIGHT: i32 = 1;
    const DUP_WEIGHT: i32 = 0;
    /// 現在の `assignment` に基づいてコストをゼロから再計算する
    fn recalculate_cost(&mut self) {
        let mut total_cost: i32 = 0;
        // グラフをクリア
        for i in 0..MAX_ROOMS {
            for j in 0..MAX_DOORS {
                for k in 0..MAX_ROOMS {
                    self.graph[i][j][k] = 0;
                }
            }
            self.filled_in_future[i] = 0;
            for k in 0..MAX_ROOMS {
                self.kasikari_count[i][k] = 0;
            }
        }

        for (from_idx, door) in &self.transitions {
            let from_room = self.assignment[*from_idx];
            let to_room = self.assignment[*from_idx + 1];
            // 遷移の回数を増やす
            self.graph[from_room][*door][to_room] += 1;
        }

        for from_room in 0..self.num_rooms {
            for door in 0..MAX_DOORS {
                let sum = self.graph[from_room][door].iter().sum::<usize>();
                let max = self.graph[from_room][door].iter().max().unwrap();
                total_cost += (sum - max) as i32 * Self::DUP_WEIGHT;
            }
        }

        for from_room in 0..self.num_rooms {
            for door in 0..MAX_DOORS {
                for to_room in 0..self.num_rooms {
                    let count = self.graph[from_room][door][to_room];
                    if count > 0 {
                        self.filled_in_future[from_room] += 1;
                        self.kasikari_count[from_room][to_room] += 1 as i32;
                        self.kasikari_count[to_room][from_room] -= 1 as i32;
                    }
                }
            }
        }

        for from_room in 0..self.num_rooms {
            for to_room in 0..self.num_rooms {
                let count: i32 = self.kasikari_count[from_room][to_room];
                self.filled_in_future[from_room] -= count.min(0);
            }
            if self.filled_in_future[from_room] > 6 {
                total_cost += (self.filled_in_future[from_room] - 6) * Self::HENPOU_WEIGHT;
            }
        }

        for (room1, room2) in &self.known_inequalities {
            if self.assignment[*room1] == self.assignment[*room2] {
                total_cost += 1 * Self::INEQ_WEIGHT;
            }
        }
        self.cost = total_cost;
    }
    fn calculate_penalty(&self, room_id: usize) -> i32 {
        let mut ret: i32 = 0;
        // for door in 0..MAX_DOORS {
        //     let sum = self.graph[room_id][door].iter().sum::<usize>();
        //     let max = self.graph[room_id][door].iter().max().unwrap();
        //     ret += (sum - max) as i32;
        // }
        if self.filled_in_future[room_id] > 6 {
            ret += (self.filled_in_future[room_id] - 6) * Self::HENPOU_WEIGHT;
        }
        ret
    }

    /// 特定の観測点の部屋割り当てを変更した際のコスト差分を計算・適用する (修正版)
    fn update_point(&mut self, obs_idx: usize, new_room: usize) {
        let old_room = self.assignment[obs_idx];
        if old_room == new_room {
            return;
        }

        // --- Step 1: 影響を受ける部屋を特定し、変更前のコストを減算 ---
        let from_room_opt = if obs_idx > 0 {
            Some(self.assignment[obs_idx - 1])
        } else {
            None
        };
        let to_room_opt = if obs_idx < self.assignment.len() - 1 {
            Some(self.assignment[obs_idx + 1])
        } else {
            None
        };

        let mut affected_rooms = HashSet::default();
        affected_rooms.insert(old_room);
        affected_rooms.insert(new_room);
        if let Some(r) = from_room_opt {
            affected_rooms.insert(r);
        }
        if let Some(r) = to_room_opt {
            affected_rooms.insert(r);
        }

        for &room in &affected_rooms {
            self.cost -= self.calculate_penalty(room);
        }

        // --- Step 2: 状態を更新 ---

        // In-edge: (obs_idx - 1) -> obs_idx
        if let Some(from_room) = from_room_opt {
            let door = self.transitions[obs_idx - 1].1;

            // 古い遷移を削除
            self.graph[from_room][door][old_room] -= 1;
            if self.graph[from_room][door][old_room] == 0 {
                // (from_room, door) のエッジがなくなった
                self.filled_in_future[from_room] -= 1;
                // kasikariの更新
                let old_k_from_old = self.kasikari_count[from_room][old_room];
                let old_k_old_from = self.kasikari_count[old_room][from_room];
                self.kasikari_count[from_room][old_room] -= 1;
                self.kasikari_count[old_room][from_room] += 1;
                // kasikari変更によるfilled_in_futureの更新
                self.filled_in_future[from_room] -=
                    self.kasikari_count[from_room][old_room].min(0) - old_k_from_old.min(0);
                self.filled_in_future[old_room] -=
                    self.kasikari_count[old_room][from_room].min(0) - old_k_old_from.min(0);
            }

            // 新しい遷移を追加
            self.graph[from_room][door][new_room] += 1;
            if self.graph[from_room][door][new_room] == 1 {
                // (from_room, door) のエッジが新しくできた
                self.filled_in_future[from_room] += 1;
                // kasikariの更新
                let old_k_from_new = self.kasikari_count[from_room][new_room];
                let old_k_new_from = self.kasikari_count[new_room][from_room];
                self.kasikari_count[from_room][new_room] += 1;
                self.kasikari_count[new_room][from_room] -= 1;
                // kasikari変更によるfilled_in_futureの更新
                self.filled_in_future[from_room] -=
                    self.kasikari_count[from_room][new_room].min(0) - old_k_from_new.min(0);
                self.filled_in_future[new_room] -=
                    self.kasikari_count[new_room][from_room].min(0) - old_k_new_from.min(0);
            }
        }

        // Out-edge: obs_idx -> (obs_idx + 1)
        if let Some(to_room) = to_room_opt {
            let door = self.transitions[obs_idx].1;

            // 古い遷移を削除
            self.graph[old_room][door][to_room] -= 1;
            if self.graph[old_room][door][to_room] == 0 {
                self.filled_in_future[old_room] -= 1;
                let old_k_old_to = self.kasikari_count[old_room][to_room];
                let old_k_to_old = self.kasikari_count[to_room][old_room];
                self.kasikari_count[old_room][to_room] -= 1;
                self.kasikari_count[to_room][old_room] += 1;
                self.filled_in_future[old_room] -=
                    self.kasikari_count[old_room][to_room].min(0) - old_k_old_to.min(0);
                self.filled_in_future[to_room] -=
                    self.kasikari_count[to_room][old_room].min(0) - old_k_to_old.min(0);
            }

            // 新しい遷移を追加
            self.graph[new_room][door][to_room] += 1;
            if self.graph[new_room][door][to_room] == 1 {
                self.filled_in_future[new_room] += 1;
                let old_k_new_to = self.kasikari_count[new_room][to_room];
                let old_k_to_new = self.kasikari_count[to_room][new_room];
                self.kasikari_count[new_room][to_room] += 1;
                self.kasikari_count[to_room][new_room] -= 1;
                self.filled_in_future[new_room] -=
                    self.kasikari_count[new_room][to_room].min(0) - old_k_new_to.min(0);
                self.filled_in_future[to_room] -=
                    self.kasikari_count[to_room][new_room].min(0) - old_k_to_new.min(0);
            }
        }

        // --- Step 3: 不等式制約と部屋割り当ての更新 ---
        let ineq_idxs = &self.known_inequalities_by_obs_idx[obs_idx];
        for &neq_idx in ineq_idxs {
            if self.assignment[neq_idx] == old_room {
                self.cost -= Self::INEQ_WEIGHT;
            }
            if self.assignment[neq_idx] == new_room {
                self.cost += Self::INEQ_WEIGHT;
            }
        }
        self.assignment[obs_idx] = new_room;

        // --- Step 4: 変更後のコストを加算 ---
        for &room in &affected_rooms {
            self.cost += self.calculate_penalty(room);
        }
    }

    pub fn solve(
        &mut self,
        thread_id: usize,
        stop_signal: Arc<Mutex<bool>>,

        full_plan_steps: Vec<PlanStep>,
        results_labeled_vec: Vec<usize>,

        layer_num: usize,
    ) -> Option<Vec<usize>> {
        let mut label_candidates_per_label = vec![vec![]; 4];
        for room_id in 0..self.num_rooms {
            let label = room_id % 4;
            label_candidates_per_label[label].push(room_id);
        }
        println!("Label candidates: {:?}", label_candidates_per_label);
        let mut rng = thread_rng();

        // 初期化
        for obs_idx in 1..self.assignment.len() {
            let label = self.observed_labels[obs_idx];
            self.assignment[obs_idx] = label_candidates_per_label[label]
                .choose(&mut rng)
                .unwrap()
                .to_owned();
        }
        self.assignment[0] = 0;
        self.recalculate_cost();

        let mut temperature = INITIAL_TEMPERATURE;

        println!("[Thread {}] Initial cost: {}", thread_id, self.cost);

        for i in 0..MAX_ITERATIONS {
            // Check if another thread has found a solution
            if i % 100000 == 0 && *stop_signal.lock().unwrap() {
                println!("[Thread {}] Stopped by another thread", thread_id);
                return None;
            }
            temperature = temperature.max(0.01);
            if self.cost == 0 {
                self.recalculate_cost();
                assert!(self.cost == 0);
                println!("[Thread {}] Found a solution with cost 0!", thread_id);

                let mut invalid = false;
                if !self.is_valid_assignment() {
                    println!("[Thread {}] But the assignment is invalid!", thread_id);
                    invalid = true;
                } else {
                    let base_map = self.build_base_map();

                    let mut dfs_solver = DfsSolver::new(
                        base_map,
                        full_plan_steps.clone(),
                        results_labeled_vec.clone(),
                        layer_num,
                    );

                    if let Some(solution) = dfs_solver.solve() {
                        *stop_signal.lock().unwrap() = true;
                        println!("[Thread {}] DFS found a solution!", thread_id);
                    } else {
                        println!("[Thread {}] DFS could not find a solution.", thread_id);
                        invalid = true;
                    }
                }
                if invalid {
                    // 適当にkick
                    for obs_idx in 1..self.assignment.len() {
                        if rng.gen_bool(0.6) {
                            let label = self.observed_labels[obs_idx];
                            self.assignment[obs_idx] = label_candidates_per_label[label]
                                .choose(&mut rng)
                                .unwrap()
                                .to_owned();
                        }
                    }
                    self.recalculate_cost();
                    continue;
                }

                return Some(self.assignment.clone());
            }
            let original_cost: i32 = self.cost;
            if i % 100000 == 0 && i > 0 {
                // 特定のラベルのノードを30%くらいリセットする
                //println!("[Thread {}] Resetting all nodes", thread_id);
                for obs_idx in 1..self.assignment.len() {
                    if rng.gen_bool(0.05) {
                        self.assignment[obs_idx] = label_candidates_per_label
                            [self.observed_labels[obs_idx]]
                            .choose(&mut rng)
                            .unwrap()
                            .to_owned();
                    }
                }
                self.recalculate_cost();
            } else {
                let obs_idx_to_move = rng.gen_range(1..self.assignment.len());
                let old_room = self.assignment[obs_idx_to_move];

                let new_room = loop {
                    let label = self.observed_labels[obs_idx_to_move];
                    break label_candidates_per_label[label]
                        .choose(&mut rng)
                        .unwrap()
                        .to_owned();
                };
                if new_room == old_room {
                    continue;
                }

                // 差分計算を使用（assignmentを更新する前に呼ぶ）
                self.update_point(obs_idx_to_move, new_room);

                let new_cost = self.cost;
                let cost_delta = new_cost - original_cost;

                // --- 遷移の承認/棄却 ---
                if cost_delta < 0
                    || (temperature > 0.0
                        && rng.r#gen::<f64>() < (-cost_delta as f64 / temperature).exp())
                {
                    // 遷移を承認
                } else {
                    self.update_point(obs_idx_to_move, old_room);
                    assert!(self.cost == original_cost);
                }
            }

            temperature *= COOLING_RATE;

            if i % 1000000 == (1000000 - 1) {
                // Cost が１桁のときは、赤色にする. Cost: 1の時が一番赤くて、Cost:10の時は黄色。グラデーションに
                let cost_color = if self.cost <= 10 {
                    let ratio = (10 - self.cost) as f64 / 10.0;
                    let red = (255.0 * ratio) as u8;
                    let green = (255.0 * (1.0 - ratio)) as u8;
                    format!("\x1b[38;2;{};{};0m", red, green)
                } else {
                    "\x1b[0m".to_string()
                };
                println!(
                    "{}[Thread {}] Iter: {}, Temp: {:.4}, Cost: {}, NumRooms: {}",
                    cost_color, thread_id, i, temperature, self.cost, self.num_rooms
                );
            }
        }

        // let mut count_incoming_tuple_of_room_and_door: HashMap<usize, HashSet<(usize, usize)>> =
        //     HashMap::default();

        // for (from_idx, door) in &self.transitions {
        //     let to_idx = from_idx + 1;
        //     let from_room = self.assignment[*from_idx];
        //     let to_room = self.assignment[to_idx];
        //     count_incoming_tuple_of_room_and_door
        //         .entry(to_room)
        //         .or_default()
        //         .insert((from_room, *door));
        // }
        // for (room_id, incoming) in &count_incoming_tuple_of_room_and_door {
        //     println!("Room {}: incoming {:?}", room_id, incoming);
        // }

        println!("[Thread {}] Finished without finding cost 0.", thread_id);
        println!("[Thread {}] Final cost: {}", thread_id, self.cost);
        println!("[Thread {}] Final num_rooms: {}", thread_id, self.num_rooms);
        None
    }
    pub fn build_base_map(&self) -> BaseMap {
        let mut connections = HashMap::default();
        for (from_idx, door) in self.transitions.iter() {
            let from_room = self.assignment[*from_idx];
            let to_room = self.assignment[*from_idx + 1];
            let existing = connections.get(&(from_room, *door));
            if existing != Some(&to_room) && existing.is_some() {
                println!(
                    "Warning: Duplicate connection from room {} door {}. {} vs {}",
                    from_room,
                    door,
                    connections.get(&(from_room, *door)).unwrap(),
                    to_room
                );
                assert!(false);
            }
            connections.insert((from_room, *door), to_room);
        }

        BaseMap {
            num_rooms: self.num_rooms,
            starting_room: self.assignment[0],
            connections,
        }
    }

    pub fn is_valid_assignment(&self) -> bool {
        let mut cur = 0;
        assert!(self.assignment[0] == 0);

        // 行き先のduplicateがないことを確認
        for from_room in 0..self.num_rooms {
            for door in 0..MAX_DOORS {
                if self.graph[from_room][door]
                    .iter()
                    .filter(|&&x| x > 0)
                    .count()
                    > 1
                {
                    println!(
                        "Invalid: from_room {} door {} has multiple outgoing edges",
                        from_room, door
                    );
                    return false;
                }
            }
        }

        for i in 0..self.transitions.len() {
            let (from_idx, door) = self.transitions[i];
            let to_idx = from_idx + 1;
            let from_room = self.assignment[from_idx];
            let to_room = self.assignment[to_idx];
            assert!(from_room == cur);
            assert!(self.observed_labels[from_idx] == from_room % 4);
            assert!(self.observed_labels[to_idx] == to_room % 4);
            cur = to_room;
        }
        true
    }
}

// シグネチャ： ある長さのsuffixに対して、resultsが少しでも異なるなら、異なる部屋がわりあたるべきだ.
// returns 複数の不等式
fn find_signatures_ineqs(plan: &str, results: &Vec<usize>) -> Vec<(usize, usize)> {
    let mut inequalities = HashSet::default();
    for sig_len in (1..plan.len()) {
        for sig_start in (0..=plan.len() - sig_len) {
            let sig = &plan[sig_start..sig_start + sig_len];
            // 他の場所で同じsubstringがあるか?
            for other_start in (sig_start + 1..=plan.len() - sig_len) {
                if results[sig_start] != results[other_start] {
                    continue;
                }
                let other = &plan[other_start..other_start + sig_len];
                if sig == other {
                    // 同じsubstringが見つかった
                    // それぞれに対応するresultsを比較して、最後の文字だけが異なるなら、不要しき
                    if (0..sig_len).all(|i| {
                        if i < sig_len - 1 {
                            results[1 + sig_start + i] == results[1 + other_start + i]
                        } else {
                            results[1 + sig_start + i] != results[1 + other_start + i]
                        }
                    }) {
                        if sig_start < other_start {
                            inequalities.insert((sig_start, other_start));
                        } else {
                            inequalities.insert((other_start, sig_start));
                        }
                    }
                }
            }
        }
    }
    inequalities.into_iter().collect()
}
