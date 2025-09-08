use anyhow::Result;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// const BASE_URL: &str = "https://31pwr5t6ij.execute-api.eu-west-2.amazonaws.com/";
const BASE_URL: &str = "http://localhost:5000";
const ID: &str = "";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectRequest {
    id: String,
    problem_name: String,
}

#[derive(Debug, Serialize)]
pub struct ExploreRequest {
    id: String,
    plans: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExploreResult {
    pub results: Vec<Vec<usize>>,
}

#[derive(Debug, Serialize)]
pub struct GuessRequest {
    id: String,
    map: Map,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GuessResult {
    pub correct: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Map {
    pub rooms: Vec<usize>,
    pub starting_room: usize,
    pub connections: Vec<Connection>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Connection {
    pub from: Door,
    pub to: Door,
}

#[derive(Debug, Clone, Serialize)]
pub struct Door {
    pub room: usize,
    pub door: usize,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct ApiResponse {
    #[serde(flatten)]
    pub data: Value,
}

pub struct ApiClient {
    client: Client,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub fn select(&self, problem_name: &str) -> Result<ApiResponse> {
        let request = SelectRequest {
            id: ID.to_string(),
            problem_name: problem_name.to_string(),
        };

        let response = self
            .client
            .post(&format!("{}/select", BASE_URL))
            .json(&request)
            .send()?
            .json::<ApiResponse>()?;

        Ok(response)
    }

    pub fn explore(&self, plans: &Vec<String>) -> Result<ExploreResult> {
        let request = ExploreRequest {
            id: ID.to_string(),
            plans: plans.clone(),
        };

        println!("{}", serde_json::to_string_pretty(&request)?);

        let response = self
            .client
            .post(format!("{BASE_URL}/explore"))
            .json(&request)
            .send()?
            .json::<ApiResponse>()?;

        let result = ExploreResult {
            results: response.data["results"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| {
                    x.as_array()
                        .unwrap()
                        .iter()
                        .map(|x| x.as_u64().unwrap() as usize)
                        .collect()
                })
                .collect(),
        };

        println!("{:?}", result.results);

        Ok(result)
    }

    pub fn guess(
        &self,
        rooms: Vec<usize>,
        starting_room: usize,
        connections: Vec<Connection>,
    ) -> Result<GuessResult> {
        let request = GuessRequest {
            id: ID.to_string(),
            map: Map {
                rooms,
                starting_room,
                connections,
            },
        };

        println!("{}", serde_json::to_string_pretty(&request)?);

        let response = self
            .client
            .post(format!("{BASE_URL}/guess"))
            .json(&request)
            .send()?
            .json::<GuessResult>()?;

        Ok(response)
    }
}
