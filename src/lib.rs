use std::env;

use chrono::{DateTime, Utc};

use reqwest::Client;
use reqwest::Error;

const BASE_URL: &str = "https://api-football-v1.p.rapidapi.com/v3/fixtures?league=61&";

#[derive(Debug)]
pub enum CommandType {
    Scores,
    Schedule,
    Teams,
}

#[derive(Debug)]
pub struct Command {
    pub command_type: CommandType
    // TODO: pub opts: Vec<char>
}

impl Command {
    pub fn build(
        mut args: impl Iterator<Item = String>,
    ) -> Result<Command, &'static str> {
        args.next();

        // matching input to a command type
        let command_type = match args.next() {
            Some(arg) => match arg.as_ref() {
                "scores" => CommandType::Scores,
                "schedule" => CommandType::Schedule,
                "teams" => CommandType::Teams,
                _ => return Err("Invalid command type")
            },
            None => return Err("Didn't enter any command"),
        };

        // TODO: opts logic

        Ok(Command {
            command_type,
        })
    }
}

// TODO: implement settings
pub struct Settings {
    pub teams: Vec<String>,
    pub default: Command,
}

pub async fn run(cmd: Command) {
    println!("Made it to run with command type {cmd:?}");
    
    let result = match_cmd_and_call(cmd).await;
    
    match_result(result);
}

async fn match_cmd_and_call(cmd: Command) -> Result<String, String> {
    match cmd.command_type {
        CommandType::Schedule => get_schedule().await.map_err(|err| err.to_string()),
        CommandType::Scores => Err("Scores not implemented yet".to_string()),
        CommandType::Teams => Err("Teams not implemented yet".to_string()),
    }
}

fn match_result(result: Result<String, String>) {
    match result {
        Ok(response_body) => println!("Success with body:\n {}", response_body),
        Err(error) => println!("Request failed with message: {}", error),
    }
}

// will pull params from environment using Settings struct

// TODO: function for calling scores endpoint
//async fn get_scores() {}

// TODO: function for calling schedule endpoint
async fn get_schedule() -> Result<String, Error> {
    let key = env::var("FOOTY_API_KEY").unwrap();
    let client = Client::new();
    let url = get_fixtures_url().await;
    let response = client.get(url).header("X-RapidAPI-KEY", key).header("X-RapidAPI-Host", "api-football-v1.p.rapidapi.com").send().await.unwrap();
    dbg!(&response);
    let body = response.text().await?;
    
    Ok(body)
}

// TODO: put in utils package
async fn get_today_date() -> String {
    let now: DateTime<Utc> = Utc::now();
    let formatted_date = now.format("%Y-%m-%d").to_string();
    formatted_date
}

async fn get_fixtures_url() -> String {
    let date = get_today_date().await;
    let season = &date[0..4];

    format!("{}season={}&date={}", BASE_URL, season, date)
}

/*
 date (YYYY-MM-DD)
 live (id-id) to filter by league id
 team id
 next/last to get next/last k fixtures
 from/to date params
 Timezone from endpoint
 */

// TODO: function for calling teams endpoint
//async fn get_teams() {}

/*
use tokio::task;

async fn call_api() -> Result<String, String> {
    let response = reqwest::get("https://example.com/api/v1/users").await?;
    let body = response.text().await?;
    Ok(body)
}

fn main() {
    task::spawn(call_api()).await?;
} */