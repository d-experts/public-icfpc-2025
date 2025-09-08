use crate::{
    _PROBLEMS,
    client::ApiClient,
    ganba_dfs,
    omori2::{self, omori2_sa::SimulatedAnnealingSolver},
    utils::{Action, matrix_to_connections},
};

pub struct Graph {
    /// edges[i][j] は 部屋 i から扉 j で移動したときの部屋
    pub labels: Vec<usize>,
    pub doors: Vec<Vec<usize>>,
}

impl Graph {
    fn from_sasolver(solver: &SimulatedAnnealingSolver) -> Self {
        let map = solver.build_submission_map();
        let mut labels = map.rooms.clone();
        let mut doors = vec![vec![0; 6]; solver.num_rooms];

        for connection in map.connections.iter() {
            doors[connection.from.room][connection.from.door] = connection.to.room;
            doors[connection.to.room][connection.to.door] = connection.from.room;
        }

        Self { labels, doors }
    }

    fn print(&self) {
        println!("labels: {:?}", self.labels);
        for (i, row) in self.doors.iter().enumerate() {
            println!("doors {}: {:?}", i, row);
        }
    }
}

pub fn day2_solver() {
    let client = ApiClient::new();

    let problem = &_PROBLEMS[6];

    loop {
        let problem_name = problem.name;
        let select_result = client.select(problem_name);
        let N = problem.N;
        let N_layer = problem.layers;

        let solver = omori2::omori2_sa::identify_omori2(
            N / N_layer,
            problem.query.to_string(),
            problem.query.to_string(), // ERROR
        );

        if solver.is_none() {
            println!("Failed to identify omori2");
            continue;
        }

        let solver = solver.unwrap();

        let graph = Graph::from_sasolver(&solver);
        let result = build_query_tour(&graph);

        let matrix = process_query_tour(&graph, &result);

        for row in matrix.iter() {
            println!("{row:?}");
        }

        let answer = matrix_to_connections(&matrix);
        if answer.is_none() {
            println!("Failed to convert matrix to connections");
            continue;
        }
        let answer = answer.unwrap();
        let all_labels = vec![graph.labels.clone(); 2].concat();

        let guess_result = client.guess(all_labels, 0, answer);
        println!("guess_result: {guess_result:?}");

        if guess_result.unwrap().correct {
            println!("Congratulations! Your map was correct!");
            break;
        } else {
            println!("Map was incorrect. Try again!");
        }
    }
}

pub fn test_query_tour() {
    let graph = Graph {
        labels: vec![0, 1, 2, 3, 0, 1],
        doors: vec![
            vec![0, 0, 2, 4, 3, 5],
            vec![1, 2, 1, 1, 4, 5],
            vec![0, 1, 3, 4, 5, 3],
            vec![0, 2, 4, 4, 2, 3],
            vec![0, 1, 2, 3, 5, 3],
            vec![0, 1, 2, 4, 5, 5],
        ],
    };
    let result = build_query_tour(&graph);
    let score = evaluate_tour(&graph, &result);

    println!("score: {score}");
}

pub fn evaluate_tour(graph: &Graph, actions: &Vec<Action>) -> usize {
    let mut used_door = vec![vec![false; 6]; graph.doors.len()];
    let mut score = 0;
    let mut current_room = 0;

    for action in actions.iter() {
        match action {
            Action::Door(door) => {
                if !used_door[current_room][*door] {
                    score += 1;
                } else {
                    println!("door ({current_room}, {door}) is used twice");
                }
                used_door[current_room][*door] = true;
                current_room = graph.doors[current_room][*door];
            }
            Action::Mark(mark) => {}
        }
    }

    for row in used_door.iter() {
        println!("{row:?}");
    }

    score
}

pub fn process_query_tour(graph: &Graph, actions: &Vec<Action>) -> Vec<Vec<Option<usize>>> {
    let client = ApiClient::new();
    let query = Action::vec_to_str(actions);
    let result = client.explore(&vec![query]).unwrap().results[0].clone();

    let path = parse_query_result(graph, actions, &result);
    let N = graph.doors.len();

    let mut result: Vec<Vec<Option<usize>>> = vec![vec![None; 6]; N * 2];

    let mut visited = vec![0; N];
    visited[0] = 1;

    let mut current_room = 0; // 二層を区別しないときの部屋番号
    let mut current_layer = 0; // 二層を区別するときの層番号
    for e in path.iter() {
        let (room, door, label) = (e.room_id_on_plane, e.door_id, e.target_label);
        assert_eq!(room, current_room);

        visited[current_room] = 1;

        let next_room = graph.doors[current_room][door];
        // ラベルがオリジナルと一致しているとき、既に訪れていれば2層目、そうでなければ1層目
        // visited => labelが一緒

        if !(visited[next_room] != 0 || label == graph.labels[next_room]) {
            println!(
                "current_room: {current_room}, next_room: {next_room}, label: {label}, graph.labels[next_room]: {}",
                graph.labels[next_room]
            );
            println!("visited: {visited:?}");
        }
        let next_layer = if visited[next_room] == 1 {
            if label == graph.labels[next_room] {
                1
            } else {
                0
            }
        } else {
            0
        };

        result[current_room + N * current_layer][door] = Some(next_room + N * next_layer);
        result[current_room + N * (1 - current_layer)][door] =
            Some(next_room + N * (1 - next_layer));

        current_room = next_room;
        current_layer = next_layer;
    }

    result
}

#[derive(Debug, Clone)]
pub struct LayerPathResult {
    pub room_id_on_plane: usize,
    pub door_id: usize,
    pub target_label: usize,
}

/// 平面での部屋番号、部屋の扉、移動先のラベルの組合せにして手に入れる
pub fn parse_query_result(
    graph: &Graph,
    actions: &Vec<Action>,
    result: &Vec<usize>,
) -> Vec<LayerPathResult> {
    let mut res = vec![];
    let mut current_room = 0;

    for i in 0..actions.len() {
        match actions[i] {
            Action::Door(door) => {
                res.push(LayerPathResult {
                    room_id_on_plane: current_room,
                    door_id: door,
                    target_label: result[i + 1],
                });
                current_room = graph.doors[current_room][door];
            }
            Action::Mark(mark) => {}
        }
    }

    res
}

pub fn build_query_tour(graph: &Graph) -> Vec<Action> {
    let tour = euler_tour(graph);
    let mut result = vec![];

    let mut visited_rooms = vec![false; graph.labels.len()];
    let mut only_move = vec![];

    for (room, door) in tour {
        if !visited_rooms[room] {
            result.push(Action::Mark(3 - graph.labels[room]));
            visited_rooms[room] = true;
        }
        result.push(Action::Door(door));
        only_move.push(Action::Door(door));
    }

    // for mv in only_move.iter() { result.push(mv.clone());
    // }

    result
}

/// オイラー閉路を求める。スタート地点からスタートして、全ての辺を巡って戻ってくる。
pub fn euler_tour(graph: &Graph) -> Vec<(usize, usize)> {
    let mut used_door = vec![vec![false; 6]; graph.doors.len()];

    let mut tour = vec![];

    dfs(graph, &mut used_door, 0, 0, &mut tour);

    let mut prev_len = 0;
    while prev_len < tour.len() {
        prev_len = tour.len();
        for (i, (room, door)) in tour.clone().iter().enumerate() {
            let mut start_new_tour = false;
            for d in 0..6 {
                if !used_door[*room][d] {
                    start_new_tour = true;
                    break;
                }
            }

            if start_new_tour {
                let mut new_tour = vec![];
                dfs(graph, &mut used_door, *room, *room, &mut new_tour);

                tour.splice(i..i, new_tour);
                break;
            }
        }
    }
    for row in used_door.iter() {
        println!("{row:?}");
    }

    tour
}

/// current_room からスタートして戻ってくるパスを求める
pub fn dfs(
    graph: &Graph,
    used_door: &mut Vec<Vec<bool>>,
    start_room: usize,
    current_room: usize,
    tour: &mut Vec<(usize, usize)>,
) {
    if current_room == start_room && !tour.is_empty() {
        return;
    }

    for door in 0..6 {
        if used_door[current_room][door] {
            continue;
        }

        used_door[current_room][door] = true;
        tour.push((current_room, door));
        dfs(
            graph,
            used_door,
            start_room,
            graph.doors[current_room][door],
            tour,
        );
        break;
    }
}
