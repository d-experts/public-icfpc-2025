use std::collections::HashMap;

use crate::{
    client::ApiClient,
    utils::{Action, all_doors, query_result_to_string},
};

/// 適当な signature を使い、まずは同定を行う
/// その後、その signature を使ってどの部屋に行くか決める
pub fn greedy_solver(n: usize) -> Vec<Vec<usize>> {
    let mut signature_query = "1111111";
    let mut doors = vec![vec![0; 6]; n];

    vec![]
}

#[derive(Debug, Clone)]
pub struct Identity {
    /// スタートから route に従ったときの signature
    pub label: usize,
    pub route: Vec<Action>,
    pub signature: String,
}

pub fn identify(n: usize, signature_query: &str) -> HashMap<String, Identity> {
    let mut identities = HashMap::new();

    identities
}

pub fn identify_one_step(
    n: usize,
    signature_query: &str,
    identities: &mut HashMap<String, Identity>,
    client: &ApiClient,
) {
    let all_doors = all_doors();

    let mut queries = vec![];
    let mut routes = vec![];
    for identity in identities.values() {
        let route = identity.route.clone();
        for action in &all_doors {
            let new_route = [route.clone(), vec![action.clone()]].concat();
            let new_route_str = Action::vec_to_str(&new_route);

            let query = new_route_str + signature_query + signature_query;
            queries.push(query);
            routes.push(new_route);
        }
    }

    let results = client.explore(&queries).unwrap().results;
    for (i, result) in results.iter().enumerate() {
        let result_str = query_result_to_string(&result);
        let route = routes[i].clone();
        let signature = result_str[route.len()..].to_string();

        identities.insert(
            signature.clone(),
            Identity {
                label: result[route.len()],
                route: route.clone(),
                signature: signature.clone(),
            },
        );
    }
}
