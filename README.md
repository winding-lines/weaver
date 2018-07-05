# Description

Weaver is an improved browser and shell history tracker. Weaver is composed of 4 components:

- command line extension
- chrome extension
- backend server
- store

# Concepts

Weaver tracks actions, these actions can be generated from the shell or from the browser. The
actions are saved in the backend store.

In order to simplify the grouping of actions you can associate actions with a an epic. An epic is
simply the name of the task that you workting towards completing.

# Components

## Local component

Since this project is under heavy development you need to build it from sources.
Follow the following steps to install the local server/cli component.

- Install rust from [https://rustup.rs/](https://rustup.rs/)
- Checkout weaver from git@gitlab.com:lab-flow/weaver.git
- Install required libraries, on Linux
    
    `sudo apt-get install libncurses-dev libsqlite3-dev lisqlite3 libncursesw5-dev`
    
- Build with `cargo build -all --release`
- Install in your `$PATH`
- Create all the data stores `weaver-data setup`
- Start the server with `weaver-server start`

Every time your reboot your computer you will need to restart the server.

See the documentation for weaver-data and weaver-server on how to setup a staging environment for development.


## Chrome integration

### Extension

Please install the Chrome Extension from the Chrome Store.
https://chrome.google.com/webstore/detail/weaver/hcijijnmldaljacnomfkjibcnobpbpmk

### Search engine

Add a custom search engine here:

chrome://settings/searchEngines

My parameters are:

  - Search engine: weaver
  - Keyword: wr
  - URL: http://localhost:8466/?term=%s 

## Command line integration

The command line extension has two goals:

1. capture the history as you type
2. allow you to search in the history

### Installation

In order to install the extension you need to change your shell configuration. Currently we only
have instructions for bash. After you install `weaver` in the path, change your `$HOME/.bashrc` to
include instructions similar to the following.

```
export PS1='{$( fc -ln -1 | weaver prompt)} \W $ '

# Bind Ctrl-x w to run the weaver action
bind '"\C-xw":"weaver actions\n"'

# Bind Ctrl-x c to copy to clipboard the selected weaver action
bind '"\C-xc":"weaver actions -c\n"'
```

### Usage

Type `weaver --help` to get more help. Two frequent use cases are:
- changing the epic, this is done with the `weaver epic` command line. The active epic is displayed
  in the shell command, if you have installed as above.
- re-running a command, this is done with the `weaver actions` command

