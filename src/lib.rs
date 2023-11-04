use std::env;

use chrono::{DateTime, Utc};

use reqwest::Client;
use reqwest::Error;

use serde::Serialize;
use serde::Deserialize;

use serde_json::Map;
use serde_json::Value;

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

// Serde structs
#[derive(Serialize, Deserialize, Debug)]
struct Fixture {
    fixture: FixtureData,
    league: LeagueData,
    teams: TeamsData,
    goals: GoalsData,
    score: ScoreData,
}

#[derive(Serialize, Deserialize, Debug)]
struct FixtureData {
    id: u64,
    referee: String,
    timezone: String,
    date: String,
    timestamp: u64,
    periods: Periods,
    venue: Venue,
    status: Status,
}

#[derive(Serialize, Deserialize, Debug)]
struct Periods {
    first: Option<u64>,
    second: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Venue {
    id: u64,
    name: String,
    city: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Status {
    long: String,
    short: String,
    elapsed: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct LeagueData {
    id: u64,
    name: String,
    country: String,
    logo: String,
    flag: String,
    season: u16,
    round: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct TeamsData {
    home: Team,
    away: Team,
}

#[derive(Serialize, Deserialize, Debug)]
struct Team {
    id: u64,
    name: String,
    logo: String,
    winner: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
struct GoalsData {
    home: Option<u64>,
    away: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ScoreData {
    halftime: HalftimeScore,
    fulltime: FulltimeScore,
    extratime: ExtraTimeScore,
    penalty: PenaltyScore,
}

#[derive(Serialize, Deserialize, Debug)]
struct HalftimeScore {
    home: Option<u64>,
    away: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FulltimeScore {
    home: Option<u64>,
    away: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ExtraTimeScore {
    home: Option<u64>,
    away: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PenaltyScore {
    home: Option<u64>,
    away: Option<u64>,
}

pub async fn run(cmd: Command) {

    let result = match_cmd_and_call(cmd).await;
    
    match result {
        Ok(response_body) => {

            let fixture_response = parse_fixtures(response_body).await;

            match fixture_response {
                Ok(fixture_list) => {
                    println!("Fixtures for today:");
                    println!("========================");
                    for fixture in fixture_list.iter() {
                        let fixture_output_str = format!("{} @ {}", &fixture.teams.away.name, &fixture.teams.home.name);
                        println!("{}", fixture_output_str);
                    }
                }
                Err(err) => {
                    eprintln!("Error parsing fixtures: {}", err);
                }
            }
        }
        Err(err) => {
            eprintln!("Error from the API: {}", err);
        }
    }
    // could filter by teams in settings: iterator from vector, check home/away fields

}

async fn parse_fixtures(json: String) -> Result<Vec<Fixture>, Box<dyn std::error::Error>> {
    let data: Map<String, Value> = serde_json::from_str(&json)?;

    // Extract the "response" field, which contains an array of fixtures
    let response = data.get("response").ok_or("Missing 'response' field")?;

    // Parse the array of fixtures into a vector of Fixture
    let fixture_list: Vec<Fixture> = serde_json::from_value(response.clone())?;

    Ok(fixture_list)
}

async fn match_cmd_and_call(cmd: Command) -> Result<String, String> {
    match cmd.command_type {
        CommandType::Schedule => get_schedule().await.map_err(|err| err.to_string()),
        CommandType::Scores => Err("Scores not implemented yet".to_string()),
        CommandType::Teams => Err("Teams not implemented yet".to_string()),
    }
}

// will pull params from environment using Settings struct

// TODO: function for calling scores endpoint
//async fn get_scores() {}

async fn get_schedule() -> Result<String, Error> {
    let key = env::var("FOOTY_API_KEY").unwrap();
    let client = Client::new();

    let url = get_fixtures_url().await;

    let response = client.get(url).header("X-RapidAPI-KEY", key).header("X-RapidAPI-Host", "api-football-v1.p.rapidapi.com").send().await.unwrap();
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