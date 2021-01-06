# Pino bot

Collects statistics on the most used words in messages. It then randomly spews
out at random the most used ones.

## Configuration

Run pino with the `--help` option to get up-to-date information

| name         | required | description                                              |
| ------------ | :------: | -------------------------------------------------------- |
| token        |   yes    | the discord token to use                                 |
| interval-min |    no    | min interval between messages (in seconds)               |
| interval-max |    no    | max interval between messages (in seconds)               |
| max-age      |    no    | Words older than this duration (in seconds) get deleted  |
| exclude      |    no    | words to exclude from the statistics                     |
| max-boost    |    no    | max random boost to a word count                         |
| default-word |    no    | If specified, default word to print if there was silence |
