use crate::user::User;
use crate::utils::password;
use crate::vault::DBError::{BadPassword, Unauthenticated, UnknownUser};
use crate::vault::{DBResult, Vault};

use bcrypt::verify;
use clap::ArgMatches;
use rusqlite::params;

#[derive(Debug)]
pub struct Authentication {
    pub login: String,
    pub password: String,
}

impl Authentication {
    pub fn from(matches: &ArgMatches) -> Option<Self> {
        let login = matches.value_of("user");
        if login.is_some() {
            Some(Authentication {
                login: login.unwrap().to_string(),
                password: password(matches.value_of("password"), None),
            })
        } else {
            None
        }
    }
}

impl Vault {
    pub fn authenticate_user(&self, auth: &Option<Authentication>) -> DBResult<User> {
        auth.as_ref().map_or(Err(Unauthenticated), |a| {
            self.connection
                .query_row(
                    "SELECT id, login, password FROM users WHERE login = ?1",
                    params![a.login],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .map_or(Err(UnknownUser), |user: (i64, String, String)| {
                    if verify(&a.password, &user.2).unwrap_or(false) {
                        Ok(User::new(user.0, &user.1))
                    } else {
                        Err(BadPassword)
                    }
                })
        })
    }
}
