mod db;
mod link;
mod query;
mod user;
mod utils;

use crate::link::Link;
use crate::user::Authentication;
use clap::{App, Arg, ArgMatches, SubCommand};
use db::{init_vault, Vault};
use log::Level;
use semver::Version;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    simple_logger::init_with_level(Level::Debug).unwrap();

    let matches = App::new("Świnka skarbonka na linki")
        .version(VERSION)
        .about("Saves your precious links into local vault")
        .arg(
            Arg::with_name("database")
                .help("database to use")
                .short("d")
                .long("db")
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("add")
                .about("Adds a new link")
                .arg(
                    Arg::with_name("url")
                        .help("link to store in database")
                        .required(true),
                )
                .arg(
                    Arg::with_name("user")
                        .help("an owner of stored link")
                        .short("u")
                        .long("user")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("password")
                        .help("owner's password")
                        .short("p")
                        .long("pass")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("description")
                        .help("optional description")
                        .short("d")
                        .long("desc")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("tags")
                        .help("optional tags assigned to the link")
                        .short("t")
                        .long("tags")
                        .use_delimiter(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("ls")
                .about("Lists matching links")
                .arg(Arg::with_name("url").help("link or its part to match"))
                .arg(
                    Arg::with_name("description")
                        .help("optional part of description to match")
                        .short("d")
                        .long("desc")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("user")
                        .help("an owner of stored link")
                        .short("u")
                        .long("user")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("tags")
                        .help("optional comma-separated tags to match")
                        .short("t")
                        .long("tags")
                        .use_delimiter(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("users")
                .about("Manage with users")
                .subcommand(
                    SubCommand::with_name("add")
                        .about("Adds a new user")
                        .arg(
                            Arg::with_name("user")
                                .help("User's identifier (login)")
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("password")
                                .help("user's password")
                                .short("p")
                                .long("pass")
                                .takes_value(true),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("passwd")
                        .about("Changes user's password")
                        .arg(
                            Arg::with_name("user")
                                .help("User's identifier (login)")
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("password")
                                .help("user's new password")
                                .short("p")
                                .long("pass")
                                .takes_value(true),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("ls")
                        .about("Lists matching users")
                        .arg(Arg::with_name("login").help("User's identifier pattern to list")),
                ),
        )
        .get_matches();

    let db = matches.value_of("database").unwrap_or("links.db");
    match init_vault(db, Version::parse(VERSION).unwrap()) {
        Ok(v) => process_command(v, matches),
        _ => panic!("cannot initialize database"),
    }
}

fn process_command(mut vault: Vault, matches: ArgMatches) {
    match matches.subcommand() {
        ("add", Some(sub_m)) => {
            match vault.add_link(&Link::from(sub_m), &Authentication::from(sub_m)) {
                Ok(link) => println!("{}", link),
                Err(e) => println!("Error while adding a link ({:?})", e),
            }
        }
        ("ls", Some(sub_m)) => {
            match vault.match_links(&Link::from(sub_m), &Authentication::from(sub_m)) {
                Ok(links) => {
                    for link in links {
                        println!("{}", link)
                    }
                }
                Err(e) => println!("Error while fetching links ({:?}).", e),
            }
        }
        ("users", Some(sub_m)) => match sub_m.subcommand() {
            ("add", Some(sub_m)) => match vault.add_user(&Authentication::from(sub_m)) {
                Ok(_) => println!("Ok."),
                Err(_) => println!("Error while adding new user. User might already exist."),
            },
            ("passwd", Some(sub_m)) => match vault.passwd_user(&Authentication::from(sub_m)) {
                Ok(_) => println!("Changed."),
                Err(e) => println!("Error while changing password ({:?}).", e),
            },
            ("ls", Some(sub_m)) => match vault.match_users(sub_m.value_of("login")) {
                Ok(users) => {
                    for (user, count) in users {
                        println!("{} ({})", user, count);
                    }
                }
                Err(_) => println!("Error while fetching users."),
            },
            _ => (),
        },
        _ => {}
    }
}
