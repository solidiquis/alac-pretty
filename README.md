# Alac-pretty
---

https://user-images.githubusercontent.com/45523555/136723881-ee7b14ff-7242-4a8f-84c3-0908f7380f28.mp4

If you're like me in that you constantly need to change the colors of your dev environment because visual stagnation really bothers you, then get on [Alacritty](https://github.com/alacritty/alacritty) and download this BlAZiNgLy FAsT color-scheme shuffler. This only works for POSIX-compliant systems... sorry Windows peeps.

## Installation
Unfortunately I am too lazy to do releases and compile this for various systems so you'll have to do this manually for now.
1. Make sure you have [Rust and its toolchain](https://www.rust-lang.org/tools/install) installed.
2. `$ cargo install --git https://github.com/solidiquis/alac-pretty`
3. Stick the resultant binary somewhere in your path.

## Essential pre-requisites before using
The official Alacritty docs requires that you have your Alacritty config at one of the following locations:
1. $XDG_CONFIG_HOME/alacritty/alacritty.yml
2. $XDG_CONFIG_HOME/alacritty.yml
3. $HOME/.config/alacritty/alacritty.yml
4. $HOME/.alacritty.yml

To use this program, you'll need to stick [this additional file](https://github.com/solidiquis/alac-pretty/blob/master/assets/sample_alacritty_color_schemes.yml) in one of the following locations as well:
1. $XDG_CONFIG_HOME/alacritty/alacritty_color_schemes.yml
2. $XDG_CONFIG_HOME/alacritty_color_schemes.yml
3. $HOME/.config/alacritty/alacritty_color_schemes.yml
4. $HOME/.alacritty_color_schemes.yml

Lastly, your `alacritty.yml` file's `scheme` and `colors` settings will need to be formatted exactly like this [file](https://github.com/solidiquis/alac-pretty/blob/master/assets/sample_alacritty.yml).

## How to use
- Scrolling up: `k` or `↑`
- Scrolling down: `j` or `↓`
- Exiting: `Ctrl-c`

## To-do
- Better test coverage.
- Incorporate Github workflows + do releases
- Handle `SIGWINCH` signal.

## Acknowledgements
Thanks to [eendroroy](https://github.com/eendroroy) for putting together all the colorschemes which I used to put together [this bad boy](https://github.com/solidiquis/alac-pretty/blob/master/assets/sample_alacritty_color_schemes.yml).

## Author notes
I made [another version of this project](https://github.com/solidiquis/alacpretty) in Go but wasn't very happy with it. Decided to make a skinnier version in Rust for fun that only handled colorschemes and didn't leverage any Terminal UI libs.
