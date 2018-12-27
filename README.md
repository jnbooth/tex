# [Tex]
A different approach to chatbot commands.

## Development

### Setup

Install [Rust](https://www.rust-lang.org/tools/install) if you don't already have it.

In order to run, the program needs access to a PostgreSQL server, which can be local or remote. PostgreSQL can be downloaded [here](https://www.postgresql.org/download/).

Copy `.env.example` to `.env` and fill out the fields, including the link to your PostgreSQL server. The server must be running for the next steps:

~~~
cargo install diesel_cli
diesel migration run
~~~

Once complete, the SQL server is safe to shut down.

### Building

To build and run the project, use `cargo run --release`. To build without running it, use `cargo build --release`. Note that the first time building the project will take much longer in order to download and compile dependencies. 

The PostgreSQL server from above must be running in order for the program to work.

## Commands

#### [choose], [ch]

__Usage:__ `choose choices, separated, by commas`

Randomly selects an item from a list.

#### [define], [def], [d]

__Usage:__ `define word`

Looks up the dictionary definition of a word.

#### [help]

__Usage:__ `help command`

Displays usage help for a command.

#### [hug]

__Usage:__ `hug`

Hugs the bot!

#### [remindme], [remind], [r]

__Usage:__ `remindme [<days>d][<hours>h][<minutes>m]`

Adds a reminder that will activate after a duration. When the reminder activates, the bot sends it to the user privately as soon as it sees a message from the user. Example: `[remindme 4h30m Fix my voice filter.]`

#### [roll]

__Usage examples:__ [roll d20 + 4 - 2d6!], [roll 3dF], [roll 2d6>3 + 10]

Randomly rolls some dice. Basic dice notation follows the format of `<# of dice>d<# of sides>`. Appending `!` marks dice as exploding, which means that if a die lands on its maximum value, it will be rolled again. `dF` are Fudge dice; they can have a value of `[+]` (1), `[ ]` (0), or `[-]` (-1). If followed by `>` and a number, that number is the success threshold; the roll's score is the number of dice that land on a number higher than that threshold. If followed by `<` and a number, that number is the failure threshold; the roll's score is the number of dice that land on a number lower than the threshold.

#### [seen], [se]

__Usage:__ `seen [-f|-t] nick`

With no flags, displays the most recent message seen from a user and how long ago it occurred. With `-f`, displays the first message seen from a user and how long ago it occurred. With `-t`, displays the total number of messages seen from a user. Note: `/me` emotes are ignored.

#### [select], [sel]

__Usage:__ `select number`

Selects one of several options given by the bot, such as when it retrieves a Wikipedia disambiguation page.

#### [wikipedia], [wiki], [w]

__Usage:__ `wiki article`

Looks up the search term on Wikipedia and returns a link to its article and an excerpt.

### Authorized Commands

These commands can only be used by users who have been granted authority by the `[auth]` command.

####  [auth]

__Usage:__ `auth level nick`

Promotes or demotes a user to a specified authorization level.

#### [disable]

__Usage:__ `enable command`

Disables usage of a command in the same channel. The bot does not respond to disabled commands.

#### [enable]

__Usage:__ `enable command`

Enables usage of a command in the same channel.

#### [forget]

__Usage:__ `forget nick`

Deletes all information about a user.

#### [quit]

__Usage:__ `quit`

Shuts down the bot.

### [reload]

__Usage:__ `reload`

Reloads the bot's data from its SQL database.
