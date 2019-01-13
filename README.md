# [Tex]

[![Build Status](https://travis-ci.com/jnbooth/tex.svg?branch=master)](https://travis-ci.com/jnbooth/tex)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

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

To build and run the project, use `cargo run --release`. To run it in local offline mode, use `cargo run --release -- -o`. To build without running it, use `cargo build --release`. Note that the first time building the project will take much longer in order to download and compile dependencies. 

The PostgreSQL server from above must be running in order for the program to work.

### Testing

To test the project, use `cargo test`. By default, the test skips tests that require database access or API keys with thresholds. To run those tests, use `cargo test -- --ignored`. Note that in order to avoid false positives, optional features such as bans will fail their tests if disabled.

## Commands

Commands can be invoked in several ways. For example, any of the following will work:

* .w Enron
* !w Enron
* So basically they're the new [w Enron].

#### [choose], [ch]

__Usage:__ `ch <choices, separated, by commas>`

Randomly select an item from a list.

#### [define], [def], [d]

__Usage:__ `def <word>`

Look up the dictionary definition of a word.

#### [gis]

__Usage:__ `gis <query>`

Search for an image using Google.

#### [google], [g]

__Usage:__ `g <query>`

Search for a website using Google.

#### [help]

__Usage:__ `help <command>`

Display usage help for a command.

#### [hug]

__Usage:__ `hug`

Hug the bot!

#### [lastcreated], [lc], [l]

__Usage:__ `lc`

#### [memo]

__Usage:__ `memo <user>`

Displays a user's memo in the current channel. A user can only have one memo per channel.

##### [memo add]

__Usage:__ `memo add <user> <message>`

Stores a memo for a user. Fails if the user already has a memo.

##### [memo append], [rem]

__Usage:__ `memo append <user> <message>`

Stores a memo for a user. If the user already has a memo, the message is appended to the end of it.

##### [memo del]

__Usage:__ `memo del <user> <message>`

Deletes a user's memo. Fails if the memo differs from the message provided.

#### [name]

__Usage:__ `name [-f|-m]`

Randomly generates a name. With no flags, gender is random. `-f` generates a female name. `-m` generates a male name.

#### [remindme], [remind], [r]

__Usage:__ `r [<days>d][<hours>h][<minutes>m] <message>`

Add a reminder that will activate after a duration. When the reminder activates, the bot sends it to the user privately as soon as it sees a message from the user. Example: `[remindme 4h30m Fix my voice filter.]`

#### [roll]

__Usage examples:__ [roll d20 + 4 - 2d6!], [roll 3dF], [roll 2d6>3 + 10]

Randomly roll some dice. Basic dice notation follows the format of `<# of dice>d<# of sides>`. Appending `!` marks dice as exploding, which means that if a die lands on its maximum value, it will be rolled again. `dF` are Fudge dice; they can have a value of `[+]` (1), `[ ]` (0), or `[-]` (-1). If followed by `>` and a number, that number is the success threshold; the roll's score is the number of dice that land on a number higher than that threshold. If followed by `<` and a number, that number is the failure threshold; the roll's score is the number of dice that land on a number lower than the threshold.

#### [seen], [se]

__Usage:__ `seen [#<channel>] [-f|-t] <user>`

With no flags, display the most recent message seen from a user and how long ago it occurred. `-f` displays the first message seen from a user and how long ago it occurred. `-t` displays the total number of messages seen from a user. If a channel is not given, the current channel is used. Note: `/me` emotes are ignored.

#### [showmore], [sm]

__Usage:__ `sm <number>`

Select one of several options given by the bot, such as when it retrieves a Wikipedia disambiguation page.

#### [tell], [t]

__Usage:__ `tell <user> <message>`

Send a message to another user. The bot will privately send the message to the user when it next sees a message from them.

#### [wikipedia], [wiki], [w]

__Usage:__ `w <article>`

Look up the search term on Wikipedia and returns a link to its article and an excerpt.

### Authorized Commands

These commands can only be used by users who have been granted authority by the `[auth]` command.

####  [auth]

__Usage:__ `auth <level> <user>`

Promote or demote a user to a specified authorization level.

#### [disable]

__Usage:__ `disable <command>`

Disable usage of a command in the same channel. The bot does not respond to disabled commands.

#### [enable]

__Usage:__ `enable <command>`

Enable usage of a command in the same channel.

#### [forget]

__Usage:__ `forget <user>`

Delete all information about a user, including tells from and to them.

#### [quit]

__Usage:__ `quit`

Shut down the bot.

### [reload]

__Usage:__ `reload`

Reload the bot's data from its SQL database and supplemental webpages.
