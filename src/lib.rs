use std::{env, collections::HashMap, io, error::Error, fs::OpenOptions};

use csv::{ReaderBuilder, StringRecord, Writer};

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
    Add,
}

#[derive(Debug)]
pub struct Command {
    pub command_type: CommandType
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
                "add" => CommandType::Add,
                _ => return Err("Invalid command type")
            },
            None => return Err("Didn't enter any command"),
        };

        Ok(Command {
            command_type,
        })
    }
}

pub struct Settings {
    pub teams: HashMap<String, u64>,
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

impl Clone for Venue {
    fn clone(&self) -> Self {
        Venue {
            id: self.id.clone(),
            name: self.name.clone(),
            city: self.city.clone(),
        }
    }
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
struct TeamResponse {
    response: Vec<TeamInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TeamInfo {
    team: TeamCSVRecord,
    venue: Venue,
}

impl Clone for TeamInfo {
    fn clone(&self) -> Self {
        TeamInfo {
            team: self.team.clone(),
            venue: self.venue.clone(),
        }
    }
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

#[derive(Serialize, Deserialize, Debug)]
struct TeamCSVRecord {
    name: String,
    id: u64,
}

impl Clone for TeamCSVRecord {
    fn clone(&self) -> Self {
        TeamCSVRecord {
            name: self.name.clone(),
            id: self.id.clone(),
        }
    }
}

pub async fn run(cmd: Command) {

    let result = match_cmd_and_call(&cmd).await;

    match result {
        Ok(response_body) => {
            match parse_fixtures(response_body).await {
                Ok(fixture_responses) => { 
                    for fixture_list in fixture_responses.iter() {
                        if fixture_list.is_empty() { continue; }
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

}

// Top-level command matching
async fn match_cmd_and_call(cmd: &Command) -> Result<Vec<String>, String> {
    match cmd.command_type {
        CommandType::Schedule => get_schedule().await.map_err(|err| err.to_string()),
        CommandType::Scores => Err("Scores not implemented yet".to_string()),
        CommandType::Teams => Err("Teams not implemented yet".to_string()),
        CommandType::Live => get_live_fixtures().await.map_err(|err| err.to_string()),
        CommandType::Add => {
            prompt_add().await;
            Ok(vec![])
        }
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
        let response = client.get(url)
        .header("X-RapidAPI-KEY", &key)
        .header("X-RapidAPI-Host", "api-football-v1.p.rapidapi.com")
        .send().await.unwrap();
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
    let response = client.get(url)
        .header("X-RapidAPI-KEY", &key)
        .header("X-RapidAPI-Host", "api-football-v1.p.rapidapi.com")
        .send().await.unwrap();
    let body = response.text().await?;

    res.push(body);
    
    Ok(res)
}

async fn get_teams_fixtures() -> Result<Vec<String>, reqwest::Error> {
    //let mut res: Vec<String> = Vec::new();

    // need to follow same logic
    // could query all team fixtures for this current season, then filter by closest to today's date in separate function
    // would also need to check for past, live, and upcoming
    // could just iterate thru all fixtures, check for most recent before today's date, save in variable
    // if one is live, it can override this value
    // then, take first value that is upcoming and break and output


    Ok(vec![])
}

async fn try_get_team_id(team: String) -> Result<TeamInfo, Box<dyn Error>> {
    let key = env::var("FOOTY_API_KEY").unwrap();
    let url = format!("{}?name={}", "https://api-football-v1.p.rapidapi.com/v3/teams", team);
    let client = Client::new();

    let response = client.get(url)
        .header("X-RapidAPI-KEY", &key)
        .header("X-RapidAPI-Host", "api-football-v1.p.rapidapi.com")
        .send()
        .await
        .unwrap()
        .text()
        .await?;

    let team_response: TeamResponse = serde_json::from_str(&response)?;

    match team_response.response.get(0).cloned() {
        Some(data) => Ok(data),
        None => Err("Not a valid team. Try again!".to_string().into()),
    }
}


// Serde parsing
async fn parse_fixtures(json_list: Vec<String>) -> Result<Vec<Vec<Fixture>>, Box<dyn std::error::Error>> {

    // logic to step out on add: no fixtures to parse
    if json_list.len() == 0 {
        return Ok(vec![vec![]])
    }

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

    format_date(local_time.to_string())
}

fn format_date(date: String) -> String {
    let parts: Vec<&str> = date.split_whitespace().collect(); 
    parts[1].to_string()
}

fn read_from_teams_csv() -> Result<HashMap<String, u64>, Box<dyn std::error::Error>> {
    let mut teams_with_ids: HashMap<String, u64> = HashMap::new();
    let mut csv = ReaderBuilder::new().has_headers(false).delimiter(b',').from_path("./teams.csv")?;

    for res in csv.records() {
        let row: StringRecord = res?;
        let team_record: TeamCSVRecord = row.deserialize(None)?;
        teams_with_ids.insert(team_record.name, team_record.id);
    }
    Ok(teams_with_ids)
}

async fn add_team(team: String) -> Result<(), reqwest::Error> {

    let t = team.clone();

    match try_get_team_id(team).await  {
        Ok(team_struct) => {
            add_team_to_csv(team_struct.team).unwrap();
            println!("Added {}", t);
        },
        Err(error) => {
            println!("{}", error);
        }
    }

    Ok(())
}

fn add_team_to_csv(team: TeamCSVRecord) -> Result<(), Box<dyn std::error::Error>> {

    let file = OpenOptions::new()
    .create(true)
    .append(true)
    .open("./teams.csv")?;

    let mut csv_writer = csv::WriterBuilder::new()
        .has_headers(false)
        .delimiter(b',')
        .from_writer(file);
    
    csv_writer.serialize(team)?;

    csv_writer.flush()?;

    Ok(())
}

async fn prompt_add()  {
    println!("Type 'l' to add a league or 't' to add a team");

    let mut char_input = String::new();

    let stdin = io::stdin();

    // Read user input into the `user_input` string
    stdin.read_line(&mut char_input)
        .expect("Failed to read input");

    match char_input.trim() {
        "t" => {
            let team = get_team_input();
            add_team(team).await;
        }
        "l" => {
            println!("Not yet configured. Check again soon!");
        },
        &_ => todo!()
    }
}

fn get_team_input() -> String {
    println!("Enter a team to add to your list of teams: ");
    let mut team_input = String::new();

    let stdin = io::stdin();
    stdin.read_line(&mut team_input)
        .expect("Failed to read input");
    
    team_input.trim().to_string()
}

// URL Configuration Functions
async fn get_fixtures_url_by_league(league_id: u64) -> String {
    let date = get_today_date().await;
    let season = &date[0..4];

    format!("{}league={}&season={}&date={}", BASE_URL, league_id, season, date)
}

async fn get_live_fixtures_url(settings: Settings) -> String {
    let mut leagues_live_field: String = String::from("");
    for league_id in settings.full_leagues {
        let append_item = format!("{}{}", league_id, "-");
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
    let teams_vec: HashMap<String, u64> = HashMap::new();

    Settings {
        teams: teams_vec,
        preferred_leagues: leagues_vec,
        full_leagues: full_leagues_vec,
        default: CommandType::Schedule,
    }
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
        CommandType::Teams => {
            let output = format!("{} @ {}", &fixture.teams.away.name.blue(), &fixture.teams.home.name.red());
            println!("{}", output);
        },
        _ => {
            println!("Formatting not yet implemented");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::read_from_teams_csv;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn csv_read() {
        let mut tester: HashMap<String, u64>  = HashMap::new();
        tester.insert(String::from("Liverpool"), 40);
        tester.insert(String::from("AC Milan"), 907);

        let res = read_from_teams_csv();

        match &res {
            Ok(res) => {
                dbg!(&res);
            },
            Err(error) => {
                dbg!("Error reading csv into HashMap: {}", error);
            }
        }

        assert_eq!(res.unwrap(), tester);
    }


}