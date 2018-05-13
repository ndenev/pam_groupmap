# LDAP Group to User mapping module

[![Join the chat at https://gitter.im/pam_groupmap/Lobby](https://badges.gitter.im/pam_groupmap/Lobby.svg)](https://gitter.im/pam_groupmap/Lobby?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)
[![Build Status](https://travis-ci.org/ndenev/pam_groupmap.svg?branch=master)](https://travis-ci.org/ndenev/pam_groupmap)

## Description

This PAM service module can be used to map given user to another based
on LDAP group membership.
It can work only if used as PAM *accounting* module.

## Example

## Requirements

* Rust 1.18.0 or newer
* Working compiler.
* pkg-config, libssl-dev, libpam0g

## Installation

Compile and install the `.so`:

```shell
cargo build --release
sudo cp target/release/libpam_groupmap.so /lib/security/pam_groupmap.so
```

Create the config file `/etc/pam_groupmap.toml`:

```toml
# LDAP connection parameters
[ldap]
# Comma separated list of LDAP servers.
uri = "ldaps://ldap1.example.com:636,ldaps://ldap2.example.com:636"
# LDAP simple bind credentials (at the moment they are the same for all servers)
user = "XXX"
pass = "YYY"
#
# LDAP server connection timeout in seconds, default is 2.
# conn_timeout = 2
# LDAP server opeartion timeout in seconds (bind and search), default is 5.
# op_timeout = 5
#
# pam_groupmap will do an LDAP subtree search for the
# attribute $group_attribute under $user_base_dn with
# filter ($uid_attribute=$pam_username)
# Then the results are going to be filtered locally for
# only those that end with $group_base_dn
user_base_dn = "OU=people,OU=user,DC=example,DC=com"
group_base_dn = "OU=db,OU=groups,DC=example,DC=com"
uid_attribute = "sAMAccountName"
group_attribute = "memberOf"

# LDAP Group to User mappings
[mappings]
"dbadmin" = "dbadmin"
"dbreadonly" = "dbrouser"
"dbreadwrite" = "rbrwuser"
```

Make sure the config has the correct permissions:

```shell
chown root:mysql /etc/pam_groupmap.toml
chmod 640 /etc/pam_groupmap.toml
```

Setup PAM, for example for Percona XtraDB in `/etc/pam.d/mysqld`:

```pam
auth       requisite     pam_unix.so
account    requisite     pam_groupmap.so /etc/pam_groupmap.toml
```
