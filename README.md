# [Tex]
A different approach to chatbot commands.

## Development

### Setup

Install [Rust](#https://www.rust-lang.org/tools/install) if you don't already have it, then run the following commands:

~~~
cargo install diesel_cli
diesel migration run
cp .env.example .env
~~~

Finally, fill out the fields in `.env`.

### Building

To build and run the project, use `cargo run --release`. To build without running it, use `cargo build --release`. Note that the first time building the project will take much longer in order to download and compile dependencies. 

In order to run, the program needs access to a live PostgreSQL server, which can be local or remote. PostgreSQL can be downloaded [here](#https://www.postgresql.org/download/). The server connection is configured in the `.env` file.

## Commands

#### [choose], [ch]

__Usage:__ `choose choices, separated, by commas`

Randomly selects an item from a list.

#### [help]

__Usage:__ `help command`

Displays usage help for a command.

#### [hug]

__Usage:__ `hug`

Hugs the bot!

#### [remindme], [remind], [r]

__Usage:__ `remindme [<days>d][<hours>h][<minutes>m]`

Adds a reminder that will activate after a duration. When the reminder activates, the bot sends it to the user privately as soon as it sees a message from the user. Example: `[remindme 4h30m Fix my voice filter.]`

#### [wikipedia], [wiki], [w]

__Usage:__ `wiki article`

Looks up the search term on Wikipedia and returns a link to its article and an excerpt.

### Authorized Commands

These commands can only be used by users who have been granted authority by the `[auth]` command.

####  [auth]

__Usage:__ `auth level user`

Promotes or demotes a user to a specified authorization level.

#### [forget]

__Usage:__ `forget user`

Deletes all information about a user.

#### [quit]

__Usage:__ `quit`

Shuts down the bot.

### [reload]

__Usage:__ `reload`

Reloads the bot's data from its SQL database.
