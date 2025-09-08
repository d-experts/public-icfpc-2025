const NUM_PARALLEL_THREADS: usize = 1;
pub mod api;
pub mod dfs;
pub mod sa;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use rand::{Rng, thread_rng};

use crate::api::{PlanStep, parse_full_plan};
use crate::dfs::DfsSolver;
use crate::sa::SimulatedAnnealingSolver;

fn gen_random_string(alphabet: &str, length: usize, rng: &mut impl Rng) -> String {
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..alphabet.len());
            alphabet.chars().nth(idx).unwrap()
        })
        .collect()
}

fn run_simulated_annealing(
    plan: &str,
    results_str: &str,
    num_rooms: usize,
    full_plan_steps: Vec<PlanStep>,
    results_labeled_vec: Vec<usize>,
    layer_num: usize,
) -> Option<(usize, SimulatedAnnealingSolver)> {
    if NUM_PARALLEL_THREADS == 1 {
        // Single-threaded execution
        println!("Starting simulated annealing (single-threaded)...");
        let stop_signal = Arc::new(Mutex::new(false));
        let mut solver = SimulatedAnnealingSolver::new(plan, results_str, num_rooms);
        if solver
            .solve(
                0,
                stop_signal,
                full_plan_steps,
                results_labeled_vec,
                layer_num,
            )
            .is_some()
        {
            Some((0, solver))
        } else {
            None
        }
    } else {
        // Multi-threaded execution
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
            let plan_clone = plan.to_string();
            let results_str_clone = results_str.to_string();

            let full_plan_steps = full_plan_steps.clone();
            let results_labeled_vec = results_labeled_vec.clone();

            let handle = thread::spawn(move || {
                let mut solver =
                    SimulatedAnnealingSolver::new(&plan_clone, &results_str_clone, num_rooms);
                if let Some(_assignment) = solver.solve(
                    thread_id,
                    stop_signal,
                    full_plan_steps,
                    results_labeled_vec,
                    layer_num,
                ) {
                    tx.send((thread_id, solver)).ok();
                }
            });
            handles.push(handle);
        }

        // Drop the original sender so the receiver can detect when all threads are done
        drop(tx);

        // Wait for the first solution or all threads to finish
        let solution = rx.recv().ok();

        // Signal all threads to stop
        *stop_signal.lock().unwrap() = true;

        // Wait for all threads to finish
        for handle in handles {
            handle.join().ok();
        }

        solution
    }
}

fn main() {
    let api_client = api::ApiClient::new();
    let mut rng = thread_rng();

    loop {
        let num_base_rooms = 3;
        let layer_num = 1;
        let select_response = api_client.select_problem("probatio").unwrap();
        println!("Select response: {:?}", select_response);

        let num_sum_rooms = num_base_rooms * layer_num;
        let bb = if layer_num > 2 { 6 } else { 18 };
        let simple_plan = gen_random_string("012345", num_sum_rooms * bb, &mut thread_rng());
        let mut plan_with_labels = String::new();
        for (i, door_char) in simple_plan.chars().enumerate() {
            plan_with_labels.push_str(&format!("[{}]", rng.gen_range(0..4)));
            plan_with_labels.push(door_char);
        }

        println!("explore...");
        let explore_response = api_client
            .explore(&vec![simple_plan.clone(), plan_with_labels.clone()])
            .map_err(|e| {
                println!("Explore API error: {:?}", e);
                e
            })
            .unwrap();
        let results_simple_vec = explore_response.results[0].clone();
        let results_simple_str = results_simple_vec
            .iter()
            .map(|&x| std::char::from_digit(x as u32, 10).unwrap())
            .collect::<String>();

        let results_labeled_vec = explore_response.results[1].clone();
        let results_labeled_str = results_labeled_vec
            .iter()
            .map(|&x| std::char::from_digit(x as u32, 10).unwrap())
            .collect::<String>();

        println!("Simple Plan:    {}", simple_plan);
        println!("Simple Results: {}", results_simple_str);
        println!("Labeled Plan:   {}", plan_with_labels);
        println!("Labeled Results:{}", results_labeled_str);
        // 1c. 焼きなましで基本構造を決定
        let sa_solution = run_simulated_annealing(
            &simple_plan,
            &results_simple_str,
            num_base_rooms,
            parse_full_plan(&plan_with_labels).0,
            results_labeled_vec.clone(),
            layer_num,
        );

        if let Some((_thread_id, sa_solver)) = sa_solution {
            println!("\n★ SA found a potential base structure! ★");
            let base_map: api::BaseMap = sa_solver.build_base_map();
            let assignment_str = sa_solver
                .assignment
                .iter()
                .map(|&x| x.to_string())
                .collect::<String>();
            println!("SA Assignment:  {}", assignment_str);
            base_map.print_connections();
            assert!(sa_solver.is_valid_assignment());

            println!("\n--- Phase 2: Layer Identification Exploration ---");

            // 2c. DFSで階層を解決
            println!("\n--- Running DFS to resolve layers ---");
            let full_plan_steps: Vec<api::PlanStep> = parse_full_plan(&plan_with_labels).0;

            let mut dfs_solver = DfsSolver::new(base_map, full_plan_steps, results_labeled_vec, 2);

            if let Some(solution) = dfs_solver.solve() {
                println!("\n★ DFS successfully found a consistent path through layers! ★");

                println!("Submitting the guess...");
                let guess_res = api_client.guess(solution).unwrap();
                println!("Guess result: correct = {}", guess_res.correct);

                if guess_res.correct {
                    println!("★★★ Congratulations! Your map was correct! ★★★");
                    break;
                } else {
                    println!("Map was incorrect. Retrying the whole process...");
                }
            } else {
                println!("DFS failed. The base structure from SA might be incorrect. Retrying...");
            }
        } else {
            println!("SA failed to find a solution. Retrying...");
        }
    }
}
