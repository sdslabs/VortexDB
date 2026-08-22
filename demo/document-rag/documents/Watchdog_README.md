# Watchdog

> Lightweight server access management system

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/sdslabs/watchdog/blob/master/LICENSE.md)

Watchdog is a personalised server access management tool (and a slack bot) which keeps a track of all the administrative rights attempts (like sudo and su) on server (via SSH) and allows/disallows log-in attempts based on public key of user and logs all activity in form of slack message. It provides easy granting/revoking access to servers to team members through pull requests on a keyhouse repository.

Check out this blog post to know how watchdog works and design methodologies behind it: https://blog.sdslabs.co/2020/04/watchdog

## Contents

* [Features](#features)
* [Dependencies](#dependencies)
* [Installation](#installation)
* [Usage](#usage)
* [Development](#development)
* [Contact](#contact)


## Features

* Request SSH access to a server just by creating a PR to the Keyhouse repository.
* Stateless and serverless. Watchdog runs on a single binary.
* Optional server activity logs to your favourite workspace like Slack or Discord.
* Easy Installation and Configuration
* Get notified when someone escalates privileges or performs administrative tasks using `sudo` or `su`

## Dependencies

The following softwares are required for running Watchdog:-

* PAM
* OpenSSH server

## Installation

1. Create a Keyhouse Repository using the template repository [here](https://github.com/sdslabs/keyhouse-template).

2. Clone the watchdog repository

    `git clone https://github.com/sdslabs/watchdog.git`

3. Change into the repository directory and build the latest binaries using Cargo

    `cargo build --release`

4. Copy `sample.config.toml` to `config.toml` and make changes to the config this way:

    ```toml
    # Hostname of the machine running watchdog. Note that this should be
    # same as the file you create in the `hosts` directory in keyhouse.
    hostname = 'virtual-machine'

    # Keyhouse repository configuration
    [keyhouse]

    # URL of the Keyhouse repository, it should be of the format
    # `https://api.github.com/repos/<ORGANIZATION>/<KEYHOUSE-REPOSITORY>/contents`
    base_url = 'https://api.github.com/repos/sdslabs/keyhouse-template/contents'

    # This should be a personal access token made by a member of organization on his/her
    # behalf who can read the Keyhouse repository. Go to this
    # https://github.com/settings/tokens/new?description=Keyhouse%20Token&scopes=repo
    # to make a new token with correct scopes.
    token = 'secret_token'

    # Webhook APIs corresponding to various notifiers
    [notifiers]

    # Make an incoming hook to your Slack workspace from this
    # app(https://slack.com/apps/A0F7XDUAZ-incoming-webhooks)
    # and paste the hook URL here. You can customize the icon and name as you like.
    slack = 'https://hooks.slack.com/services/ABCDEFGHI/ABCDEFGHI/abcdefghijklmnopqrstuvwx'
    ```

5. Once you are done configuring, run this command with root(sudo) privileges

    `cd install && sudo ./install.sh`

6. Add `/opt/watchdog/bin` to your PATH variable.

## Usage

```
$ watchdog --help

Watchdog 0.1.0
SDSLabs <contact@sdslabs.co>
Simple server access management system on a binary

USAGE:
    watchdog [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    auth      Authorizes users based on from keyhouse repository. This command is passed through
              `AuthorizedKeysCommand` in sshd_config.
    config    Get or set Watchdog configuration
    help      Prints this message or the help of the given subcommand(s)
    logs      Get the global watchdog logs
    ssh       Handles the PAM SSH calls by pam_exec for Watchdog
    su        Handles the PAM su calls by pam_exec for Watchdog
    sudo      Handles the PAM sudo calls by pam_exec for Watchdog
```

Though most of the commands are for internal use of PAM, you can edit configuration of Watchdog any time

```sh
$ watchdog config --help
```

_NOTE:_ config can be fetched/edited only with `root` (`sudo`) access.

To view logs

```sh
$ watchdog logs --help
```

## Development

You need to have [Rust](https://www.rust-lang.org/tools/install) installed along with the mentioned [dependencies](#dependencies)

Open your favourite terminal and perform the following tasks:-

1. Clone this repository.

```bash
$ git clone https://github.com/sdslabs/watchdog
```

2. Make the required changes inside the source code directory ([src/](src/))

3. Run `cargo test` to test your changes.

4. Rebuild the binary using `cargo build` command.

## Contact

If you have a query regarding the product or just want to say hello then feel free to visit
[chat.sdslabs.co](http://chat.sdslabs.co/) or drop a mail at [contact@sdslabs.co.in](mailto:contact@sdslabs.co.in)
