<h1 align="center"> cliphoard </h1>
<p align="center"><i>a quick and simple way to expand your clipboard</i>.</p>

Cliphoard started as a solution for me to fill out job application more efficiently by avoiding the need to switch tabs to copy my job descriptions everytime I was asked. After getting a working clipboard manager, I found I was using it for more than just job applications and began using it to save snippets like CLI commands and boilerplate code.

Cliphoard was inspired by popup application launchers, like Rofi and dmenu, and is also intended to be launched with keybindings by using a window manager.

![Peek 2025-04-02 18-52](https://github.com/user-attachments/assets/f7c7119c-69b3-4c03-a628-82a12bceee59)

## Features
- i3 keybinding launch
- window customization
- fuzzy searching snippets
- optional snippet nicknames for easy search

## Requirements
- SDL2 >= 2.26.0 (required to capture text selection)
- xdotool
- rustup

## Installation
I made cliphoard on Mint 22 with i3, but hasn't been tested on other distributions (please let me know how it runs on your machine!).

Clone this repository to any directory
````bash
git clone https://github.com/christianc521/cliphoard.git && cd cliphoard
````

Build from source
````bash
cargo build
````

Install with Cargo
````bash
cargo install --path .
# You can test the installation by running:
cliphoard --copy
cliphoard
````

Add keybindings to your window manager config
````
# i3wm config
bindsym $mod+c exec "cliphoard --copy"
bindsym $mod+v exec "cliphoard"
````


