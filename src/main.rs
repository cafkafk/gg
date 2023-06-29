//    A Rust GitOps/symlinkfarm orchestrator inspired by GNU Stow.
//    Copyright (C) 2023  Christina Sørensen <christina@cafkafk.com>
//
//    This program is free software: you can redistribute it and/or modify
//    it under the terms of the GNU General Public License as published by
//    the Free Software Foundation, either version 3 of the License, or
//    (at your option) any later version.
//
//    This program is distributed in the hope that it will be useful,
//    but WITHOUT ANY WARRANTY; without even the implied warranty of
//    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//    GNU General Public License for more details.
//
//    You should have received a copy of the GNU General Public License
//    along with this program.  If not, see https://www.gnu.org/gpl-3.0.html.

extern crate log;
extern crate pretty_env_logger;

#[allow(unused)]
mod cli;
#[allow(unused)]
mod git;
#[allow(unused)]
mod utils;

use cli::{Args, Commands};
use git::Config;

use clap::Parser;
#[allow(unused)]
use log::{debug, error, info, trace, warn};

fn main() {
    pretty_env_logger::init();
    let args = Args::parse();
    let config = Config::new(&args.config);
    match &args {
        args if args.license => println!("{}", utils::strings::INTERACTIVE_LICENSE),
        args if args.warranty => println!("{}", utils::strings::INTERACTIVE_WARRANTY),
        args if args.code_of_conduct => unimplemented!(),
        _ => (),
    }
    match &args.command {
        Some(Commands::Link { msg: _ }) => {
            config.link_all();
        }
        Some(Commands::Quick { msg }) => {
            config.quick(msg.as_ref().get_or_insert(&"gg: quick commit".to_string()));
        }
        Some(Commands::Clone { msg: _ }) => {
            config.clone_all();
        }
        Some(Commands::Pull { msg: _ }) => {
            config.pull_all();
        }
        Some(Commands::Add { msg: _ }) => {
            config.add_all();
        }
        Some(Commands::Commit { msg: _ }) => {
            config.commit_all();
        }
        Some(Commands::CommitMsg { msg }) => {
            config.commit_all_msg(msg.as_ref().unwrap());
        }
        None => (),
    }
    trace!("{:?}", config);
}

#[cfg(test)]
mod config {
    use crate::*;
    use git::RepoFlags::{Clone, Push};
    use git::{Category, GitRepo};
    use relative_path::RelativePath;
    use std::collections::HashMap;
    use std::env::current_dir;
    use std::fs::File;
    use std::io::prelude::*;
    #[test]
    fn init_config() {
        let _config = Config {
            categories: HashMap::new(),
            links: vec![],
        };
    }
    #[test]
    fn init_config_populate() {
        let default_category = Category {
            flags: vec![],
            repos: HashMap::new(),
        };
        let mut config = Config {
            categories: HashMap::new(),
            links: vec![],
        };
        config
            .categories
            .insert(format!("{}", 0).to_string(), default_category);
        for i in 0..=5 {
            config
                .categories
                .get_mut(&format!("{}", 0).to_string())
                .expect("category not found")
                .repos
                .insert(
                    format!("{}", i).to_string(),
                    GitRepo {
                        name: "test repo".to_string(),
                        path: "/tmp".to_string(),
                        url: "https://github.com/cafkafk/gg".to_string(),
                        flags: vec![Clone, Push],
                    },
                );
        }
        // let yaml = serde_yaml::to_string(&config).unwrap();
        // println!("{}", yaml);
    }
    #[test]
    fn read_config_populate() {
        let _config = Config::new(&RelativePath::new("./src/test/config.yaml").to_string());
    }
    #[test]
    fn write_config() {
        let root = current_dir().unwrap();
        let config = Config::new(
            &RelativePath::new("./src/test/config.yaml")
                .to_logical_path(&root)
                .into_os_string()
                .into_string()
                .unwrap(),
        );

        let mut test_file = File::create(
            RelativePath::new("./src/test/test.yaml")
                .to_logical_path(&root)
                .into_os_string()
                .into_string()
                .unwrap(),
        )
        .expect("failed to create test file");
        let contents = serde_yaml::to_string(&config).unwrap();
        test_file.write_all(contents.as_bytes()).unwrap();

        let test_config = Config::new(&RelativePath::new("./src/test/test.yaml").to_string());
        assert_eq!(config, test_config);
    }
    fn get_category<'cat>(config: &'cat Config, name: &'cat str) -> &'cat Category {
        config.categories.get(name).expect("failed to get category")
    }
    fn get_repo<F>(config: &Config, cat_name: &str, repo_name: &str, f: F)
    where
        F: FnOnce(&GitRepo),
    {
        f(config
            .categories
            .get(cat_name)
            .expect("failed to get category")
            .repos
            .get(repo_name)
            .expect("failed to get category"))
    }
    #[test]
    fn is_config_readable() {
        let root = current_dir().unwrap();
        let config = Config::new(
            &RelativePath::new("./src/test/config.yaml")
                .to_logical_path(root)
                .into_os_string()
                .into_string()
                .unwrap(),
        );

        let flags = vec![Clone, Push];
        // FIXME not very extensive
        #[allow(clippy::bool_assert_comparison)]
        {
            get_repo(&config, "config", "qmk_firmware", |repo| {
                assert_eq!(repo.name, "qmk_firmware");
                assert_eq!(repo.path, "/home/ces/org/src/git/");
                assert_eq!(repo.url, "git@github.com:cafkafk/qmk_firmware.git");
            })
        }
        {
            assert_eq!(config.links[0].name, "gg");
            assert_eq!(config.links[0].rx, "/home/ces/.config/gg");
            assert_eq!(config.links[0].tx, "/home/ces/.dots/gg");
            assert_eq!(config.links[1].name, "starship");
            assert_eq!(config.links[1].rx, "/home/ces/.config/starship.toml");
            assert_eq!(config.links[1].tx, "/home/ces/.dots/starship.toml");
            // FIXME doesn't check repoflags
        }
    }
}

/* FIXME Unable to test with networking inside flake
#[cfg(test)]
mod repo_actions {
    use crate::*;
    use git::GitRepo;
    use relative_path::RelativePath;
    use std::env::current_dir;
    use std::process::Command;
    #[test]
    #[allow(clippy::redundant_clone)]
    fn test_repo_actions() {
        let test_repo_name: String = "test".to_string();
        let root = current_dir().unwrap();
        let test_repo_dir: String = RelativePath::new("./src/test")
            .to_logical_path(&root)
            .into_os_string()
            .into_string()
            .unwrap();
        let test_repo_url: String = "git@github.com:cafkafk/test.git".to_string();
        println!("{}", test_repo_dir);
        let mut config = Config {
            repos: vec![],
            links: vec![],
        };
        let repo = GitRepo {
            name: test_repo_name.to_owned(),
            path: test_repo_dir.to_owned(),
            url: test_repo_url.to_owned(),
            clone: true,
        };
        config.repos.push(repo);
        // BUG FIXME can't do this in flake
        // should have a good alternative
        // config.clone_all();
        // config.pull_all();
        for r in config.repos.iter() {
            Command::new("touch")
                .current_dir(&(r.path.to_owned() + &r.name))
                .arg("test")
                .status()
                .expect("failed to create test file");
        }
        config.add_all();
        config.commit_all_msg(&"test".to_string());
    }
}
*/
