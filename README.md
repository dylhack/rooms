# Rooms Bot
This bot automatically hides text-channels until a user is in a "linked" voice channel.

## Active Bot
[Click Here!](https://discord.com/api/oauth2/authorize?client_id=750816469557837926&permissions=3152&scope=bot)

## Commands
To add link a text-channel with a voice channel
 * .link add `#channel` `voice channel ID`

To remove a link
 * .link remove `#channel` or `voice channel ID`

To list all your linked channels
 * .link list


## Run Your own Instance

### Requirements
 * [Rust](https://www.rust-lang.org/tools/install)
 * [Git](https://git-scm.com/downloads)

### Setup
Open up your terminal / command line and run the following commands

```sh
$ git clone https://github.com/dylhack/rooms
$ cd rooms/
$ cargo build --release
$ cp ./target/release/rooms .
```

### Running
Now you can execute the rooms binary. Upon executing it for the first time it will generate a 
`config.yml` file.
```sh
$ ./rooms
Please fill out the config.yml
```
