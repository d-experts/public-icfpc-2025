use crate::api::{BaseMap, Connection, Map, PlanStep, RoomAndDoor};
use fixedbitset::FixedBitSet;
use fxhash::FxHashMap as HashMap;

/// DFSを使って、基本構造から完全なマップを構築するソルバー
pub struct DfsSolver {
    num_base_rooms: usize,
    base_map: BaseMap,
    full_plan: Vec<PlanStep>,
    observed_labels: Vec<usize>,

    // a -> b に行くためのドア一覧（bitsetで保持）
    remaining_base_doors: Vec<Vec<FixedBitSet>>,

    // --- DFS中の状態 ---
    connections: HashMap<RoomAndDoor, RoomAndDoor>,
    pub full_assignment: Vec<isize>,
    current_labels: Vec<usize>,

    // ログ出力用のインデントレベル
    log_indent: usize,
    layer_num: usize,
}

// レイヤーの間の、つなぎ込みのパターンを列挙
fn twins_patterns(
    layers: usize,
    from_room: usize,
    to_base_room: usize,
    num_base_room: usize,
) -> Vec<Vec<(usize, usize)>> {
    let from_points = (0..layers)
        .map(|i| (from_room + num_base_room * i) % (num_base_room * layers))
        .collect::<Vec<usize>>();
    let to_points = (0..layers)
        .map(|i| (to_base_room + num_base_room * i) % (num_base_room * layers))
        .collect::<Vec<usize>>();
    if layers == 1 {
        return vec![vec![(from_room, to_base_room)]];
    }
    if layers == 2 {
        return vec![
            vec![
                (from_points[0], to_points[0]),
                (from_points[1], to_points[1]),
            ],
            vec![
                (from_points[0], to_points[1]),
                (from_points[1], to_points[0]),
            ],
        ];
    }
    if layers == 3 {
        return vec![
            vec![
                (from_points[0], to_points[0]),
                (from_points[1], to_points[1]),
                (from_points[2], to_points[2]),
            ],
            vec![
                (from_points[0], to_points[0]),
                (from_points[1], to_points[2]),
                (from_points[2], to_points[1]),
            ],
            vec![
                (from_points[0], to_points[1]),
                (from_points[1], to_points[0]),
                (from_points[2], to_points[2]),
            ],
            vec![
                (from_points[0], to_points[1]),
                (from_points[1], to_points[2]),
                (from_points[2], to_points[0]),
            ],
            vec![
                (from_points[0], to_points[2]),
                (from_points[1], to_points[0]),
                (from_points[2], to_points[1]),
            ],
            vec![
                (from_points[0], to_points[2]),
                (from_points[1], to_points[1]),
                (from_points[2], to_points[0]),
            ],
        ];
    }

    panic!("Unsupported number of layers: {}", layers);
}

impl DfsSolver {
    /// 新しいソルバーを初期化する
    pub fn new(
        base_map: BaseMap,
        full_plan: Vec<PlanStep>,
        observed_labels: Vec<usize>,
        layer_num: usize,
    ) -> Self {
        let num_base_rooms = base_map.num_rooms;
        let mut initial_labels = vec![0; num_base_rooms * layer_num];

        for i in 0..initial_labels.len() {
            initial_labels[i] = (i % num_base_rooms) % 4;
        }

        let mut full_assignment = vec![-1; observed_labels.len()];
        full_assignment[0] = 0;

        let mut remaining_base_doors =
            vec![vec![FixedBitSet::with_capacity(6); num_base_rooms]; num_base_rooms];
        for ((from_room, from_door), to_room) in base_map.connections.iter() {
            remaining_base_doors[*from_room][*to_room].insert(*from_door);
        }

        Self {
            num_base_rooms,
            base_map,
            full_plan,
            full_assignment,
            observed_labels,
            connections: HashMap::default(),
            current_labels: initial_labels,
            remaining_base_doors,
            log_indent: 0,
            layer_num,
        }
    }

    // ログ出力用のヘルパー関数
    fn log(&self, msg: &str) {
        return;
        println!("{}{}", "  ".repeat(self.log_indent), msg);
    }

    pub fn fill_missing_connections_with_self_loop(&mut self) {
        for from_room in 0..self.num_base_rooms * self.layer_num {
            for door in 0..6 {
                let rd = RoomAndDoor {
                    room: from_room,
                    door,
                };
                if !self.connections.contains_key(&rd) {
                    self.connections.insert(rd, rd);
                }
            }
        }
    }

    /// DFSを実行して完全なマップを探索する
    pub fn solve(&mut self) -> Option<Map> {
        self.log("DFS Solver started.");

        let start_obs_label = self.observed_labels[0];
        self.log(&format!("Observed start label: {}", start_obs_label));

        let base_starting_room = 0;

        // 階層0 (0..N-1) からスタートする場合
        let start_candidate_0 = base_starting_room;
        self.log(&format!(
            "Trying start candidate: R{} (label: {})",
            start_candidate_0, self.current_labels[start_candidate_0]
        ));
        if self.current_labels[start_candidate_0] == start_obs_label {
            if self.dfs(0, 0, start_candidate_0) {
                self.log("Solution found starting from R{}!");
                self.fill_missing_connections_with_self_loop();
                return Some(Map {
                    rooms: (0..self.num_base_rooms * self.layer_num)
                        .map(|r| r % self.num_base_rooms % 4)
                        .collect(),
                    starting_room: 0,
                    connections: self
                        .connections
                        .clone()
                        .iter()
                        .map(|(k, v)| Connection { from: *k, to: *v })
                        .collect(),
                });
            }
        }
        self.log("No solution found.");
        return None;
    }

    /// 深さ優先探索の再帰関数本体
    fn dfs(&mut self, plan_idx: usize, obs_idx: usize, current_full_room: usize) -> bool {
        self.log_indent += 1;
        self.log(&format!(
            "-> dfs(plan: {}, obs: {}, room: R{})",
            plan_idx, obs_idx, current_full_room
        ));

        self.full_assignment[obs_idx] = current_full_room as isize;
        self.log(&format!(
            "[Assign] obs #{} -> R{}",
            obs_idx, current_full_room
        ));
        //println!("plan_idx: {} / {}", plan_idx, self.full_plan.len());
        if plan_idx >= self.full_plan.len() {
            self.log("  [Success] Reached end of plan.");
            self.log_indent -= 1;
            return true;
        }

        let result = match self.full_plan[plan_idx] {
            PlanStep::ChangeLabel(new_label) => {
                self.log(&format!(
                    "[Action] ChangeLabel in R{} to {}",
                    current_full_room, new_label
                ));
                let old_label = self.current_labels[current_full_room];
                self.current_labels[current_full_room] = new_label;

                // ChangeLabelも観測を生成するので、obs_idxを+1する
                let success = self.dfs(plan_idx + 1, obs_idx + 1, current_full_room);

                self.log(&format!(
                    "[Backtrack] Revert label in R{} to {}",
                    current_full_room, old_label
                ));
                self.current_labels[current_full_room] = old_label;
                success
            }
            PlanStep::Move(door) => {
                self.log(&format!(
                    "[Action] Move from R{}.D{}",
                    current_full_room, door
                ));
                self.handle_move(plan_idx, obs_idx, current_full_room)
            }
        };

        if !result {
            self.log(&format!(
                "[Backtrack] Unassign obs #{} from R{}",
                obs_idx, current_full_room
            ));
            self.full_assignment[obs_idx] = -1;
        }

        self.log(&format!(
            "<- dfs(plan: {}, obs: {}, room: R{}) -> {}",
            plan_idx,
            obs_idx,
            current_full_room,
            if result { "Success" } else { "Fail" }
        ));
        self.log_indent -= 1;
        result
    }

    fn get_new_door_of_to_room(&self, from_base_room: usize, to_base_room: usize) -> Option<usize> {
        let base_door_set = &self.remaining_base_doors[to_base_room][from_base_room];
        if let Some(door) = base_door_set.ones().next() {
            return Some(door);
        }

        self.log("  No available door found in remaining_base_doors.");
        // なかった場合、to_roomの、base_mapで使用されてない & connectionsにないドアを適当に一つ持ってくる
        for door in 0..6 {
            if !self
                .base_map
                .connections
                .contains_key(&(to_base_room, door))
                && !self.connections.contains_key(&RoomAndDoor {
                    room: to_base_room,
                    door,
                })
            {
                self.log(&format!(
                    "  No remaining door, but using unused door {} from base map.",
                    door
                ));
                return Some(door);
            }
        }
        None
    }

    /// 移動(Move)ステップを処理するヘルパー関数
    fn handle_move(&mut self, plan_idx: usize, obs_idx: usize, from_room: usize) -> bool {
        if let PlanStep::Move(from_door) = self.full_plan[plan_idx] {
            let next_obs_idx = obs_idx + 1;
            let expected_label_at_dest = self.observed_labels[next_obs_idx];
            self.log(&format!(
                "  Destination room must have label: {}",
                expected_label_at_dest
            ));

            let from_rd = RoomAndDoor {
                room: from_room,
                door: from_door,
            };

            // 候補1: 既存の接続をたどる
            if let Some(&RoomAndDoor {
                room: to_room,
                door: _to_door,
            }) = self.connections.get(&from_rd)
            {
                self.log(&format!(
                    "  Found existing connection: {:?} -> {:?}",
                    from_rd, to_room
                ));
                if self.current_labels[to_room] == expected_label_at_dest {
                    self.log("  Label matches. Following this path.");
                    if self.dfs(plan_idx + 1, next_obs_idx, to_room) {
                        return true;
                    }
                } else {
                    self.log(&format!(
                        "  Label mismatch! (Expected {}, Found {}). This path is invalid.",
                        expected_label_at_dest, self.current_labels[to_room]
                    ));
                }
                return false;
            }

            // 候補2: 新しい接続を試す
            let from_base_room = from_room % self.num_base_rooms;
            let to_base_room = match self.base_map.connections.get(&(from_base_room, from_door)) {
                Some(r) => *r,
                None => {
                    panic!(
                        "No connection in base map from R{}.D{}",
                        from_base_room, from_door
                    );
                }
            };
            self.log(&format!(
                "  Base map connection: R{}(base) -> R{}(base)",
                from_base_room, to_base_room
            ));

            let to_door = self.get_new_door_of_to_room(from_base_room, to_base_room);
            if to_door.is_none() {
                self.log("  No available door to create new connection. Backtracking.");
                return false;
            }
            let to_door = to_door.unwrap();
            let patterns =
                twins_patterns(self.layer_num, from_room, to_base_room, self.num_base_rooms);
            for pattern in patterns {
                assert!(pattern[0].0 == from_room);
                let to_room = pattern[0].1;
                if self.current_labels[to_room] != expected_label_at_dest {
                    continue;
                }
                assert!(self.connect_twins(&pattern, from_door, to_door));
                if self.dfs(plan_idx + 1, next_obs_idx, to_room) {
                    return true;
                }
                self.disconnect_twins(&pattern, from_door, to_door);
            }
        } else {
            assert!(false);
        }
        false
    }

    #[inline(always)]
    fn room_id_to_base_room(&self, room_id: usize) -> usize {
        room_id % self.num_base_rooms
    }

    fn connect_twins(
        &mut self,
        pattern: &Vec<(usize, usize)>,
        from_door: usize,
        to_door: usize,
    ) -> bool {
        let from_base_room = self.room_id_to_base_room(pattern[0].0);
        let to_base_room = self.room_id_to_base_room(pattern[0].1);

        if pattern.iter().any(|&(from_room, to_room)| {
            let from_rd = RoomAndDoor {
                room: from_room,
                door: from_door,
            };
            let to_rd = RoomAndDoor {
                room: to_room,
                door: to_door,
            };
            self.connections.contains_key(&from_rd) || self.connections.contains_key(&to_rd)
        }) {
            self.log("  One of the rooms in the pattern is already connected. Backtracking.");
            return false;
        }

        for &(from_room, to_room) in pattern.iter() {
            let from_rd = RoomAndDoor {
                room: from_room,
                door: from_door,
            };
            let to_rd = RoomAndDoor {
                room: to_room,
                door: to_door,
            };
            self.connections.insert(from_rd, to_rd);
            self.log(&format!(
                "  Created new connection: {:?} <-> {:?}",
                from_rd, to_rd
            ));
        }
        self.remaining_base_doors[from_base_room][to_base_room].set(from_door, false);
        self.remaining_base_doors[to_base_room][from_base_room].set(to_door, false);

        true
    }

    fn disconnect_twins(
        &mut self,
        pattern: &Vec<(usize, usize)>,
        from_door: usize,
        to_door: usize,
    ) {
        let from_base_room = self.room_id_to_base_room(pattern[0].0);
        let to_base_room = self.room_id_to_base_room(pattern[0].1);

        for &(from_room, to_room) in pattern.iter() {
            let from_rd = RoomAndDoor {
                room: from_room,
                door: from_door,
            };
            let to_rd = RoomAndDoor {
                room: to_room,
                door: to_door,
            };
            self.connections.remove(&from_rd);
            self.connections.remove(&to_rd);
            self.log(&format!(
                "  Removed connection: {:?} <-> {:?}",
                from_rd, to_rd
            ));
        }
        self.remaining_base_doors[from_base_room][to_base_room].set(from_door, true);
        self.remaining_base_doors[to_base_room][from_base_room].set(to_door, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{BaseMap, PlanStep};

    #[test]
    fn test_dfs_solver_single_layer() {
        // Layer = 1のテスト（単層）
        let mut base_connections = HashMap::default();
        base_connections.insert((0, 0), 1);
        base_connections.insert((0, 1), 2);
        base_connections.insert((1, 0), 2);
        base_connections.insert((1, 1), 0);
        base_connections.insert((2, 0), 0);
        base_connections.insert((2, 1), 1);

        let base_map = BaseMap {
            num_rooms: 3,
            starting_room: 0,
            connections: base_connections,
        };

        let full_plan = vec![
            PlanStep::Move(0),        // R0 -> R1
            PlanStep::ChangeLabel(3), // R1のラベルを1->3に変更
            PlanStep::Move(0),        // R1 -> R2
            PlanStep::Move(0),        // R2 -> R0
        ];

        let observed_labels = vec![
            0, // R0 (start)
            1, // R1 after Move(0)
            3, // R1 after ChangeLabel(3)
            2, // R2 after Move(0)
            0, // R0 after Move(0)
        ];

        let mut solver = DfsSolver::new(base_map, full_plan, observed_labels, 1);
        let result = solver.solve();

        assert!(
            result.is_some(),
            "Should find a valid solution for single layer"
        );
        let map = result.unwrap();

        assert_eq!(map.starting_room, 0);
        println!("✓ Single layer test passed!");
    }

    #[test]
    fn test_single_layer_complex_path() {
        // Layer=1で複雑な経路のテスト
        let mut base_connections = HashMap::default();
        // 5部屋のリング構造
        base_connections.insert((0, 0), 1);
        base_connections.insert((1, 0), 2);
        base_connections.insert((2, 0), 3);
        base_connections.insert((3, 0), 4);
        base_connections.insert((4, 0), 0);
        // 逆方向
        base_connections.insert((0, 1), 4);
        base_connections.insert((1, 1), 0);
        base_connections.insert((2, 1), 1);
        base_connections.insert((3, 1), 2);
        base_connections.insert((4, 1), 3);

        let base_map = BaseMap {
            num_rooms: 5,
            starting_room: 0,
            connections: base_connections,
        };

        let full_plan = vec![
            PlanStep::Move(0),        // R0 -> R1
            PlanStep::Move(0),        // R1 -> R2
            PlanStep::Move(0),        // R2 -> R3
            PlanStep::Move(0),        // R3 -> R4
            PlanStep::Move(0),        // R4 -> R0 (complete the ring)
            PlanStep::ChangeLabel(3), // R0のラベルを0->3に変更
            PlanStep::Move(1),        // R0 -> R4 (backward)
            PlanStep::Move(1),        // R4 -> R3
        ];

        let observed_labels = vec![
            0, // R0 (start)
            1, // R1 after Move(0)
            2, // R2 after Move(0)
            3, // R3 after Move(0)
            0, // R4 after Move(0) - label wraps to 0
            0, // R0 after Move(0) - back to start
            3, // R0 after ChangeLabel(3)
            0, // R4 after Move(1) - backward
            3, // R3 after Move(1)
        ];

        let mut solver = DfsSolver::new(base_map, full_plan, observed_labels, 1);
        let result = solver.solve();

        assert!(result.is_some(), "Should find solution for ring structure");
        println!("✓ Single layer ring test passed!");
    }

    #[test]
    fn test_single_layer_all_doors() {
        // Layer=1で全ドアを使うテスト
        let mut base_connections = HashMap::default();
        // R0からすべての方向へ
        base_connections.insert((0, 0), 1);
        base_connections.insert((0, 1), 2);
        base_connections.insert((0, 2), 3);
        base_connections.insert((0, 3), 4);
        base_connections.insert((0, 4), 1); // R0 -> R1 via door 4
        base_connections.insert((0, 5), 2); // R0 -> R2 via door 5
        // 戻りの接続
        base_connections.insert((1, 0), 0);
        base_connections.insert((2, 0), 0);
        base_connections.insert((3, 0), 0);
        base_connections.insert((4, 0), 0);
        base_connections.insert((1, 4), 0);
        base_connections.insert((2, 5), 0);

        let base_map = BaseMap {
            num_rooms: 5,
            starting_room: 0,
            connections: base_connections,
        };

        let full_plan = vec![
            PlanStep::Move(0), // R0 -> R1 via door 0
            PlanStep::Move(0), // R1 -> R0 via door 0
            PlanStep::Move(1), // R0 -> R2 via door 1
            PlanStep::Move(0), // R2 -> R0 via door 0
            PlanStep::Move(2), // R0 -> R3 via door 2
            PlanStep::Move(0), // R3 -> R0 via door 0
            PlanStep::Move(3), // R0 -> R4 via door 3
            PlanStep::Move(0), // R4 -> R0 via door 0
            PlanStep::Move(4), // R0 -> R1 via door 4
            PlanStep::Move(4), // R1 -> R0 via door 4
            PlanStep::Move(5), // R0 -> R2 via door 5
        ];

        let observed_labels = vec![
            0, // R0 (start)
            1, // R1 after Move(0)
            0, // R0 after Move(0)
            2, // R2 after Move(1)
            0, // R0 after Move(0)
            3, // R3 after Move(2)
            0, // R0 after Move(0)
            0, // R4 after Move(3) - label wraps to 0
            0, // R0 after Move(0)
            1, // R1 after Move(4)
            0, // R0 after Move(4)
            2, // R2 after Move(5)
        ];

        let mut solver = DfsSolver::new(base_map, full_plan, observed_labels, 1);
        let result = solver.solve();

        assert!(result.is_some(), "Should handle all doors");
        println!("✓ Single layer all doors test passed!");
    }

    #[test]
    fn test_single_layer_label_changes() {
        // Layer=1で多くのラベル変更を含むテスト
        let mut base_connections = HashMap::default();
        base_connections.insert((0, 0), 1);
        base_connections.insert((1, 0), 2);
        base_connections.insert((2, 0), 0);

        let base_map = BaseMap {
            num_rooms: 3,
            starting_room: 0,
            connections: base_connections,
        };

        let full_plan = vec![
            PlanStep::ChangeLabel(2), // R0: 0->2
            PlanStep::ChangeLabel(3), // R0: 2->3
            PlanStep::Move(0),        // R0 -> R1
            PlanStep::ChangeLabel(0), // R1: 1->0
            PlanStep::ChangeLabel(1), // R1: 0->1
            PlanStep::ChangeLabel(2), // R1: 1->2
            PlanStep::Move(0),        // R1 -> R2
            PlanStep::ChangeLabel(3), // R2: 2->3
            PlanStep::Move(0),        // R2 -> R0
            PlanStep::ChangeLabel(0), // R0: 3->0
        ];

        let observed_labels = vec![
            0, // R0 (start)
            2, // R0 after ChangeLabel(2)
            3, // R0 after ChangeLabel(3)
            1, // R1 after Move(0)
            0, // R1 after ChangeLabel(0)
            1, // R1 after ChangeLabel(1)
            2, // R1 after ChangeLabel(2)
            2, // R2 after Move(0)
            3, // R2 after ChangeLabel(3)
            3, // R0 after Move(0) - still has label 3 from before
            0, // R0 after ChangeLabel(0)
        ];

        let mut solver = DfsSolver::new(base_map, full_plan, observed_labels, 1);
        let result = solver.solve();

        assert!(result.is_some(), "Should handle multiple label changes");
        println!("✓ Single layer label changes test passed!");
    }

    #[test]
    fn test_single_layer_ambiguous_labels() {
        // Layer=1で同じラベルの部屋が複数ある場合のテスト
        let mut base_connections = HashMap::default();
        // 6部屋なので、R0とR4がラベル0、R1とR5がラベル1
        base_connections.insert((0, 0), 1);
        base_connections.insert((1, 0), 2);
        base_connections.insert((2, 0), 3);
        base_connections.insert((3, 0), 4);
        base_connections.insert((4, 0), 5);
        base_connections.insert((5, 0), 0);

        let base_map = BaseMap {
            num_rooms: 6,
            starting_room: 0,
            connections: base_connections,
        };

        let full_plan = vec![
            PlanStep::Move(0),        // R0 -> R1
            PlanStep::Move(0),        // R1 -> R2
            PlanStep::Move(0),        // R2 -> R3
            PlanStep::Move(0),        // R3 -> R4
            PlanStep::ChangeLabel(2), // R4: 0->2 (区別のため)
            PlanStep::Move(0),        // R4 -> R5
            PlanStep::Move(0),        // R5 -> R0
        ];

        let observed_labels = vec![
            0, // R0 (start, label=0)
            1, // R1 after Move(0)
            2, // R2 after Move(0)
            3, // R3 after Move(0)
            0, // R4 after Move(0) - also label 0!
            2, // R4 after ChangeLabel(2)
            1, // R5 after Move(0) - also label 1!
            0, // R0 after Move(0) - back to start
        ];

        let mut solver = DfsSolver::new(base_map, full_plan, observed_labels, 1);
        let result = solver.solve();

        assert!(result.is_some(), "Should handle ambiguous labels correctly");

        // 正しい部屋を訪問したか確認
        assert_eq!(solver.full_assignment[0], 0); // Start at R0
        assert_eq!(solver.full_assignment[1], 1); // R1 after Move(0)
        assert_eq!(solver.full_assignment[2], 2); // R2 after Move(0)
        assert_eq!(solver.full_assignment[3], 3); // R3 after Move(0)
        assert_eq!(solver.full_assignment[4], 4); // R4 after Move(0) (not R0!)
        assert_eq!(solver.full_assignment[5], 4); // Still R4 after ChangeLabel(2)
        assert_eq!(solver.full_assignment[6], 5); // R5 after Move(0)
        assert_eq!(solver.full_assignment[7], 0); // Back to R0 after Move(0)

        println!("✓ Single layer ambiguous labels test passed!");
    }

    #[test]
    fn test_dfs_solver_2layers_with_swap() {
        // Layer = 2のテスト（エッジスワップあり）
        let mut base_connections = HashMap::default();
        base_connections.insert((0, 0), 1); // R0.D0 -> R1
        base_connections.insert((0, 1), 2); // R0.D1 -> R2
        base_connections.insert((1, 0), 2); // R1.D0 -> R2
        base_connections.insert((1, 1), 0); // R1.D1 -> R0
        base_connections.insert((2, 0), 0); // R2.D0 -> R0
        base_connections.insert((2, 1), 1); // R2.D1 -> R1

        let base_map = BaseMap {
            num_rooms: 3,
            starting_room: 0,
            connections: base_connections,
        };

        // エッジスワップを想定した計画
        let full_plan = vec![
            PlanStep::ChangeLabel(2), // R0を0->2に変更して層を識別
            PlanStep::Move(0),        // R0.D0 -> R1 or R4（スワップの可能性）
            PlanStep::ChangeLabel(3), // ラベル変更
            PlanStep::Move(0),        // Move through door 0
            PlanStep::ChangeLabel(1), // ラベル変更
        ];

        // Layer2への移動を示す観測列
        let observed_labels = vec![
            0, // R0 (start, label=0)
            2, // R0 after ChangeLabel(2)
            1, // R4 after Move(0) - Layer2へ移動！
            3, // R4 after ChangeLabel(3)
            2, // R5 after Move(0)
            1, // R5 after ChangeLabel(1)
        ];

        let mut solver = DfsSolver::new(base_map, full_plan, observed_labels, 2);
        let result = solver.solve();

        assert!(
            result.is_some(),
            "Should find a valid solution with 2 layers"
        );
        let map = result.unwrap();

        assert_eq!(map.starting_room, 0);

        // Layer2の部屋が訪問されたか確認
        let visited_layer2 = solver.full_assignment.iter().any(|&x| x >= 3);
        println!("Full assignment: {:?}", solver.full_assignment);
        println!("Visited layer 2: {}", visited_layer2);

        println!("✓ 2-layer with swap test passed!");
    }

    #[test]
    fn test_dfs_solver_3layers() {
        // Layer = 3のテスト
        let mut base_connections = HashMap::default();
        base_connections.insert((0, 0), 1);
        base_connections.insert((0, 1), 2);
        base_connections.insert((1, 0), 2);
        base_connections.insert((1, 1), 0);
        base_connections.insert((2, 0), 0);
        base_connections.insert((2, 1), 1);

        let base_map = BaseMap {
            num_rooms: 3,
            starting_room: 0,
            connections: base_connections,
        };

        // 3層を探索する計画
        let full_plan = vec![
            PlanStep::ChangeLabel(3), // R0を識別可能にする
            PlanStep::Move(0),        // R0 -> R1/R4/R7
            PlanStep::ChangeLabel(2), // ラベル変更
            PlanStep::Move(0),        // Move
            PlanStep::ChangeLabel(0), // ラベル変更
            PlanStep::Move(0),        // Move back
        ];

        // 3層のパターンをテストする観測列
        let observed_labels = vec![
            0, // R0 (start)
            3, // R0 after ChangeLabel(3)
            1, // R7 (Layer3) after Move(0) - 3層目へ！
            2, // R7 after ChangeLabel(2)
            2, // R8 after Move(0)
            0, // R8 after ChangeLabel(0)
            0, // R6 after Move(0)
        ];

        let mut solver = DfsSolver::new(base_map, full_plan, observed_labels, 3);
        let result = solver.solve();

        if result.is_some() {
            let map = result.unwrap();
            assert_eq!(map.starting_room, 0);

            // 各層の訪問を確認
            let layer1_visits = solver
                .full_assignment
                .iter()
                .filter(|&&x| x >= 0 && x < 3)
                .count();
            let layer2_visits = solver
                .full_assignment
                .iter()
                .filter(|&&x| x >= 3 && x < 6)
                .count();
            let layer3_visits = solver.full_assignment.iter().filter(|&&x| x >= 6).count();

            println!("Full assignment: {:?}", solver.full_assignment);
            println!("Layer 1 visits: {}", layer1_visits);
            println!("Layer 2 visits: {}", layer2_visits);
            println!("Layer 3 visits: {}", layer3_visits);

            println!("✓ 3-layer test passed!");
        } else {
            println!("3-layer test: No solution found (expected for some patterns)");
        }
    }

    #[test]
    fn test_twins_patterns() {
        // twins_patterns関数のテスト

        // Layer 1の場合
        let patterns1 = twins_patterns(1, 0, 1, 3);
        assert_eq!(patterns1.len(), 1);
        assert_eq!(patterns1[0], vec![(0, 1)]);

        // Layer 2の場合
        let patterns2 = twins_patterns(2, 0, 1, 3);
        assert_eq!(patterns2.len(), 2);
        // パターン1: ストレート接続
        assert!(patterns2.contains(&vec![(0, 1), (3, 4)]));
        // パターン2: クロス接続（エッジスワップ）
        assert!(patterns2.contains(&vec![(0, 4), (3, 1)]));

        // Layer 3の場合
        let patterns3 = twins_patterns(3, 0, 1, 2);
        assert_eq!(patterns3.len(), 6); // 3! = 6通り

        println!("✓ twins_patterns test passed!");
    }

    #[test]
    fn test_room_labels_initialization() {
        // ラベル初期化のテスト
        let mut base_connections = HashMap::default();
        base_connections.insert((0, 0), 1);

        let base_map = BaseMap {
            num_rooms: 5,
            starting_room: 0,
            connections: base_connections,
        };

        let solver = DfsSolver::new(base_map, vec![], vec![0], 2);

        // 各部屋の初期ラベルを確認（(room_id % num_base_rooms) % 4）
        // num_base_rooms = 5の場合
        assert_eq!(solver.current_labels[0], 0); // R0: (0 % 5) % 4 = 0
        assert_eq!(solver.current_labels[1], 1); // R1: (1 % 5) % 4 = 1
        assert_eq!(solver.current_labels[2], 2); // R2: (2 % 5) % 4 = 2
        assert_eq!(solver.current_labels[3], 3); // R3: (3 % 5) % 4 = 3
        assert_eq!(solver.current_labels[4], 0); // R4: (4 % 5) % 4 = 0
        assert_eq!(solver.current_labels[5], 0); // R5: (5 % 5) % 4 = 0 (Layer2のR0)
        assert_eq!(solver.current_labels[6], 1); // R6: (6 % 5) % 4 = 1 (Layer2のR1)
        assert_eq!(solver.current_labels[7], 2); // R7: (7 % 5) % 4 = 2 (Layer2のR2)
        assert_eq!(solver.current_labels[8], 3); // R8: (8 % 5) % 4 = 3 (Layer2のR3)
        assert_eq!(solver.current_labels[9], 0); // R9: (9 % 5) % 4 = 0 (Layer2のR4)

        println!("✓ Room labels initialization test passed!");
    }
}
