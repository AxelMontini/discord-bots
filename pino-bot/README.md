# Pino bot

Collects statistics on the most used words in messages. It then randomly spews
out at random the most used ones.

## Configuration

| name         | required | description                                |
| ------------ | :------: | ------------------------------------------ |
| token        |   yes    | the discord token to use                   |
| interval_min |    no    | min interval between messages (in seconds) |
| interval_max |    no    | max interval between messages (in seconds) |
| exclude      |    no    | words to exclude from the statistics       |
