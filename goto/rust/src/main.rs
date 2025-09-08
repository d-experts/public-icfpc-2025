use core::num;
use fxhash::FxHashMap as HashMap;
use fxhash::FxHashSet as HashSet;
use itertools::Itertools;
use rand::prelude::*;
use std::iter::once;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::aleph::gen_new_plan;
use crate::api::{Connection, Map, RoomAndDoor};

// --- 焼きなましパラメータ ---
const INITIAL_TEMPERATURE: f64 = 1.0;
const COOLING_RATE: f64 = 0.99999;
const MAX_ITERATIONS: usize = 10000000;
const NUM_PARALLEL_THREADS: usize = 90;
const MAX_ROOMS: usize = 30;
const MAX_DOORS: usize = 6;

struct SimulatedAnnealingSolver {
    /// 問題定義
    observed_labels: Vec<Vec<usize>>, // observed_labels[query_id][observation_idx] = label
    transitions: Vec<Vec<(usize, usize)>>, // transitions[query_id][step] = (from_observation_idx, door)
    num_rooms: usize,                      // 現在の部屋数

    /// 探索中の状態
    assignment: Vec<Vec<usize>>, // assignment[query_id][observation_idx] = room_id

    /// コスト計算用の補助データ構造
    cost: i32,
    // graph[from_room][door][to_room] = 遷移の回数
    graph: Vec<Vec<Vec<usize>>>,
    filled_in_future: Vec<i32>,    // room -> count of filled doors
    kasikari_count: Vec<Vec<i32>>, // kasikari_count[from_room][to_room] = count

    known_inequalities: Vec<(usize, usize, usize, usize)>, // (plan_idx1, obs_idx1, plan_idx2, obs_idx2) という形で、obs_idx1 != obs_idx2 であるべきことを示す
    known_inequalities_by_obs_idx: Vec<Vec<Vec<(usize, usize)>>>,
}

impl SimulatedAnnealingSolver {
    pub fn new(plan_str: Vec<String>, results_str: Vec<String>, num_rooms: usize) -> Self {
        let plan: Vec<Vec<usize>> = plan_str
            .iter()
            .map(|s| {
                s.chars()
                    .map(|c| c.to_digit(10).unwrap() as usize)
                    .collect()
            })
            .collect();
        let observed_labels: Vec<Vec<usize>> = results_str
            .iter()
            .map(|s| {
                s.chars()
                    .map(|c| c.to_digit(10).unwrap() as usize)
                    .collect()
            })
            .collect();

        let known_inequalities = find_signatures_ineqs(plan_str, &observed_labels);

        let mut transitions = plan
            .iter()
            .map(|single_plan| {
                let mut single_t = Vec::new();
                for i in 0..single_plan.len() {
                    single_t.push((i, single_plan[i]))
                }
                single_t
            })
            .collect();

        let mut assignment = vec![];
        observed_labels
            .iter()
            .enumerate()
            .for_each(|(idx, single_obs)| {
                let num_observations = single_obs.len();
                let mut assignment_single = vec![0; num_observations];
                for i in 0..num_observations {
                    let label = single_obs[i];
                    assignment_single[i] = label;
                }
                assignment.push(assignment_single);
            });

        let mut known_inequalities_by_obs_idx =
            vec![vec![vec![]; observed_labels[0].len()]; observed_labels.len()];
        for (p1, r1, p2, r2) in &known_inequalities {
            known_inequalities_by_obs_idx[*p1][*r1].push((*p2, *r2));
            known_inequalities_by_obs_idx[*p2][*r2].push((*p1, *r1));
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

        for i in 0..self.transitions.len() {
            let transitions_single = &self.transitions[i];
            for (from_idx, door) in transitions_single {
                let from_room = self.assignment[i][*from_idx];
                let to_room = self.assignment[i][*from_idx + 1];
                // 遷移の回数を増やす
                self.graph[from_room][*door][to_room] += 1;
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
                total_cost += self.filled_in_future[from_room] - 6;
            }
        }

        for (plan1, room1, plan2, room2) in &self.known_inequalities {
            if self.assignment[*plan1][*room1] == self.assignment[*plan2][*room2] {
                total_cost += 1;
            }
        }

        self.cost = total_cost;
    }
    fn calculate_penalty(&self, room_id: usize) -> i32 {
        if self.filled_in_future[room_id] > 6 {
            self.filled_in_future[room_id] - 6
        } else {
            0
        }
    }

    /// 特定の観測点の部屋割り当てを変更した際のコスト差分を計算・適用する (修正版)
    fn update_point(&mut self, plan_idx: usize, obs_idx: usize, new_room: usize) {
        let old_room = self.assignment[plan_idx][obs_idx];
        if old_room == new_room {
            return;
        }

        // --- Step 1: 影響を受ける部屋を特定し、変更前のコストを減算 ---
        let from_room_opt = if obs_idx > 0 {
            Some(self.assignment[plan_idx][obs_idx - 1])
        } else {
            None
        };
        let to_room_opt = if obs_idx < self.assignment[plan_idx].len() - 1 {
            Some(self.assignment[plan_idx][obs_idx + 1])
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
            let door = self.transitions[plan_idx][obs_idx - 1].1;

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
            let door = self.transitions[plan_idx][obs_idx].1;

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
        let ineq_idxs = &self.known_inequalities_by_obs_idx[plan_idx][obs_idx];
        for &(neq_plan, neq_idx) in ineq_idxs {
            if self.assignment[neq_plan][neq_idx] == old_room {
                self.cost -= 1;
            }
            if self.assignment[neq_plan][neq_idx] == new_room {
                self.cost += 1;
            }
        }
        self.assignment[plan_idx][obs_idx] = new_room;

        // --- Step 4: 変更後のコストを加算 ---
        for &room in &affected_rooms {
            self.cost += self.calculate_penalty(room);
        }

        // // デバッグ用: 差分計算が正しいか確認
        // #[cfg(debug_assertions)]
        // {
        //     let old_cost = self.cost;
        //     let old_graph = self.graph.clone();
        //     let old_filled = self.filled_in_future.clone();
        //     let old_kasikari = self.kasikari_count.clone();

        //     self.recalculate_cost();

        //     if old_cost != self.cost {
        //         println!("Cost mismatch: delta={}, recalc={}", old_cost, self.cost);
        //         println!(
        //             "obs_idx={}, old_room={}, new_room={}",
        //             obs_idx, old_room, new_room
        //         );
        //         // filled_countの差異を出力
        //         for i in 0..self.num_rooms {
        //             if old_filled[i] != self.filled_in_future[i] {
        //                 println!(
        //                     "  filled_count[{}]: {} -> {}",
        //                     i, old_filled[i], self.filled_in_future[i]
        //                 );
        //             }
        //         }
        //         // kasikari_countの差異を出力
        //         println!("Kasikari count differences:");
        //         for i in 0..self.num_rooms {
        //             for j in 0..self.num_rooms {
        //                 if old_kasikari[i][j] != self.kasikari_count[i][j] {
        //                     println!(
        //                         "  kasikari[{}][{}]: {} -> {}",
        //                         i, j, old_kasikari[i][j], self.kasikari_count[i][j]
        //                     );
        //                 }
        //             }
        //         }
        //         panic!("Cost mismatch detected!");
        //     }
        //     if old_graph != self.graph {
        //         println!("Graph mismatch after update_cost_delta");
        //         println!(
        //             "obs_idx={}, old_room={}, new_room={}",
        //             obs_idx, old_room, new_room
        //         );
        //         println!("assignment[obs_idx]={}", self.assignment[obs_idx]);
        //         if obs_idx > 0 {
        //             println!("assignment[obs_idx-1]={}", self.assignment[obs_idx - 1]);
        //         }
        //         if obs_idx < self.assignment.len() - 1 {
        //             println!("assignment[obs_idx+1]={}", self.assignment[obs_idx + 1]);
        //         }
        //         for i in 0..self.num_rooms {
        //             for j in 0..MAX_DOORS {
        //                 for k in 0..self.num_rooms {
        //                     if old_graph[i][j][k] != self.graph[i][j][k] {
        //                         println!(
        //                             "  graph[{}][{}][{}]: {} -> {}",
        //                             i, j, k, old_graph[i][j][k], self.graph[i][j][k]
        //                         );
        //                     }
        //                 }
        //             }
        //         }
        //         panic!("Graph mismatch detected!");
        //     }
        // }
    }

    pub fn solve(
        &mut self,
        thread_id: usize,
        stop_signal: Arc<Mutex<bool>>,
    ) -> Option<Vec<Vec<usize>>> {
        let mut label_candidates_per_label = vec![vec![]; 4];
        for room_id in 0..self.num_rooms {
            let label = room_id % 4;
            label_candidates_per_label[label].push(room_id);
        }
        println!("Label candidates: {:?}", label_candidates_per_label);
        let mut rng = thread_rng();

        // 初期化
        for plan_idx in 0..self.assignment.len() {
            for obs_idx in 1..self.assignment[plan_idx].len() {
                let label = self.observed_labels[plan_idx][obs_idx];
                self.assignment[plan_idx][obs_idx] = label_candidates_per_label[label]
                    .choose(&mut rng)
                    .unwrap()
                    .to_owned();
            }
        }
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
                println!("[Thread {}] Found a solution with cost 0!", thread_id);
                *stop_signal.lock().unwrap() = true;
                self.print_results();
                return Some(self.assignment.clone());
            }
            let original_cost: i32 = self.cost;
            if i % 100000 == 0 && i > 0 {
                // 特定のラベルのノードを30%くらいリセットする
                // println!("[Thread {}] Resetting all nodes", thread_id);
                for plan_idx in 0..self.assignment.len() {
                    for obs_idx in 1..self.assignment[plan_idx].len() {
                        if rng.gen_bool(0.05) {
                            let label = self.observed_labels[plan_idx][obs_idx];
                            self.assignment[plan_idx][obs_idx] = label_candidates_per_label[label]
                                .choose(&mut rng)
                                .unwrap()
                                .to_owned();
                        }
                    }
                }
                self.recalculate_cost();
            } else {
                let plan_idx_to_move = rng.gen_range(0..self.assignment.len());
                let obs_idx_to_move = rng.gen_range(1..self.assignment[plan_idx_to_move].len());
                let old_room = self.assignment[plan_idx_to_move][obs_idx_to_move];

                let new_room = loop {
                    let label = self.observed_labels[plan_idx_to_move][obs_idx_to_move];
                    break label_candidates_per_label[label]
                        .choose(&mut rng)
                        .unwrap()
                        .to_owned();
                };
                if new_room == old_room {
                    continue;
                }

                // 差分計算を使用（assignmentを更新する前に呼ぶ）
                self.update_point(plan_idx_to_move, obs_idx_to_move, new_room);

                let new_cost = self.cost;
                let cost_delta = new_cost - original_cost;

                // --- 遷移の承認/棄却 ---
                if cost_delta < 0
                    || (temperature > 0.0
                        && rng.r#gen::<f64>() < (-cost_delta as f64 / temperature).exp())
                {
                    // 遷移を承認
                } else {
                    self.update_point(plan_idx_to_move, obs_idx_to_move, old_room);
                    assert_eq!(self.cost, original_cost);
                }
            }

            temperature *= COOLING_RATE;

            if i % 100000 == (100000 - 1) {
                // Cost が１桁のときは、赤色にする. Cost: 1の時が一番赤くて、Cost:10の時は黄色。グラデーションに
                let cost_color = if self.cost <= 10 {
                    let ratio = (10 - self.cost) as f64 / 10.0;
                    let red = (255.0 * ratio) as u8;
                    let green = (255.0 * (1.0 - ratio)) as u8;
                    format!("\x1b[38;2;{};{};0m", red, green)
                } else {
                    "\x1b[0m".to_string()
                };
                // println!(
                //     "{}[Thread {}] Iter: {}, Temp: {:.4}, Cost: {}, NumRooms: {}",
                //     cost_color, thread_id, i, temperature, self.cost, self.num_rooms
                // );
            }
        }

        let mut count_incoming_tuple_of_room_and_door: HashMap<usize, HashSet<(usize, usize)>> =
            HashMap::default();

        for plan_idx in 0..self.assignment.len() {
            for (from_idx, door) in &self.transitions[plan_idx] {
                let to_idx = from_idx + 1;
                let from_room = self.assignment[plan_idx][*from_idx];
                let to_room = self.assignment[plan_idx][to_idx];
                count_incoming_tuple_of_room_and_door
                    .entry(to_room)
                    .or_default()
                    .insert((from_room, *door));
            }
            for (room_id, incoming) in &count_incoming_tuple_of_room_and_door {
                println!("Room {}: incoming {:?}", room_id, incoming);
            }
        }

        println!("[Thread {}] Finished without finding cost 0.", thread_id);
        println!("[Thread {}] Final cost: {}", thread_id, self.cost);
        println!("[Thread {}] Final num_rooms: {}", thread_id, self.num_rooms);
        None
    }

    fn fill_missing_connections_randomly(
        &self,
        map: &HashMap<(usize, usize), usize>,
    ) -> HashMap<(usize, usize), usize> {
        let mut map = map.clone();
        // 2,0 -> 1  みたいな遷移があると、 1,? -> 2 も必要になる. 部屋同士の貸し借りを数える
        for c in (0..self.num_rooms).permutations(2) {
            let (from_room, to_room) = (c[0], c[1]);
            let mut from_to_count: HashSet<usize> = HashSet::default();
            let mut to_from_count: HashSet<usize> = HashSet::default();
            for door in 0..6 {
                if let Some(&dest_room) = map.get(&(from_room, door)) {
                    if dest_room == to_room {
                        from_to_count.insert(door);
                    }
                }
                if let Some(&src_room) = map.get(&(to_room, door)) {
                    if src_room == from_room {
                        to_from_count.insert(door);
                    }
                }
            }
            let total_from_to: usize = from_to_count.len();
            let total_to_from: usize = to_from_count.len();
            println!(
                "Room {} <-> Room {} : {} -> {}",
                from_room, to_room, total_from_to, total_to_from
            );
            if total_from_to > total_to_from {
                for _ in 0..(total_from_to - total_to_from) {
                    // from_room -> to_room の方が多いので、to_room -> from_room の遷移を追加する
                    let doors: Vec<usize> = (0..6).collect();
                    let empty_door = *doors
                        .iter()
                        .find(|&&d| map.get(&(to_room, d)).is_none())
                        .unwrap();
                    map.insert((to_room, empty_door), from_room);
                }
            }
        }
        // まだ空のやつは自己ループにする
        for room_id in 0..self.num_rooms {
            for door in 0..6 {
                if map.get(&(room_id, door)).is_none() {
                    println!("Filling self-loop: {} --{}--> {}", room_id, door, room_id);
                    map.insert((room_id, door), room_id);
                }
            }
        }
        map
    }

    fn print_results(&self) {
        println!("\n--- Assignment Results ---");
        for plan_idx in 0..self.assignment.len() {
            print!("Plan {}: ", plan_idx);
            for (obs_idx, room_id) in self.assignment[plan_idx].iter().enumerate() {
                print!("{},", room_id);
            }
            println!();
        }

        println!();

        let mut transition_table: HashMap<(usize, usize), usize> = HashMap::default();
        for plan_idx in 0..self.assignment.len() {
            for (from_idx, door) in self.transitions[plan_idx].iter() {
                let from_room = self.assignment[plan_idx][*from_idx];
                let to_room = self.assignment[plan_idx][*from_idx + 1];
                transition_table.insert((from_room, *door), to_room);
            }
        }

        let mut incoming_count: HashMap<usize, usize> = HashMap::default();
        for ((from_room, door), to_room) in
            transition_table.iter().sorted_by_key(|&((f, d), _)| (f, d))
        {
            println!("({}, {}) -> {}", from_room, door, to_room);
            incoming_count
                .entry(*to_room)
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }
        println!("Incoming counts: {:?}", incoming_count);

        let transition_table = self.fill_missing_connections_randomly(&transition_table);
        println!("\n--- Final Transition Table ---");
        for room_id in 0..self.num_rooms {
            for door in 0..6 {
                if let Some(&to_room) = transition_table.get(&(room_id, door)) {
                    println!("({}, {}) -> {}", room_id, door, to_room);
                }
            }
        }
        let d2d = self.calc_door_2_door_map(transition_table);
        println!("\n--- Door-to-Door Map ---");
        for ((from_room, door1), (to_room, door2)) in
            d2d.iter().sorted_by_key(|&((f, d1), _)| (f, d1))
        {
            println!("({}, {}) -> ({}, {})", from_room, door1, to_room, door2);
        }
    }

    fn calc_door_2_door_map(
        &self,
        transition_table: HashMap<(usize, usize), usize>,
    ) -> HashMap<(usize, usize), (usize, usize)> {
        // from, door1 -> to に対して to, ? -> from を適当に一つ見つけてきて、(from, door1) -> (to, door2) の形にする
        let mut door_2_door_map: HashMap<(usize, usize), (usize, usize)> = HashMap::default();
        for ((from_room, door1), to_room) in transition_table.iter() {
            if door_2_door_map.contains_key(&(*from_room, *door1)) {
                continue;
            }
            let mut candidates: Vec<usize> = Vec::new();
            for door2 in 0..6 {
                if let Some(&dest_room) = transition_table.get(&(to_room.clone(), door2)) {
                    if dest_room == *from_room && !door_2_door_map.contains_key(&(*to_room, door2))
                    {
                        candidates.push(door2);
                    }
                }
            }
            if candidates.is_empty() {
                panic!(
                    "No available door found for reverse transition: {} --?--> {}. Original: {} --{}--> {}",
                    to_room, from_room, from_room, door1, to_room
                );
            }
            let door2 = candidates.choose(&mut thread_rng()).unwrap();
            door_2_door_map.insert((*from_room, *door1), (*to_room, *door2));
            door_2_door_map.insert((*to_room, *door2), (*from_room, *door1));
        }
        door_2_door_map
    }

    fn build_submission_map(&self) -> api::Map {
        // 1. (room, door) -> next_room のテーブルを構築
        let mut transition_table: HashMap<(usize, usize), usize> = HashMap::default();
        for plan_idx in 0..self.assignment.len() {
            for (from_idx, door) in &self.transitions[plan_idx] {
                let to_idx = from_idx + 1;
                let from_room = self.assignment[plan_idx][*from_idx];
                let to_room = self.assignment[plan_idx][to_idx];
                transition_table.insert((from_room, *door), to_room);
            }
        }

        // 2. 未確定の接続を補完する
        let full_transition_table = self.fill_missing_connections_randomly(&transition_table);

        // 3. (room, door) -> (room, door) のペアを作る
        let door_to_door_map = self.calc_door_2_door_map(full_transition_table);

        // 4. 提出形式に変換
        let rooms: Vec<usize> = (0..self.num_rooms).map(|r| r % 4).collect();
        let starting_room = self.assignment[0][0];

        println!("!!!!!!!Starting room: {}!!!!!!!", starting_room);

        let mut connections = Vec::new();
        let mut processed_connections = HashSet::default();

        for ((from_room, from_door), (to_room, to_door)) in door_to_door_map.iter() {
            let conn1 = RoomAndDoor {
                room: *from_room,
                door: *from_door,
            };
            let conn2 = RoomAndDoor {
                room: *to_room,
                door: *to_door,
            };

            // 無向グラフなので、(A,B) と (B,A) の両方を処理しないようにする
            let key1 = (conn1.room, conn1.door, conn2.room, conn2.door);
            let key2 = (conn2.room, conn2.door, conn1.room, conn1.door);

            if !processed_connections.contains(&key1) && !processed_connections.contains(&key2) {
                connections.push(Connection {
                    from: conn1,
                    to: conn2,
                });
                processed_connections.insert(key1);
                processed_connections.insert(key2);
            }
        }

        Map {
            rooms,
            starting_room,
            connections,
        }
    }
}

// シグネチャ： ある長さのsuffixに対して、resultsが少しでも異なるなら、異なる部屋がわりあたるべきだ.
// returns 複数の不等式
fn find_signatures_ineqs(
    plans: Vec<String>,
    results: &Vec<Vec<usize>>,
) -> Vec<(usize, usize, usize, usize)> {
    let mut inequalities = HashSet::default();
    for start_plan in 0..plans.len() {
        for sig_len in 1..plans[start_plan].len() {
            for sig_start in 0..plans[start_plan].len() - sig_len {
                let sig = &plans[start_plan][sig_start..sig_start + sig_len];
                // 他の場所で同じsubstringがあるか?
                for other_plan in start_plan..plans.len() {
                    for other_start in match start_plan == other_plan {
                        true => sig_start + 1..plans[other_plan].len() - sig_len,
                        false => 0..plans[other_plan].len() - sig_len,
                    } {
                        if results[start_plan][sig_start] != results[other_plan][other_start] {
                            continue;
                        }
                        let other = &plans[other_plan][other_start..other_start + sig_len];
                        if sig == other {
                            // 同じsubstringが見つかった
                            // それぞれに対応するresultsを比較して、最後の文字だけが異なるなら、不要しき
                            if (0..sig_len).all(|i| {
                                if i < sig_len - 1 {
                                    results[start_plan][1 + sig_start + i]
                                        == results[other_plan][1 + other_start + i]
                                } else {
                                    results[start_plan][1 + sig_start + i]
                                        != results[other_plan][1 + other_start + i]
                                }
                            }) {
                                inequalities.insert((
                                    start_plan,
                                    sig_start,
                                    other_plan,
                                    other_start,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
    inequalities.into_iter().collect()
}

fn gen_random_string(alphabet: &str, length: usize, rng: &mut impl Rng) -> String {
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..alphabet.len());
            alphabet.chars().nth(idx).unwrap()
        })
        .collect()
}

// マップを見てすべての

pub mod aleph;
pub mod api;

fn main() {
    let api_client = api::ApiClient::new();

    let mut iteeeer = 0;
    loop {
        iteeeer += 1;
        let num_rooms = 24;
        let bb = 18;
        let select_response = api_client.select_problem("teth").unwrap();
        // println!("Select response: {:?}", select_response);
        let oni_plan = "101000355110224551423435433021124433432312145253145124220224433254303442443030550402353401153505245234541244013123041522553444102052141153442355244134242325423132220032442040450311012513112254353413014132045533205510051322500155213225531225303232043000144345515151111053311223533013443540045351501020524234235231500511344453422134033231443300021331105455314301041113453331023303110035055150325044222550550111213234133540201315415545".to_string();

        let num_simple_plans = 1;
        let mut simple_plans = (0..num_simple_plans)
            .map(|_| gen_random_string("012345", num_rooms * bb, &mut thread_rng()))
            .collect::<Vec<String>>();

        let gachi_plan = gen_new_plan(&oni_plan, &mut thread_rng());

        let explore_response: api::ExploreResponse = api_client
            .explore(
                &simple_plans
                    .iter()
                    .chain(once(&gachi_plan))
                    .cloned()
                    .collect::<Vec<String>>(),
            )
            .unwrap();
        let simple_results = explore_response.results[0..num_simple_plans].to_vec();
        let simple_results_strs = simple_results
            .iter()
            .map(|res| res.iter().map(|r| r.to_string()).collect::<String>())
            .collect::<Vec<String>>();
        let gachi_result = explore_response.results[num_simple_plans].clone();

        // Run parallel simulated annealing with configurable thread count
        println!(
            "Starting parallel simulated annealing with {} threads...",
            NUM_PARALLEL_THREADS
        );

        let (tx, rx) = mpsc::channel();
        let stop_signal = Arc::new(Mutex::new(false));
        let mut handles = vec![];

        for thread_id in 0..NUM_PARALLEL_THREADS {
            let tx = tx.clone();
            let stop_signal = stop_signal.clone();

            let _simple_plans = simple_plans.clone();
            let _simple_results_strs = simple_results_strs.clone();
            // plan_clone, plan2_clone の vec を渡す
            let handle = thread::spawn(move || {
                let mut solver =
                    SimulatedAnnealingSolver::new(_simple_plans, _simple_results_strs, num_rooms);
                if let Some(assignment) = solver.solve(thread_id, stop_signal) {
                    tx.send((thread_id, solver)).ok();
                }
            });
            handles.push(handle);
        }

        // Drop the original sender so the receiver can detect when all threads are done
        drop(tx);

        // Wait for the first solution or all threads to finish
        let solution = rx.recv(); // No timeout - wait indefinitely

        // Signal all threads to stop
        *stop_signal.lock().unwrap() = true;

        // Wait for all threads to finish
        for handle in handles {
            handle.join().ok();
        }

        // Process the solution if found
        if let Ok((winning_thread, mut solver)) = solution {
            println!("\n★ Thread {} found the solution first! ★", winning_thread);
            if solver.cost == 0 {
                // println!("\n--- Found a valid graph structure! ---");
                let mut rooms_map: HashMap<usize, Vec<usize>> = HashMap::default();

                for plan_idx in 0..solver.assignment.len() {
                    for (obs_idx, room_id) in solver.assignment[plan_idx].iter().enumerate() {
                        rooms_map.entry(*room_id).or_default().push(obs_idx);
                    }
                }
                println!("Number of rooms: {}", rooms_map.len());
                for (room_id, obs_indices) in rooms_map.iter() {
                    let label = solver.observed_labels[0][obs_indices[0]];
                    println!(
                        "Room {} (label {}): assigned observations {:?}",
                        room_id, label, obs_indices
                    );
                }

                solver.print_results();

                // 5. 解が見つかったら、提出用のMap形式に変換
                let final_map = solver.build_submission_map();
                let mut cpp_result = false;
                let mut go_result = false;

                match aleph::run_cpp_with_json(&gachi_plan, &gachi_result, &final_map) {
                    Ok(res) => {
                        println!("Cpp program returned: {}", res);
                        if res {
                            println!("Cpp program returned true, exiting...");
                            cpp_result = true;
                        }
                    }
                    Err(e) => {
                        println!("Error running Cpp program: {}, continuing...", e);
                    }
                }

                // match aleph::run_go_with_json(&gachi_plan, &gachi_result, &final_map) {
                //     Ok(res) => {
                //         println!("Go program returned: {}", res);
                //         if res {
                //             println!("Go program returned true, exiting...");
                //             go_result = true;
                //         }
                //     }
                //     Err(e) => {
                //         println!("Error running Go program: {}, continuing...", e);
                //     }
                // }

                println!("Cpp result: {}, Go result: {}", cpp_result, go_result);
                if go_result || cpp_result {
                    break;
                }
            }
        } else {
            println!("No solution found within timeout");
        }
    }
}
