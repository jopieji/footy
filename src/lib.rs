use std::env;

use chrono::{DateTime, Utc, Local, TimeZone};

use reqwest::Client;

use serde::{Serialize, Deserialize};
use serde_json::{Map, Value};

use colored::Colorize;

const BASE_URL: &str = "https://api-football-v1.p.rapidapi.com/v3/fixtures?";

#[derive(Debug)]
pub enum CommandType {
    Scores,
    Schedule,
    Teams,
    Live,
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
                "live" => CommandType::Live,
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

pub struct Settings {
    pub teams: Vec<String>,
    pub preferred_leagues: Vec<u64>,
    pub full_leagues: Vec<u64>,
    pub default: CommandType,
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
    timestamp: i64,
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

    let result = match_cmd_and_call(&cmd).await;

    match result {
        Ok(response_body) => {
            match parse_fixtures(response_body).await {
                Ok(fixture_responses) => {
                    for fixture_list in fixture_responses.iter() {
                        if fixture_list.len() == 0 { continue; }
                        println!("\n{} fixtures", &fixture_list[0].league.name);
                        for fixture in fixture_list.iter() {
                            print_based_on_command(fixture, &cmd);
                        }
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

// Top-level command matching
async fn match_cmd_and_call(cmd: &Command) -> Result<Vec<String>, String> {
    match cmd.command_type {
        CommandType::Schedule => get_schedule().await.map_err(|err| err.to_string()),
        CommandType::Scores => Err("Scores not implemented yet".to_string()),
        CommandType::Teams => Err("Teams not implemented yet".to_string()),
        CommandType::Live => get_live_fixtures().await.map_err(|err| err.to_string()),
    }
}

// Football-API calling methods
async fn get_schedule() -> Result<Vec<String>, reqwest::Error> {

    let mut res: Vec<String> = Vec::new();

    let key = env::var("FOOTY_API_KEY").unwrap();
    let client = Client::new();
    let settings = load_settings();

    // TODO: toggle for preferred vs full leagues Vectors

    for league_id in settings.preferred_leagues {
        let url = get_fixtures_url_by_league(league_id).await;
        let response = client.get(url).header("X-RapidAPI-KEY", &key).header("X-RapidAPI-Host", "api-football-v1.p.rapidapi.com").send().await.unwrap();
        let body = response.text().await?;
        res.push(body)
    }
    
    Ok(res)
}

async fn get_live_fixtures() -> Result<Vec<String>, reqwest::Error> {
    let mut res: Vec<String> = Vec::new();

    let key = env::var("FOOTY_API_KEY").unwrap();
    let client = Client::new();
    let settings = load_settings();

    let url = get_live_fixtures_url(settings).await;
    let response = client.get(url).header("X-RapidAPI-KEY", &key).header("X-RapidAPI-Host", "api-football-v1.p.rapidapi.com").send().await.unwrap();
    let body = response.text().await?;
    res.push(body);
    
    Ok(res)
}

// Serde parsing
async fn parse_fixtures(json_list: Vec<String>) -> Result<Vec<Vec<Fixture>>, Box<dyn std::error::Error>> {

    let mut res: Vec<Vec<Fixture>> = Vec::new();

    for json in json_list {
        let data: Map<String, Value> = serde_json::from_str(&json)?;

        // Extract the "response" field, which contains an array of fixtures
        let response = data.get("response").ok_or("Missing 'response' field")?;

        // Parse the array of fixtures into a vector of Fixture
        let league_fixture_list: Vec<Fixture> = serde_json::from_value(response.clone())?;

        res.push(league_fixture_list);
    }

    Ok(res)
}

// Utils Functions
async fn get_today_date() -> String {
    let now: DateTime<Utc> = Utc::now();
    let formatted_date = now.format("%Y-%m-%d").to_string();
    formatted_date
}


fn unix_to_cst (unix_timestamp: i64) -> String {
    // Create a DateTime object from the Unix timestamp (assuming it's in UTC)
    let local_time = Local.timestamp_opt(unix_timestamp, 0).unwrap();

    format_date(&local_time.to_string())
}

fn format_date(date: &String) -> String {
    let parts: Vec<&str> = date.split_whitespace().collect(); 
    parts[1].to_string()
}

// URL Configuration Functions
async fn get_fixtures_url_by_league(league_id: u64) -> String {
    let date = get_today_date().await;
    let season = &date[0..4];

    format!("{}league={}&season={}&date={}", BASE_URL, league_id, season, date)
}

async fn get_live_fixtures_url(settings: Settings) -> String{
    let mut leagues_live_field: String = String::from("");
    for league_id in settings.full_leagues {
        let append_item = format!("{}{}", league_id.to_string(), "-");
        leagues_live_field = leagues_live_field + &append_item;
    }
    leagues_live_field.pop();
    let url = format!("{}live={}", BASE_URL, leagues_live_field);
    url
}

// Settings functions
fn load_settings() -> Settings {
    let leagues_vec: Vec<u64> = vec!(39, 135, 78);
    let full_leagues_vec: Vec<u64> = vec!(39, 45, 48, 140, 143, 78, 88, 135);
    let teams_vec: Vec<String> = vec!("Liverpool".to_string(), "AC Milan".to_string());

    let settings = Settings {
        teams: teams_vec,
        preferred_leagues: leagues_vec,
        full_leagues: full_leagues_vec,
        default: CommandType::Schedule,
    };

    settings
}

// Output formatting
fn print_based_on_command(fixture: &Fixture, cmd: &Command) {
    match cmd.command_type {
        CommandType::Live => {
            let output = format!("{} @ {}: {} - {} in {}'", &fixture.teams.away.name.blue(), &fixture.teams.home.name.red(), &fixture.goals.away.unwrap().to_string().blue(), &fixture.goals.home.unwrap().to_string().red(), &fixture.fixture.status.elapsed.unwrap().to_string().bold());
            println!("{}", output);
        },
        CommandType::Schedule => {
            let output = format!("{} @ {} at {}", &fixture.teams.away.name.blue(), &fixture.teams.home.name.red(), unix_to_cst(fixture.fixture.timestamp).bold());
            println!("{}", output);
        },
        _ => {
            println!("Formatting not yet implemented");
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    
}