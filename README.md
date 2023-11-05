# Footy CLI
### A CLI tool written in Rust to provide quick access to global football scores and schedules.

### Usage
You need to install the app by cloning the repository and executing a few commands:
`git clone https://github.com/jopieji/footy`

Add the executable (in {install_folder/footy/target/release}) to your PATH

Then, the command `footy schedule` will be available via your terminal.

Still, you need to add an API key to your environment. To obtain an API key, visit [the API-Football site](https://www.api-football.com/pricing) 

Open a `.zshrc` or `.bashrc` file and add\n
`EXPORT FOOTY_API_KEY={your_key_here}`

### Notes
Currently, the app only supports the schedule functionality. Scores and teams support will be added soon!

Also, it is hard coded to my favorite leagues, and only displays today's fixtures. This will be updated to be more customizable in the coming weeks.

### Commands
`footy scores` will return scores of your favorite teams, which can be configured via the CLI

`footy schedule` will return a schedule of today's fixtures

`footy teams` will open up a teams configuration portal
