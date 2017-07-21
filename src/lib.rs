extern crate libc;
extern crate toml;
extern crate ldap3;
extern crate serde;
#[macro_use]
extern crate serde_derive;

mod pam;
mod config;

use std::time::Duration;
use std::collections::BTreeSet;
use ldap3::{LdapConn, LdapConnBuilder, Scope, SearchEntry};
use ldap3::result::SearchResult;
use ldap3::ldap_escape;

// Re-export the PAM callbacks
pub use pam::callbacks::*;
use pam::{PamResultCode, get_user, set_user};
use config::Config;


#[allow(dead_code)]
fn spare() {
    println!("");
    panic!("");
}

pub fn acct_mgmt(pamh: pam::PamHandleT, args: Vec<String>, silent: bool) -> PamResultCode {

    let user = match get_user(pamh) {
        Ok(u) => u,
        Err(_) => return PamResultCode::PAM_AUTH_ERR,
    };

    if args.len() != 1 {
        return PamResultCode::PAM_SERVICE_ERR;
    }

    let config = match Config::load(&args[0]) {
        Ok(c) => c,
        Err(e) => {
            if !silent {
                println!("ERROR: {:?}", e);
            }
            return PamResultCode::PAM_SERVICE_ERR;
        }
    };

    let lconn = match ldap_connect(&config.ldap) {
        Ok(c) => c,
        Err(_) => return PamResultCode::PAM_SERVICE_ERR,
    };

    let groups = match get_user_groups(&lconn, &config.ldap, &user) {
        Ok(g) => g,
        Err(e) => return e,
    };

    let mut ret = PamResultCode::PAM_AUTH_ERR;
    for (group, mapped_user) in config.mappings.iter() {
        if groups.contains(group) {
            if !silent {
                println!("Mapping {} -> {}", user, mapped_user);
            }
            match set_user(pamh, mapped_user.clone()) {
                Ok(_) => {
                    ret = PamResultCode::PAM_SUCCESS;
                    break;
                }
                Err(_) => {
                    ret = PamResultCode::PAM_SERVICE_ERR;
                    break;
                }
            }
        };
    }
    lconn.unbind().ok();
    ret
}

fn extract_ldap_servers<'a>(uri: &'a String) -> Vec<&'a str> {
    uri.split(',').collect::<Vec<_>>()
}

#[test]
fn test_extract_ldap_servers() {
    assert_eq!(
        extract_ldap_servers(&String::from("asdf,qwer")),
        vec!["asdf", "qwer"]
    );
}

fn ldap_connect(ldap: &config::LdapConfig) -> Result<LdapConn, ()> {
    let servers = extract_ldap_servers(&ldap.uri);

    for server in servers {
        println!("Trying to connect to {}", server);
        let lconn = match LdapConnBuilder::<LdapConn>::new()
            .with_conn_timeout(Duration::from_secs(ldap.conn_timeout))
            .connect(server) {
            Ok(c) => {
                println!("Connected to {}", server);
                c
            }
            Err(_) => {
                println!("Failed to connect to {}, trying next.", server);
                continue;
            }
        };
        match lconn
            .with_timeout(Duration::from_secs(ldap.op_timeout))
            .simple_bind(&ldap.user, &ldap.pass) {
            Ok(_) => {
                println!("Simple Bind Succeeded");
                return Ok(lconn);
            }
            Err(_) => {
                println!("Simple Bind Failed, trying next.");
                continue;
            }
        }
    }
    println!("Server list exhausted.");
    Err(())
}

fn get_user_groups(
    lconn: &LdapConn,
    config: &config::LdapConfig,
    user: &String,
) -> Result<BTreeSet<String>, PamResultCode> {
    lconn
        .with_timeout(Duration::from_secs(config.op_timeout))
        .search(
            &config.user_base_dn,
            Scope::Subtree,
            &format!("({}={})", config.uid_attribute, ldap_escape(user.as_ref())),
            vec![config.group_attribute.clone()],
        )
        .and_then(SearchResult::success)
        .map(|(rset, _)| {
            rset
                .into_iter()
                .map(|r| SearchEntry::construct(r))
                .flat_map(|e| e.attrs)
                .filter_map(|e| if e.0 == config.group_attribute {
                    Some(e.1)
                } else {
                    None
                })
                .flat_map(|e| e)
                .filter(|e| {
                    e.to_lowercase().ends_with(
                        &config.group_base_dn.to_lowercase(),
                    )
                })
                .filter_map(|e| if let Some(end) = e.find(",") {
                    Some(e[3..end].to_owned())
                } else {
                    None
                })
                .collect::<BTreeSet<_>>()
        })
        .map_err(|_| PamResultCode::PAM_SERVICE_ERR)
}
