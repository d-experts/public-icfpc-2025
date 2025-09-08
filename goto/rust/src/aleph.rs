use crate::api;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::process::Command;

fn gen_random_string(alphabet: &str, length: usize, rng: &mut impl Rng) -> String {
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..alphabet.len());
            alphabet.chars().nth(idx).unwrap()
        })
        .collect()
}

pub fn gen_new_plan(bef_plan: &String, rng: &mut impl Rng) -> String {
    let mut new_plan = "[0]".to_string();
    let charcoals = gen_random_string("0123", bef_plan.len(), rng);
    for (p, c) in bef_plan.chars().zip(charcoals.chars()) {
        new_plan += &format!("{}[{}]", p, c);
    }
    new_plan
}
#[derive(Serialize, Debug)]
pub struct GoInput {
    pub plan: String,
    pub results: Vec<usize>,
    #[serde(rename = "mapData")]
    pub map_data: api::Map,
}

pub fn run_go_with_json(
    plan: &str,
    results: &[usize],
    map_data: &api::Map,
) -> Result<bool, Box<dyn std::error::Error>> {
    use std::io::Write;
    use std::process::Stdio;

    // JSONデータを作成
    let input_data = GoInput {
        plan: plan.to_string(),
        results: results.to_vec(),
        map_data: map_data.clone(),
    };

    let json_input = serde_json::to_string(&input_data)?;

    // Goプログラムを実行（標準入力でJSONを渡す）
    let mut child = Command::new("go")
        .arg("run")
        .arg("main.go")
        .current_dir("../golang")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // JSONを標準入力に書き込む
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(json_input.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    // 標準出力と標準エラー出力をプリント
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("Go program stdout: {}", stdout);
    if !stderr.is_empty() {
        println!("Go program stderr: {}", stderr);
    }

    if !output.status.success() {
        return Err(format!("Go program exited with status: {}", output.status).into());
    }

    // JSONレスポンスのみを抽出（最後の行、または{}で囲まれた部分）
    let json_line = stdout
        .lines()
        .rev() // 最後の行から検索
        .find(|line| {
            line.trim().starts_with('{') || line.trim() == "true" || line.trim() == "false"
        })
        .unwrap_or(stdout.trim());

    // Go言語から返されたboolean結果をデシリアライズ
    let result: bool = serde_json::from_str(json_line)?;
    Ok(result)
}

pub fn run_cpp_with_json(
    plan: &str,
    results: &[usize],
    map_data: &api::Map,
) -> Result<bool, Box<dyn std::error::Error>> {
    use std::io::Write;
    use std::process::Stdio;

    // JSONデータを作成
    let input_data = GoInput {
        plan: plan.to_string(),
        results: results.to_vec(),
        map_data: map_data.clone(),
    };

    let json_input = serde_json::to_string(&input_data)?;

    // jsonデータを../cpp/input.jsonに保存
    std::fs::write("../cpp/input.json", &json_input)?;

  use std::process::Command;
  use std::thread;
  use std::sync::Arc;

  // 並列実行（スレッド使用）
  let mut handles = vec![];

    for i in 0..2 {
        let handle = thread::spawn(move || {
            let mut child = Command::new("../cpp/a.out")
                .current_dir("../cpp")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to spawn");

            let output = child.wait_with_output().expect("Failed to wait");
            println!("Process {} finished", i);
            output
        });
        handles.push(handle);
    }

    // 全て完了を待つ
    let results: Vec<_> = handles.into_iter()
        .map(|h| h.join().unwrap())
        .collect();

    // 標準出力と標準エラー出力をプリント
  // どれかが"1"を返したかチェック
    let mut found = false;
    for (i, output) in results.iter().enumerate() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim() == "1" {
            println!("Process {} returned 1!", i);
            found = true;
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Process {}: {}", i, stderr);
    }
    // // cppがcoutで 1 か 0 を出力するので、それをboolに変換
    // let result: bool = match stdout.trim() {
    //      => true,
    //     "0" => false,
    //     _ => return Err("Unexpected output from C++ program".into()),
    // };
    // if !found
    //     return Err("Unexpected output from C++ program".into());
    Ok(found)
}
