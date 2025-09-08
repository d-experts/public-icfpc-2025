use std::collections::HashMap;

use crate::{_PROBLEMS, client::ApiClient, utils::create_random_route};

pub fn fill_table_manual() {
    let client = ApiClient::new();
    let problem = &_PROBLEMS[1];
    let problem_name = problem.name;
    let select_result = client.select(problem_name);
    let v = problem.N;

    let random_route = create_random_route(v);

    let random_result = client.explore(&vec![random_route.clone()]).unwrap().results[0].clone();

    let mut table = vec![vec![HashMap::<usize, usize>::new(); 6]; v];

    for i in 0..random_route.len() {
        let room_id = random_result[i];
        let door_id = random_route.chars().nth(i).unwrap() as u8 - b'0';
        let next_room_id = random_result[i + 1];

        *table[room_id][door_id as usize]
            .entry(next_room_id)
            .or_insert(0) += 1;
    }

    for i in 0..4 {
        for d in 0..6 {
            println!("table[{}][{}]: {:?}", i, d, table[i][d]);
        }
    }
}
