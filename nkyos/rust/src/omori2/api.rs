use std::{error::Error, time::Duration};

use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://31pwr5t6ij.execute-api.eu-west-2.amazonaws.com/";
const TEAM_ID: &str = "";

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

#[derive(Serialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct RoomAndDoor {
    pub room: usize,
    pub door: usize,
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
