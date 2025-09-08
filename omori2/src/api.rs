use std::{error::Error, fmt, time::Duration};

use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};

//const BASE_URL: &str = "https://31pwr5t6ij.execute-api.eu-west-2.amazonaws.com";
const BASE_URL: &str = "http://localhost:5000";
const TEAM_ID: &str = "";

#[derive(Debug, Clone)]
pub struct BaseMap {
    pub num_rooms: usize,
    pub starting_room: usize,
    // (from_room, door) -> to_room
    pub connections: HashMap<(usize, usize), usize>,
}
impl BaseMap {
    pub fn fill_missing_connections(&self) -> BaseMap {
        let mut full_connections = self.connections.clone();

        // --- 1. 貸し借り関係の解決 ---
        // kasikari[A][B] > 0 なら、AからBへの片道接続が kasikari[A][B] 本多いことを示す
        let mut kasikari = vec![vec![0i32; self.num_rooms]; self.num_rooms];
        for ((from, _), to) in &full_connections {
            kasikari[*from][*to] += 1;
            kasikari[*to][*from] -= 1;
        }

        for r1 in 0..self.num_rooms {
            for r2 in (r1 + 1)..self.num_rooms {
                // r1 -> r2 の片道が多い場合
                while kasikari[r1][r2] > 0 {
                    // r2 -> r1 の接続を追加して帳尻を合わせる
                    let available_doors: Vec<usize> = (0..6)
                        .filter(|&d| !full_connections.contains_key(&(r2, d)))
                        .collect();

                    if let Some(&door_to_use) = available_doors.first() {
                        println!(
                            "Balancing: Adding connection ({}, {}) -> {}",
                            r2, door_to_use, r1
                        );
                        full_connections.insert((r2, door_to_use), r1);
                        kasikari[r1][r2] -= 1;
                        kasikari[r2][r1] += 1;
                    } else {
                        // 空きドアがない場合は諦める（通常は起こらないはず）
                        println!(
                            "Warning: No available door to balance connection from {} to {}",
                            r2, r1
                        );
                        break;
                    }
                }
                // r2 -> r1 の片道が多い場合
                while kasikari[r2][r1] > 0 {
                    // r1 -> r2 の接続を追加
                    let available_doors: Vec<usize> = (0..6)
                        .filter(|&d| !full_connections.contains_key(&(r1, d)))
                        .collect();

                    if let Some(&door_to_use) = available_doors.first() {
                        println!(
                            "Balancing: Adding connection ({}, {}) -> {}",
                            r1, door_to_use, r2
                        );
                        full_connections.insert((r1, door_to_use), r2);
                        kasikari[r2][r1] -= 1;
                        kasikari[r1][r2] += 1;
                    } else {
                        println!(
                            "Warning: No available door to balance connection from {} to {}",
                            r1, r2
                        );
                        break;
                    }
                }
            }
        }

        // --- 2. 残りを自己ループで埋める ---
        for room in 0..self.num_rooms {
            for door in 0..6 {
                if !full_connections.contains_key(&(room, door)) {
                    println!("Filling self-loop: ({}, {}) -> {}", room, door, room);
                    full_connections.insert((room, door), room);
                }
            }
        }
        BaseMap {
            num_rooms: self.num_rooms,
            starting_room: self.starting_room,
            connections: full_connections,
        }
    }

    /// (room, door) -> room の単方向マップから、
    /// (room, door) <-> (room, door) の双方向ペアを構築する
    pub fn build_bidirectional_door_map(&self) -> HashMap<RoomAndDoor, RoomAndDoor> {
        let mut door_map = HashMap::default();
        let mut used_doors = HashSet::default();

        for from_room in 0..self.num_rooms {
            for from_door in 0..6 {
                let from_rd = RoomAndDoor {
                    room: from_room,
                    door: from_door,
                };
                if used_doors.contains(&from_rd) {
                    continue;
                }

                let to_room = self.connections.get(&(from_room, from_door)).unwrap();

                // to_room から from_room に戻る未使用のドアを探す
                let candidates: Vec<usize> = (0..6)
                    .filter(|&to_door| {
                        let to_rd = RoomAndDoor {
                            room: *to_room,
                            door: to_door,
                        };
                        !used_doors.contains(&to_rd)
                            && *self.connections.get(&(*to_room, to_door)).unwrap() == from_room
                    })
                    .collect();

                if candidates.is_empty() {
                    // これはロジックエラー。fill_missing_connectionsが正しければ起こらないはず
                    panic!(
                        "Could not find a returning door from {} to {}",
                        to_room, from_room
                    );
                }

                // 候補からランダムに1つ選ぶ (通常は1つのはず)
                let to_door = *candidates.choose(&mut thread_rng()).unwrap();
                let to_rd = RoomAndDoor {
                    room: *to_room,
                    door: to_door,
                };

                door_map.insert(from_rd, to_rd);
                door_map.insert(to_rd, from_rd);
                used_doors.insert(from_rd);
                used_doors.insert(to_rd);
            }
        }
        door_map
    }

    pub fn print_connections(&self) {
        println!("BaseMap Connections:");
        for from_room in 0..self.num_rooms {
            for door in 0..6 {
                if let Some(to_room) = self.connections.get(&(from_room, door)) {
                    println!("  (R{}, D{}) -> R{}", from_room, door, to_room);
                }
            }
        }
    }

    /// このBaseMapを元に、提出可能な完全な `api::Map` を構築する
    pub fn to_submission_map(&self) -> Map {
        // 1. 不完全な接続を補完する
        let full_connections = self.fill_missing_connections();

        // 2. 双方向のドアのペアを構築する
        let door_map = full_connections.build_bidirectional_door_map();

        // 3. 提出形式に変換する
        let rooms: Vec<usize> = (0..self.num_rooms).map(|r| r % 4).collect();

        let mut connections_vec = Vec::new();
        let mut processed_pairs = HashSet::default();

        for (from_rd, to_rd) in door_map.iter() {
            // (A,B)と(B,A)のペアを重複して追加しないようにする
            let key1 = (from_rd.room, from_rd.door, to_rd.room, to_rd.door);
            let key2 = (to_rd.room, to_rd.door, from_rd.room, from_rd.door);

            if !processed_pairs.contains(&key1) && !processed_pairs.contains(&key2) {
                connections_vec.push(Connection {
                    from: *from_rd,
                    to: *to_rd,
                });
                processed_pairs.insert(key1);
            }
        }

        Map {
            rooms,
            starting_room: self.starting_room,
            connections: connections_vec,
        }
    }
}

// main関数の上あたりに定義
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanStep {
    Move(usize),        // door
    ChangeLabel(usize), // new_label
}

// 経路計画文字列をパースする関数
pub fn parse_full_plan(plan_str: &str) -> (Vec<PlanStep>, String) {
    let mut full_plan = Vec::new();
    let mut simple_plan = String::new(); // SAで使うドアのみのplan
    let mut chars = plan_str.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '[' {
            if let Some(label_char) = chars.next() {
                if let Some(label) = label_char.to_digit(10) {
                    full_plan.push(PlanStep::ChangeLabel(label as usize));
                }
            }
            chars.next(); // ']' をスキップ
        } else if let Some(door) = c.to_digit(10) {
            full_plan.push(PlanStep::Move(door as usize));
            simple_plan.push(c);
        }
    }
    (full_plan, simple_plan)
}
// /select
#[derive(Serialize)]
pub struct SelectRequest<'a> {
    id: &'a str,
    #[serde(rename = "problemName")]
    problem_name: &'a str,
}

#[derive(Deserialize, Debug)]
pub struct SelectResponse {
    #[serde(rename = "problemName")]
    problem_name: String,
}

// /explore
#[derive(Serialize)]
pub struct ExploreRequest<'a> {
    id: &'a str,
    plans: &'a [String],
}

#[derive(Deserialize, Debug)]
pub struct ExploreResponse {
    pub results: Vec<Vec<usize>>,
    #[serde(rename = "queryCount")]
    pub query_count: i32,
}

// /guess
#[derive(Serialize, Debug)]
pub struct GuessRequest<'a> {
    id: &'a str,
    map: Map,
}

#[derive(Serialize, Debug)]
pub struct Map {
    pub rooms: Vec<usize>,
    #[serde(rename = "startingRoom")]
    pub starting_room: usize,
    pub connections: Vec<Connection>,
}

#[derive(Serialize, Debug)]
pub struct Connection {
    pub from: RoomAndDoor,
    pub to: RoomAndDoor,
}

#[derive(Serialize, PartialEq, Eq, Hash, Clone, Copy)]
pub struct RoomAndDoor {
    pub room: usize,
    pub door: usize,
}
// デバッグ出力用にfmt::Debugを実装
impl fmt::Debug for RoomAndDoor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "R{}.D{}", self.room, self.door)
    }
}

#[derive(Deserialize, Debug)]
pub struct GuessResponse {
    pub correct: bool,
}
use reqwest::blocking::Client;
// --- APIクライアント ---
pub struct ApiClient {
    client: Client,
    base_url: String,
}

impl ApiClient {
    pub fn new() -> Self {
        ApiClient {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            base_url: BASE_URL.to_string(),
        }
    }

    pub fn select_problem(&self, problem: &str) -> Result<SelectResponse, Box<dyn Error>> {
        let request_body = SelectRequest {
            id: TEAM_ID,
            problem_name: problem,
        };
        let response = self
            .client
            .post(format!("{}/select", self.base_url))
            .json(&request_body)
            .send()?
            .json::<SelectResponse>()?;
        Ok(response)
    }

    pub fn explore(&self, plans: &[String]) -> Result<ExploreResponse, Box<dyn Error>> {
        let request_body = ExploreRequest { id: TEAM_ID, plans };
        let response = self
            .client
            .post(format!("{}/explore", self.base_url))
            .json(&request_body)
            .send()?
            .json::<ExploreResponse>()?;
        Ok(response)
    }

    pub fn guess(&self, map: Map) -> Result<GuessResponse, Box<dyn Error>> {
        let request_body = GuessRequest { id: TEAM_ID, map };
        println!(
            "Guessing with map: {:?}",
            serde_json::to_string(&request_body)?
        );
        let response = self
            .client
            .post(format!("{}/guess", self.base_url))
            .json(&request_body)
            .send()?
            .json::<GuessResponse>()?;
        Ok(response)
    }
}
