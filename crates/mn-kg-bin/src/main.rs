use anyhow::Context;
use dialoguer::{
    console::Term,
    theme::{ColorfulTheme}, Input, MultiSelect,
};
use mn_kg::{generate_serial, App, IntoEnumIterator};

fn main() -> anyhow::Result<()> {
    let term = Term::buffered_stderr();
    let theme = ColorfulTheme::default();

    let username: String = Input::with_theme(&theme)
        .with_prompt("Username")
        .interact_on(&term)
        .context("No username")?;

    if username.trim().is_empty() {
        println!("Username is empty or blank.");
        return Ok(());
    }

    let apps: Vec<App> = App::iter().collect();

    let softwares = MultiSelect::with_theme(&theme)
        .with_prompt("Softwares to generate")
        .items(&apps)
        .defaults(&vec![true; apps.len()])
        .interact_on(&term)
        .context("No software selected")?;

    if softwares.is_empty() {
        println!("No software selected, exiting...");
        return Ok(());
    }

    println!("Owner: {username}");
    apps.into_iter()
        .enumerate()
        .filter(|(idx, _)| softwares.contains(idx))
        .for_each(|(_, app)| {
            let serial = generate_serial(&username, app).unwrap();
            println!("{serial} for {app}");
        });

    Ok(())
}
