use signal_hook::{
    consts::signal::{SIGWINCH, SIGINT},
    iterator::{
        SignalsInfo,
        exfiltrator::origin::WithOrigin
    }
};
use std::env;
use std::io::{Stdout, stdout};
use std::thread;

mod ioctl;
mod parser;
mod ui;

mod tests;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let new_scheme = &args[1];
        set_scheme(new_scheme)?;
        return Ok(())
    }

    ioctl::unecho_noncanonical_cbreak_min_1()?;

    let signals = vec![SIGWINCH, SIGINT];
    let mut sig_info = SignalsInfo::<WithOrigin>::new(&signals)?;

    let mut stdout = stdout();

    ui::ansi_execute(ui::CURSOR_INVISIBLE, &mut stdout);

    // Clearing screen and setting cursor home is required
    // during debugging otherwise Rust compiler warnings gunk
    // things up.
    if cfg!(debug_assertions) {
        ui::ansi_execute(ui::SCREEN_CLEAR, &mut stdout);
        ui::ansi_execute(ui::CURSOR_HOME, &mut stdout);
    }

    thread::spawn(move || { event_loop() });

    for info in &mut sig_info {
        match info.signal {
            SIGWINCH => (), // TODO
            SIGINT | _ => restore_terminal_and_exit(&mut stdout)
        }
    }

    Ok(())
}

fn event_loop() {
    let (mut alacritty_conf, color_schemes) = match parser::find_alacritty_configs() {
        Ok(s) => s,
        Err(e) => panic!("{}", e)
    };

    let schemes = color_schemes.get_available_schemes().unwrap();

    let mut current_selection = 0;
    match alacritty_conf.get_current_scheme_name() {
        Err(_) => (),
        Ok(c) => {
            for (i, scheme) in schemes.iter().enumerate() {
                if *scheme == c {
                    current_selection = i;
                    break
                }
            }
        }
    }

    let mut list = ui::ScrollableList::new(schemes, current_selection, 5).unwrap();

    list.event_loop( move |selection: &str| -> Result<(), ui::Error> { 
        if false { return Err(ui::Error::CmdError) }

        let new_scheme = match color_schemes.get_scheme(&selection) {
            Ok(s) => s,
            Err(e) => panic!("{}", e)
        };

        alacritty_conf.set_scheme(&new_scheme);
        alacritty_conf.apply_scheme();

        Ok(())
    });
}

fn set_scheme(selection: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (mut alacritty_conf, color_schemes) = parser::find_alacritty_configs()?;

    let new_scheme = color_schemes.get_scheme(selection)?;

    alacritty_conf.set_scheme(&new_scheme);
    alacritty_conf.apply_scheme();

    Ok(())
}

fn restore_terminal_and_exit(stdout: &mut Stdout) {
    ui::ansi_execute(ui::CURSOR_VISIBLE, stdout);
    std::process::exit(1)
}

