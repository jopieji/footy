use std::{env, collections::HashMap, io, error::Error, fs::OpenOptions, process};

use csv::{ReaderBuilder, StringRecord};

use chrono::{DateTime, Utc, Local, TimeZone};

use reqwest::Client;

use serde::{Serialize, Deserialize};
use serde_json::{Map, Value};

use colored::Colorize;

const BASE_URL: &str = "https://api-football-v1.p.rapidapi.com/v3/fixtures?";

#[derive(Debug, PartialEq)]
pub enum CommandType {
    Scores,
    Schedule,
    Teams,
    Live,
    Standings
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
                "standings" => CommandType::Standings,
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
    score: Option<ScoreData>,
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
    flag: Option<String>,
    season: u16,
    round: Option<String>,
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

#[derive(Debug, Deserialize, Serialize)]
struct StandingsResponse {
    league: League,
}

#[derive(Debug, Deserialize, Serialize)]
struct League {
    standings: Vec<Vec<TeamStanding>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TeamStanding {
    all: Stats,
    away: Stats,
    description: Option<String>,
    form: Option<String>,
    #[serde (rename="goalsDiff")]
    goals_diff: i32,
    group: Option<String>,
    home: Stats,
    points: i32,
    rank: i32,
    status: Option<String>,
    team: Team,
    update: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Stats {
    draw: i32,
    goals: GoalStats,
    lose: i32,
    played: i32,
    win: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct GoalStats {
    against: i32,
    #[serde (rename="for")]
    for_: i32,
}

pub async fn run(cmd: Command) {

    let result = match_cmd_and_call(&cmd).await;

    match result {
        Ok(response_body) => {
            if check_if_not_fixtures_trait_type(&cmd) { return; }
            match parse_fixtures(response_body).await {
                Ok(fixture_responses) => {
                    for fixture_list in fixture_responses.iter() {
                        if fixture_list.is_empty() { println!("No fixtures :("); break;}
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
        CommandType::Scores => {
            get_teams_fixtures().await.map_err(|err| err.to_string())
        }
        CommandType::Teams => {
            prompt_teams_edit().await;
            Ok(vec![])
        },
        CommandType::Live => get_live_fixtures().await.map_err(|err| err.to_string()),
        CommandType::Standings => get_standings_for_base_leagues().await.map_err( | err | err.to_string()),
    }
}

// Football-API calling methods
async fn get_schedule() -> Result<Vec<String>, reqwest::Error> {

    smart_print_date();

    let mut res: Vec<String> = Vec::new();

    let key = env::var("FOOTY_API_KEY").unwrap();
    let client = Client::new();
    let settings = load_settings();

    // could add a new item at end of each league query to print whitespace between leagues 
    // or print by
    for league_id in settings.preferred_leagues {
        let url = get_fixtures_url_by_league(league_id).await;
        let response = client.get(url)
            .header("X-RapidAPI-KEY", &key)
            .header("X-RapidAPI-Host", "api-football-v1.p.rapidapi.com")
            .send()
            .await
            .unwrap();
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
        .send()
        .await
        .unwrap()
        .text()
        .await?;

    res.push(response);
    
    Ok(res)
}

async fn get_teams_fixtures() -> Result<Vec<String>, reqwest::Error> {
    println!("My Teams Fixtures\n");
    let mut res: Vec<String> = Vec::new();

    let teams_file = read_from_teams_csv();

    let teams = match teams_file {
        Ok(teams_file) => teams_file,
        Err(_) => {
            println!("File not found. Exiting");
            let mut hm = HashMap::new();
            hm.insert(String::from("Err"), 10);
            hm
        }
    };

    if teams.contains_key("Err") {
        process::exit(1);
    }

    let key = env::var("FOOTY_API_KEY").unwrap();
    let client = Client::new();

    for (_team, team_id) in teams {
        let url = get_team_url(team_id).await;
        let response = client.get(url)
            .header("X-RapidAPI-KEY", &key)
            .header("X-RapidAPI-Host", "api-football-v1.p.rapidapi.com")
            .send()
            .await
            .unwrap()
            .text()
            .await?;
        res.push(response);
    }

    Ok(res)
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

async fn get_standings_for_base_leagues() -> Result<Vec<String>,  Box<dyn Error>> {
    // can either get leauge id from config list, static list, or from teams favorited
    let key = env::var("FOOTY_API_KEY").unwrap();
    let client = Client::new();

    let static_temp_list = vec![39, 140, 78, 135];

    let mut res: Vec<String> = Vec::new();

    for league_id in static_temp_list {
        let url = format!("{}?league={}&season=2023", "https://api-football-v1.p.rapidapi.com/v3/standings", league_id);

        let response = client.get(url)
        .header("X-RapidAPI-KEY", &key)
        .header("X-RapidAPI-Host", "api-football-v1.p.rapidapi.com")
        .send()
        .await
        .unwrap()
        .text()
        .await?;

        res.push(response);
    }

    match parse_standings(&res) {
        Ok(standings) => {
            print_standings_by_league(standings);
        },
        Err(err) => {
            println!("Error occurred: {}", err);
        }

    };

    // todo: edit return item
    Ok(res)
}

// Serde parsing
async fn parse_fixtures(json_list: Vec<String>) -> Result<Vec<Vec<Fixture>>, Box<dyn Error>> {

    // logic to step out on add: no fixtures to parse
    if json_list.is_empty() {
        return Ok(vec![vec![]])
    }

    let mut res: Vec<Vec<Fixture>> = Vec::new();

    for json in json_list {
        let data: Map<String, Value> = serde_json::from_str(&json)?;
        let response = data.get("response").ok_or("Missing 'response' field")?;
        let league_fixture_list: Vec<Fixture> = serde_json::from_value(response.clone())?;
        res.push(league_fixture_list);
    }

    Ok(res)
}

fn parse_standings(raw_response: &Vec<String>) -> Result<Vec<Vec<Vec<TeamStanding>>>, Box<dyn Error>> {
    if raw_response.is_empty() {
        return Ok(vec![])
    }

    let mut leagues_list: Vec<Vec<Vec<TeamStanding>>> = Vec::new();

    for json_response in raw_response {
        let data: Map<String, Value> = serde_json::from_str(json_response)?;
        let response: &Value = data.get("response").ok_or("Missing 'response' field")?;
        let standings_response: StandingsResponse = serde_json::from_value(response[0].clone())?;
        let team_standings: Vec<Vec<TeamStanding>> = standings_response.league.standings;
        leagues_list.push(team_standings);
    }
    Ok(leagues_list)
}

// Utils Functions
fn check_if_not_fixtures_trait_type(cmd: &Command) -> bool {
    match cmd.command_type {
        CommandType::Teams => true,
        CommandType::Standings => true,
        _ => false,
    }
}

async fn get_today_date() -> String {
    let now: DateTime<Utc> = Utc::now();
    let formatted_date = now.format("%Y-%m-%d").to_string();
    formatted_date
}

fn unix_to_cst (unix_timestamp: i64) -> String {
    let local_time = Local.timestamp_opt(unix_timestamp, 0).unwrap();

    format_time(local_time.to_string())
}

fn unix_to_date (unix_timestamp: i64) -> String {
    let local_time = Local.timestamp_opt(unix_timestamp, 0).unwrap();
    format_date(local_time.to_string())
}

fn format_time(date: String) -> String {
    let parts: Vec<&str> = date.split_whitespace().collect(); 
    parts[1][0..5].to_string()
}

fn format_date(date: String) -> String {
    let parts: Vec<&str> = date.split_whitespace().collect();
    parts[0][5..].to_string()
}

fn smart_print_date() {
    let date = unix_to_date(Utc::now().timestamp());

    let hour = unix_to_cst(Utc::now().timestamp());
    let hr_int = hour[0..2].parse::<i64>().unwrap();
    if hr_int > 18 {
        println!("{} Fixtures", unix_to_date(Utc::now().timestamp()+40000));
    } else {
        println!("{} Fixtures", date);
    }
}

fn read_from_teams_csv() -> Result<HashMap<String, u64>, Box<dyn std::error::Error>> {
    let mut teams_with_ids: HashMap<String, u64> = HashMap::new();
    let path = env::var("CONFIG_PATH");
    let path_string = path.unwrap_or("./teams.csv".to_string());
    let mut csv = ReaderBuilder::new().has_headers(false).delimiter(b',').from_path(path_string)?;

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
            let _ = add_team_to_csv(team_struct.team).unwrap();
            println!("Added {}", t);
        },
        Err(error) => {
            dbg!("Not a valid team. Try again.");
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

fn remove_team_from_csv(team: String) -> Result<(), Box<dyn Error>> {
    let mut csv_reader = ReaderBuilder::new().has_headers(false).delimiter(b',').from_path("./teams.csv").unwrap();

    let mut records: Vec<TeamCSVRecord> = csv_reader.deserialize().collect::<Result<Vec<_>, _>>()?;

    records.retain(|record| record.name.to_lowercase() != team.to_lowercase());

    let file = OpenOptions::new().write(true).truncate(true).open("./teams.csv")?;
    let mut csv_writer = csv::WriterBuilder::new().has_headers(false).delimiter(b',').from_writer(file);

    for record in records {
        csv_writer.serialize(record)?;
    }
        
    csv_writer.flush()?;

    println!("Removed {} from your teams list", team);
    Ok(())
}

async fn prompt_teams_edit()  {
    println!("Type 'a' to add a team or 'r' to remove a team");

    let mut char_input = String::new();

    let stdin = io::stdin();

    // Read user input into the `user_input` string
    stdin.read_line(&mut char_input)
        .expect("Failed to read input");

    match char_input.trim() {
        "a" => {
            let team = get_team_input('a');
            let _ = add_team(team).await;
        }
        "r" => {
            let team = get_team_input('r');
            let _ = remove_team_from_csv(team);
        },
        &_ => {
            println!("Invalid input");
            // todo: prompt_add().await;
        }
    }
}

fn get_team_input(opt: char) -> String {
    if opt == 'a' {
        println!("Enter a team to add to your list of teams: ");
    } else if opt == 'r' {
        println!("\nYour current teams: ");
        print_all_teams();
        println!("\nEnter a team to remove from your list of teams:");
    }
    
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

async fn get_team_url(team_id: u64) -> String {
    let url = format!("{}season=2023&team={}&last=2", BASE_URL, team_id);
    url
} 

// Settings functions
fn load_settings() -> Settings {
    let leagues_vec: Vec<u64> = vec!(39, 135, 78);
    let full_leagues_vec: Vec<u64> = vec!(2, 39, 45, 48, 140, 143, 78, 88, 135);
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
            let output = format!("{} @ {} at {} {}", &fixture.teams.away.name.blue(), &fixture.teams.home.name.red(), unix_to_cst(fixture.fixture.timestamp).bold(), check_if_in_progress(&fixture.fixture.status.short));
            println!("{}", output);
        },
        CommandType::Teams => {
            // Empty: printing done in functions
        },
        CommandType::Scores => {
            let output = format!("{} @ {}: {} - {} on {}", &fixture.teams.away.name.blue(), &fixture.teams.home.name.red(), &fixture.goals.away.unwrap().to_string().blue(), &fixture.goals.home.unwrap().to_string().red(), &fixture.fixture.date[5..10]);
            println!("{}", output);
        },
        CommandType::Standings => {
            // Empty: printing done in functions
        },
    }
}

fn print_all_teams() {

    let mut csv = ReaderBuilder::new().has_headers(false).delimiter(b',').from_path("./teams.csv").unwrap();

    for res in csv.records() {
        let row = res.unwrap();
        let csv_row: TeamCSVRecord = row.deserialize(None).unwrap();

        println!("{}", csv_row.name);
    }
}

fn print_standings_by_league(league_standings: Vec<Vec<Vec<TeamStanding>>>) {
    for vec in league_standings {
        for league_standing in vec {
            for team in league_standing {
                format_team_row(team);
            }
        }
        println!("=================================================\n")
    }
}

fn format_team_row(team: TeamStanding) {
    let formatted_team_row = format!(
        "{:<5} {:<25} {:<10} {:<10}",
        team.rank,
        team.team.name,
        team.points,
        team.form.unwrap_or_else(|| String::from("na"))
    );
    println!("{}", formatted_team_row);
}

fn check_if_in_progress(short_status: &String) -> &str {
    if short_status != "TBD" || short_status != "NS" {
        "| In Progress"
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

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

        assert_eq!(res.unwrap().get("Liverpool"), tester.get("Liverpool"));
    }

    #[test]
    fn remove_team_test() {

        let path_string = "./teams.csv";
        
        // add team (to ensure in Vec), check length after collecting records into vec
        let team_to_add = TeamCSVRecord {
            name: String::from("Team"),
            id: 1,
        };

        let _ = add_team_to_csv(team_to_add);

        let mut csv1 = ReaderBuilder::new().has_headers(false).delimiter(b',').from_path(path_string).unwrap();
        let records1: Vec<TeamCSVRecord> = csv1.deserialize().collect::<Result<Vec<_>, _>>().unwrap();
        let original_length = records1.len();

        let _ = remove_team_from_csv(String::from("Team"));


        // collect again, compare lengths
        // assert they are different by one
        let mut csv2 =  ReaderBuilder::new().has_headers(false).delimiter(b',').from_path(path_string).unwrap();
        let records2: Vec<TeamCSVRecord> = csv2.deserialize().collect::<Result<Vec<_>, _>>().unwrap();
        let new_length = records2.len();

        assert_eq!(original_length-1, new_length);

    }

    #[test]
    fn test_check_if_teams_command() {
        let cmd: Command = Command {
            command_type: CommandType::Teams,
        };
        let check = check_if_not_fixtures_trait_type(&cmd);

        assert_eq!(true, check);
    }

    fn test_check_if_standings_command() {
        let cmd: Command = Command {
            command_type: CommandType::Standings,
        };
        let check = check_if_not_fixtures_trait_type(&cmd);

        assert_eq!(true, check);
    }


}