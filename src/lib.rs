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

pub struct Settings {
    pub teams: Vec<String>,
    pub default: Command,
}

pub async fn run(cmd: Command) {
    println!("Made it to run with command type {cmd:?}");
}