# Footy CLI
### A CLI tool written in Rust to provide quick access to global football scores and schedules.

### Usage
You need to install the app by cloning the repository and executing a few commands:
`git clone https://github.com/jopieji/footy`

Run `cargo build --release` to generate an optimized binary.

Add the executable (in {install_folder/footy/target/release}) to your PATH

Still, you need to add an API key to your environment. To obtain an API key, visit [the API-Football site](https://www.api-football.com/pricing) 

Open a `.zshrc` or `.bashrc` file and add\n
`export FOOTY_API_KEY={your_key_here}`

Set `CONFIG_PATH` to the absolute path of your teams.csv files, and add it to your `.zshrc` or `.bashrc` file with the syntax `export CONFIG_PATH={abs_path_to_your_teams.csv}`. Do the same for `RGB_PATH`.

Finally, the command `footy schedule` will be available via your terminal.

### Commands
`footy scores` will display scores of your favorite teams, which can be configured via the CLI (right now, it shows the last two fixtures)

`footy schedule` will display a schedule of today's fixtures for your favorite leagues

`footy live` will display live scores of matches for you full list of leagues

`footy standings` will display the current table for all configured leagues (right now, preconfigured for La Liga, Premier League, Serie A, and Bundesliga)

`footy teams` will allow you to edit your favorited teams

### Notes
I am looking to make preferred leagues configurable in the future. I also want to automate setup so there's no need to set env vars and everything (this was more of a time-saving and cost-saving mechanism as I didn't want my API key to get throttled. Any user can use the Football API free for ~50 calls a day.)

There are custom color configurations that only work for a set number of leagues. I built a python tool to grab the max occuring (non-white/black) color of each logo [here](https://github.com/jopieji/py-get-color-of-image.git) if you want to check it out or clone it to get even more leagues configured!


### Future
I'm looking into adding: 
- customizable league adding/deleting for your favorites list (affects `live`, `schedule` commands)
- schedule lookahead (show upcoming fixtures for teams)
- prettier printing for all commands
### Known Bugs
Teams that share names with other teams have trouble being added. For example, there are two teams that come up for "Arsenal", so Arsenal
won't properly be added to your list of teams. I am looking into this, and will find a resolution as soon as possible.

If you manually edit the teams.csv file, you need to ensure there's a newline at the end. 
