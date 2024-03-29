/*
This file is a part of mtd.

Copyright (C) 2022 Windore

Mtd is free software: you can redistribute it and/or modify it under the terms of the GNU General Public
License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later
version.

Mtd is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied
warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program. If not,
see <https://www.gnu.org/licenses/>.
 */

use std::{fs, io, process};
use std::io::Write;
use std::net::ToSocketAddrs;
use std::path::PathBuf;
use std::time::Duration;

use chrono::{Datelike, Local, NaiveDate};
use clap::{ArgEnum, Parser, Subcommand};
use rand::distributions::Alphanumeric;
use rand::Rng;

use mtd::{Config, Error, MtdNetMgr, Result, Task, TdList, Todo};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct CliArgs {
    #[clap(value_parser, long)]
    config_file: Option<PathBuf>,
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Shows specified items
    Show {
        /// Type of items to show.
        #[clap(arg_enum, value_parser, long, short)]
        item_type: Option<ItemType>,
        /// Weekday to show
        #[clap(arg_enum, value_parser, long, short, group = "show_days")]
        weekday: Option<Weekday>,
        /// Show entire week starting from today
        #[clap(value_parser, long, group = "show_days")]
        week: bool,
    },
    /// Adds a new item
    Add {
        /// Type of item to add
        #[clap(arg_enum, value_parser)]
        item_type: ItemType,
        /// Body of the item
        #[clap(value_parser)]
        body: String,
        /// Weekday(s) of the item
        #[clap(arg_enum, value_parser)]
        weekdays: Vec<Weekday>,
    },
    /// Removes an item
    Remove {
        /// Type of item to remove
        #[clap(arg_enum, value_parser)]
        item_type: ItemType,
        /// Id of the item to remove
        #[clap(value_parser)]
        id: u64,
    },
    /// Sets an item as done
    Do {
        /// Type of item to set the value(s) of
        #[clap(arg_enum, value_parser)]
        item_type: ItemType,
        /// Id of the item to set the value(s) of
        #[clap(value_parser)]
        id: u64,
    },
    /// Sets an item as undone
    Undo {
        /// Type of item to set the value(s) of
        #[clap(arg_enum, value_parser)]
        item_type: ItemType,
        /// Id of the item to set the value(s) of
        #[clap(value_parser)]
        id: u64,
    },
    /// Sets the value(s) of an item
    Set {
        /// Type of item to set the value(s) of
        #[clap(arg_enum, value_parser)]
        item_type: ItemType,
        /// Id of the item to set the value(s) of
        #[clap(value_parser)]
        id: u64,
        /// Set the body of the item
        #[clap(value_parser, long, short)]
        body: Option<String>,
        /// Set the weekday(s) of the item
        #[clap(arg_enum, value_parser, long, short)]
        weekdays: Vec<Weekday>,
    },
    /// Synchronizes local items with a server
    Sync,
    /// Runs mtd as a server
    Server,
    /// Re-initializes mtd
    /// (WARNING! This will completely delete all saved items!)
    ReInit,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum ItemType {
    Todo,
    Task,
}

// Define custom weekday for clap to parse weekdays.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum Weekday {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

impl Into<chrono::Weekday> for Weekday {
    fn into(self) -> chrono::Weekday {
        match self {
            Weekday::Mon => { chrono::Weekday::Mon }
            Weekday::Tue => { chrono::Weekday::Tue }
            Weekday::Wed => { chrono::Weekday::Wed }
            Weekday::Thu => { chrono::Weekday::Thu }
            Weekday::Fri => { chrono::Weekday::Fri }
            Weekday::Sat => { chrono::Weekday::Sat }
            Weekday::Sun => { chrono::Weekday::Sun }
        }
    }
}

fn main() {
    if let Err(e) = MtdApp::run() {
        eprintln!("{}", e);
        process::exit(1);
    } else {
        process::exit(0);
    }
}

struct MtdApp {
    conf: Config,
    list: TdList,
}

impl MtdApp {
    /// Initializes a new MtdApp. Reads/creates config and saved items.
    fn init(config_path: &PathBuf) -> Result<Self> {
        let conf;

        if config_path.exists() {
            conf = Config::new_from_json(&fs::read_to_string(config_path)?)?;
        } else {
            conf = MtdApp::create_new_config(config_path)?;
        }

        let list;

        // It is possible that a save_location has not been defined which needs to be checked before
        // checking if the path even exists.
        if let Some(list_path) = conf.save_location() {
            if list_path.exists() {
                list = TdList::new_from_json(
                    &fs::read_to_string(
                        list_path
                    )?
                )?;
            } else {
                list = MtdApp::create_new_list(&conf)?;
            }
        } else {
            list = MtdApp::create_new_list(&conf)?;
        }

        Ok(Self {
            conf,
            list,
        })
    }

    /// Creates a new TdList as a server or a client depending on user input.
    fn create_new_list(config: &Config) -> Result<TdList> {
        let mut buffer = String::new();
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        if config.local_only() {
            buffer = "c".to_string();
        } else {
            loop {
                print!("Initialize as a server or a client (s/c)? ");
                stdout.flush()?;
                buffer.clear();
                stdin.read_line(&mut buffer)?;
                buffer = buffer.to_lowercase().trim().to_string();

                if &buffer != "s" && &buffer != "c" {
                    eprintln!("Invalid option.");
                    continue;
                }
                break;
            }
        }

        if &buffer == "c" {
            Ok(TdList::new_client())
        } else {
            Ok(TdList::new_server())
        }
    }

    /// Returns the path to the config.
    fn default_config_path() -> Result<PathBuf> {
        Ok(dirs::config_dir().ok_or(Error::Unknown)?.join("mtd/conf.json"))
    }

    /// Returns the path to the default save location.
    fn default_save_path() -> Result<PathBuf> {
        Ok(dirs::data_dir().ok_or(Error::Unknown)?.join("mtd/data.json"))
    }

    /// Initializes a new config and writes it to a file.
    fn create_new_config(config_path: &PathBuf) -> Result<Config> {
        println!("Creating a new config.");

        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let mut local_only_inp_buf = String::new();

        loop {
            print!("Create a local only instance (y/n)? ");
            stdout.flush()?;
            local_only_inp_buf.clear();
            stdin.read_line(&mut local_only_inp_buf)?;
            local_only_inp_buf = local_only_inp_buf.to_lowercase().trim().to_string();

            if &local_only_inp_buf != "y" && &local_only_inp_buf != "n" {
                eprintln!("Invalid option.");
                continue;
            }
            break;
        }

        let local_only = &local_only_inp_buf == "y";
        let mut encryption_passwd;
        let mut socket_addr = String::new();

        if local_only {
            socket_addr = "127.0.0.1:55995".to_string();
            // Even though the random password wont be used in local only instances, I feel that
            // it is better to create a random password rather than hardcode some value.
            encryption_passwd = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(16)
                .map(char::from)
                .collect();
        } else {
            loop {
                print!("Input server socket address (ADDRESS:PORT): ");
                stdout.flush()?;
                socket_addr.clear();
                stdin.read_line(&mut socket_addr)?;
                socket_addr = socket_addr.trim().to_string();

                if socket_addr.to_socket_addrs().is_err() {
                    eprintln!("Cannot parse '{}' to socket address.", socket_addr);
                    continue;
                }
                break;
            }

            println!("Note! Encryption password is stored in cleartext but obfuscated locally.");

            let mut encryption_passwd_again;

            loop {
                encryption_passwd = rpassword::prompt_password("Input encryption password: ")?;
                encryption_passwd_again = rpassword::prompt_password("Input encryption password again: ")?;

                if encryption_passwd != encryption_passwd_again {
                    eprintln!("Passwords do not match.");
                    continue;
                } else if encryption_passwd.is_empty() {
                    eprintln!("Password cannot be empty.");
                    continue;
                }
                break;
            }
        }

        let mut save_location_buf = String::new();

        loop {
            print!("Input save path (Leave empty for default): ");
            stdout.flush()?;
            save_location_buf.clear();
            stdin.read_line(&mut save_location_buf)?;
            save_location_buf = save_location_buf.trim().to_string();

            if save_location_buf.parse::<PathBuf>().is_err() && &save_location_buf != "" {
                eprintln!("Cannot parse '{}' to path.", save_location_buf);
                continue;
            }
            break;
        }

        let save_path;

        if &save_location_buf == "" {
            save_path = MtdApp::default_save_path()?;
        } else {
            save_path = save_location_buf.parse().unwrap();
        }

        let conf = Config::new(
            socket_addr.parse().unwrap(),
            encryption_passwd.into_bytes(),
            Duration::from_secs(30),
            Some(save_path),
            local_only,
        );

        if let Some(conf_dir) = config_path.parent() {
            fs::create_dir_all(conf_dir)?;
        }
        fs::write(&config_path, conf.to_json()?)?;

        Ok(conf)
    }

    /// Runs the mtd cli app.
    fn run() -> Result<()> {
        let cli = CliArgs::parse();
        let config_path = cli.config_file.unwrap_or(MtdApp::default_config_path()?);

        let app;

        // Re-init is checked here because it should run without reading previous values.
        if let Commands::ReInit = &cli.command {
            app = MtdApp::re_init(&config_path)?;
        } else {
            app = MtdApp::init(&config_path)?.handle_command(cli.command)?;
        }

        if let Some(path) = app.conf.save_location() {
            if !path.exists() {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
            }
            fs::write(path, app.list.to_json()?)?;
        }

        Ok(())
    }

    // Needs to take ownership because syncing needs ownership
    fn handle_command(mut self, command: Commands) -> Result<Self> {
        match command {
            Commands::Show { item_type, weekday, week } => {
                self.show(item_type, weekday, week);
            }
            Commands::Add { item_type, weekdays, body } => {
                self.add(item_type, weekdays, body);
            }
            Commands::Remove { item_type, id } => {
                self.remove(item_type, id)?;
            }
            Commands::Do { item_type, id } => {
                self.modify_done_state(item_type, id, true)?;
            }
            Commands::Undo { item_type, id } => {
                self.modify_done_state(item_type, id, false)?;
            }
            Commands::Set { item_type, id, body, weekdays } => {
                self.set(item_type, id, body, weekdays)?;
            }
            Commands::Sync {} => {
                self.sync()?;
            }
            Commands::Server {} => {
                self.server()?;
            }
            // Re-init is handled earlier
            Commands::ReInit {} => {}
        }

        if self.conf.local_only() {
            self.list.self_sync();
        }

        Ok(self)
    }

    fn show(&self, item_type: Option<ItemType>, weekday_opt: Option<Weekday>, week: bool) {
        // If item type is None, show everything.
        let show_todos = item_type.is_none() || item_type.unwrap() == ItemType::Todo;
        let show_tasks = item_type.is_none() || item_type.unwrap() == ItemType::Task;

        if week {
            // Iterate over the next 7-days.
            let orig_wd = Local::today().weekday();
            let mut day = Local::today().naive_local();

            loop {
                // Print each day.
                self.print_date(day, show_todos, show_tasks);
                println!();

                day = day.succ();
                if day.weekday() == orig_wd {
                    break;
                }
            }
        } else {
            let weekday: chrono::Weekday;

            // If cli arg weekday is unspecified show today's weekday.
            if let Some(wd) = weekday_opt {
                weekday = wd.into();
            } else {
                weekday = Local::today().weekday();
            }

            self.print_date(mtd::weekday_to_date(weekday), show_todos, show_tasks);
        }
    }

    fn print_date(&self, date: NaiveDate, show_todos: bool, show_tasks: bool) {
        // Print weekday in yellow
        println!("\x1B[33m{}:\x1B[39m", date.weekday().to_string().to_uppercase());
        if show_todos {
            let undone_todos = self.list.undone_todos_for_date(date);
            let done_todos = self.list.done_todos_for_date(date);

            // Print header as green
            println!("\x1B[32mTodos:\x1B[39m");

            if undone_todos.len() + done_todos.len() == 0 {
                println!("\tNo todos for this day.");
            } else {
                for todo in undone_todos {
                    println!("\t{}", todo);
                }
                for todo in done_todos {
                    // Strikethrough and dim done todos.
                    println!("\t\x1B[2m\x1B[9m{}\x1B[0m", todo);
                }
            }
        }
        if show_tasks {
            let undone_tasks = self.list.undone_tasks_for_date(date);
            let done_tasks = self.list.done_tasks_for_date(date);

            // Print header as green
            println!("\x1B[32mTasks:\x1B[39m");

            if undone_tasks.len() + done_tasks.len() == 0 {
                println!("\tNo tasks for this day.");
            } else {
                for task in undone_tasks {
                    println!("\t{}", task);
                }
                for task in done_tasks {
                    // Strikethrough and dim done tasks.
                    println!("\t\x1B[2m\x1B[9m{}\x1B[0m", task);
                }
            }
        }
    }

    fn add(&mut self, item_type: ItemType, weekdays: Vec<Weekday>, body: String) {
        let mut chrono_weekdays: Vec<chrono::Weekday> = Vec::new();
        for wd in weekdays {
            chrono_weekdays.push(wd.into());
        }

        // If no weekdays are specified, add today's weekday.
        if chrono_weekdays.is_empty() {
            chrono_weekdays.push(Local::today().weekday());
        }

        match item_type {
            ItemType::Todo => {
                for day in chrono_weekdays {
                    self.list.add_todo(Todo::new_dated(body.clone(), day));
                }
            }
            ItemType::Task => {
                self.list.add_task(Task::new(body, chrono_weekdays));
            }
        }
    }

    fn remove(&mut self, item_type: ItemType, id: u64) -> Result<()> {
        match item_type {
            ItemType::Todo => {
                self.list.remove_todo(id)?;
            }
            ItemType::Task => {
                self.list.remove_task(id)?;
            }
        }
        Ok(())
    }

    fn modify_done_state(&mut self, item_type: ItemType, id: u64, to_done: bool) -> Result<()> {
        match item_type {
            ItemType::Todo => {
                self.list.get_todo_mut(id)?.set_done(to_done);
            }
            ItemType::Task => {
                let task = self.list.get_task_mut(id)?;
                let mut next_date_for_task = Local::today().naive_local();
                while !task.for_date(next_date_for_task) {
                    next_date_for_task = next_date_for_task.succ();
                }
                task.set_done(to_done, next_date_for_task);
            }
        }
        Ok(())
    }

    fn set(&mut self, item_type: ItemType, id: u64, body: Option<String>, weekdays: Vec<Weekday>) -> Result<()> {
        let mut chrono_weekdays: Vec<chrono::Weekday> = Vec::new();
        for wd in weekdays {
            chrono_weekdays.push(wd.into());
        }

        match item_type {
            ItemType::Todo => {
                let todo = self.list.get_todo_mut(id)?;
                if let Some(b) = body {
                    todo.set_body(b);
                }
                if chrono_weekdays.len() >= 1 {
                    todo.set_weekday(chrono_weekdays[0]);
                }
            }
            ItemType::Task => {
                let task = self.list.get_task_mut(id)?;
                if let Some(b) = body {
                    task.set_body(b);
                }
                if chrono_weekdays.len() >= 1 {
                    task.set_weekdays(chrono_weekdays);
                }
            }
        }

        Ok(())
    }

    fn sync(&mut self) -> Result<()> {
        let conf = &self.conf;

        let mut net_mgr = MtdNetMgr::new(&mut self.list, conf);

        net_mgr.client_sync()
    }

    fn server(&mut self) -> Result<()> {
        let conf = &self.conf;

        let mut net_mgr = MtdNetMgr::new(&mut self.list, &conf);

        net_mgr.server_listening_loop()
    }

    fn re_init(config_path: &PathBuf) -> Result<Self> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        let mut buffer = String::new();

        loop {
            print!("This will delete all items and erase the config. Proceed (y/n)? ");
            stdout.flush()?;
            buffer.clear();
            stdin.read_line(&mut buffer)?;
            buffer = buffer.to_lowercase().trim().to_string();

            if &buffer != "y" && &buffer != "n" {
                eprintln!("Invalid option.");
                continue;
            }
            break;
        }

        if &buffer == "n" {
            println!("Abort!");
            // This is not optimal, but is the easiest way to implement this.
            process::exit(0);
            // Other option would be to call, but in some cases that could seem like the abort didn't do anything
            // return Ok(MtdApp::new(config_path)?);
        }

        let config = MtdApp::create_new_config(&config_path)?;

        Ok(Self {
            list: MtdApp::create_new_list(&config)?,
            conf: config,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;

    use chrono::{Datelike, Local};

    use mtd::{Config, Task, TdList, Todo};

    use crate::{Commands, ItemType, MtdApp, Weekday};

    fn create_client_app() -> MtdApp {
        MtdApp {
            conf: Config::new_default("SecurePw".as_bytes().to_vec(), "127.0.0.1:55980".to_string(), None),
            list: TdList::new_client(),
        }
    }

    fn create_server_app() -> MtdApp {
        MtdApp {
            conf: Config::new_default("SecurePw".as_bytes().to_vec(), "127.0.0.1:55980".to_string(), None),
            list: TdList::new_server(),
        }
    }

    #[test]
    fn add_adds_todo_successfully() {
        let mut client = create_client_app();
        client.add(ItemType::Todo, vec![Weekday::Wed], "Todo".to_string());
        assert_eq!(client.list.todos()[0], &Todo::new_dated("Todo".to_string(), chrono::Weekday::Wed));
    }

    #[test]
    fn add_adds_task_successfully() {
        let mut client = create_client_app();
        client.add(ItemType::Task, vec![Weekday::Wed, Weekday::Fri, Weekday::Sun], "Task".to_string());
        assert_eq!(client.list.tasks()[0], &Task::new("Task".to_string(), vec![chrono::Weekday::Wed, chrono::Weekday::Fri, chrono::Weekday::Sun]))
    }

    #[test]
    fn add_adds_task_without_explicit_weekday() {
        let mut client = create_client_app();
        client.add(ItemType::Task, vec![], "Task".to_string());
        assert_eq!(client.list.tasks()[0], &Task::new("Task".to_string(), vec![Local::today().weekday()]))
    }

    #[test]
    fn add_adds_todo_to_multiple_weekdays() {
        let mut client = create_client_app();
        client.add(ItemType::Todo, vec![Weekday::Wed, Weekday::Fri, Weekday::Sun], "Todo".to_string());
        assert_eq!(client.list.todos()[0], &Todo::new_dated("Todo".to_string(), chrono::Weekday::Wed));
        assert_eq!(client.list.todos()[1], &Todo::new_dated("Todo".to_string(), chrono::Weekday::Fri));
        assert_eq!(client.list.todos()[2], &Todo::new_dated("Todo".to_string(), chrono::Weekday::Sun));
    }

    #[test]
    fn remove_removes_todo_successfully() {
        let mut client = create_client_app();
        client.list.add_todo(Todo::new_undated("Todo".to_string()));
        client.remove(ItemType::Todo, 0).unwrap();
        assert_eq!(client.list.todos().len(), 0);
    }

    #[test]
    fn remove_removes_task_successfully() {
        let mut client = create_client_app();
        client.list.add_task(Task::new("Task".to_string(), vec![chrono::Weekday::Sun]));
        client.remove(ItemType::Task, 0).unwrap();
        assert_eq!(client.list.tasks().len(), 0);
    }

    #[test]
    fn modify_done_state_sets_todo_done() {
        let mut client = create_client_app();
        client.list.add_todo(Todo::new_undated("Todo".to_string()));
        client.modify_done_state(ItemType::Todo, 0, true).unwrap();
        assert!(client.list.todos()[0].done());
    }

    #[test]
    fn modify_done_state_sets_task_done_for_the_next_correct_date() {
        let mut client = create_client_app();
        client.list.add_task(Task::new("Task".to_string(), vec![Local::today().weekday().succ().succ()]));
        client.modify_done_state(ItemType::Task, 0, true).unwrap();
        assert!(client.list.tasks()[0].done(Local::today().naive_local().succ().succ()));
    }

    #[test]
    fn set_sets_todo_values_to_new() {
        let mut client = create_client_app();
        client.list.add_todo(Todo::new_dated("Todo".to_string(), chrono::Weekday::Sun));
        client.set(ItemType::Todo, 0, Some("New Todo".to_string()), vec![Weekday::Wed]).unwrap();
        assert_eq!(client.list.todos()[0], &Todo::new_dated("New Todo".to_string(), chrono::Weekday::Wed));
    }

    #[test]
    fn set_sets_task_values_to_new() {
        let mut client = create_client_app();
        client.list.add_task(Task::new("Task".to_string(), vec![chrono::Weekday::Sun]));
        client.set(ItemType::Task, 0, Some("New Task".to_string()), vec![Weekday::Thu, Weekday::Fri]).unwrap();
        assert_eq!(client.list.tasks()[0], &Task::new("New Task".to_string(), vec![chrono::Weekday::Thu, chrono::Weekday::Fri]))
    }

    #[test]
    fn set_doesnt_modify_weekday_without_explicit_set() {
        let mut client = create_client_app();
        client.list.add_todo(Todo::new_dated("Todo".to_string(), chrono::Weekday::Sun));
        client.set(ItemType::Todo, 0, Some("New Todo".to_string()), vec![]).unwrap();
        assert_eq!(client.list.todos()[0], &Todo::new_dated("New Todo".to_string(), chrono::Weekday::Sun));
    }

    #[test]
    fn set_doesnt_modify_body_without_explicit_set() {
        let mut client = create_client_app();
        client.list.add_task(Task::new("Task".to_string(), vec![chrono::Weekday::Sun]));
        client.set(ItemType::Task, 0, None, vec![Weekday::Thu, Weekday::Fri]).unwrap();
        assert_eq!(client.list.tasks()[0], &Task::new("Task".to_string(), vec![chrono::Weekday::Thu, chrono::Weekday::Fri]))
    }

    #[test]
    fn sync_as_server_fails() {
        assert!(create_server_app().sync().is_err());
    }

    #[test]
    fn server_as_client_fails() {
        assert!(create_client_app().server().is_err());
    }

    #[test]
    fn syncing_works() {
        thread::spawn(|| {
            let mut server = create_server_app();
            server.list.add_todo(Todo::new_undated("Todo".to_string()));
            server.server().unwrap();
        });

        // Give server time to init
        thread::sleep(Duration::from_millis(500));

        let mut client = create_client_app();
        client.sync().unwrap();

        assert_eq!(client.list.todos().len(), 1);
        assert!(client.list.todos().contains(&&Todo::new_undated("Todo".to_string())));
    }

    #[test]
    fn local_only_syncs_with_self_automatically() {
        let mut app = MtdApp {
            list: TdList::new_client(),
            conf: Config::new(
                "127.0.0.1:55995".to_string(),
                "pw".as_bytes().to_vec(),
                Duration::from_secs(30),
                None,
                true,
            ),
        };
        app.list.add_todo(Todo::new_undated("This string doesn't remain if the todo is actually removed.".to_string()));

        // Do assert here to first check that the save format hasn't changed and will contain the todo in cleartext.
        assert!(app.list.to_json().unwrap().contains("This string doesn't remain if the todo is actually removed."));

        let app = app.handle_command(Commands::Remove { item_type: ItemType::Todo, id: 0 }).unwrap();

        assert!(!app.list.to_json().unwrap().contains("This string doesn't remain if the todo is actually removed."));
    }
}