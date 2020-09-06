# Rooms Bot
Rooms Bot links text-channel and voice-channels together. The text-channels are invisible until a 
user joins the linked voice-channel.

## Active Bot
[Click Here!](https://discord.com/oauth2/authorize?client_id=750816469557837926&scope=bot&permissions=268438592)

## Commands
To add link a text-channel with a voice channel
 * .link `#channel` `voice channel ID`

To remove a link
 * .unlink `#text-channel` or `voice channel ID`

To list all your linked channels
 * .list


## Run Your own Instance

### Requirements
 * [Rust](https://www.rust-lang.org/tools/install)
 * [Git](https://git-scm.com/downloads)

### Setup
Open up your terminal / command line and run the following commands

```sh
git clone https://github.com/dylhack/rooms rooms
cd ./rooms
cargo build --release
cp ./target/release/rooms .
```

### Running
Now you can execute the rooms binary. Upon executing it for the first time it will generate a 
`config.yml` file.
```sh
./rooms
Please fill out the config.yml
```

### Notes
Set a `CONFIG_PATH` enviroment variable to set a custom path for the config.yml
