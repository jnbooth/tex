# [Tex]
A different approach to chatbot commands.

## Commands

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
