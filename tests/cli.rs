use clap::Parser;
use uman::cli::{BackendAction, Commands};

#[test]
fn parse_read_three_args() {
    let cli = uman::cli::Cli::try_parse_from(["uman", "linux-upstream", "2", "execve"]).unwrap();
    assert!(cli.command.is_none());
    assert_eq!(cli.backend.as_deref(), Some("linux-upstream"));
    assert_eq!(cli.section.as_deref(), Some("2"));
    assert_eq!(cli.topic.as_deref(), Some("execve"));
}

#[test]
fn parse_install() {
    let cli = uman::cli::Cli::try_parse_from(["uman", "install", "linux-upstream"]).unwrap();
    match cli.command {
        Some(Commands::Install { backend }) => assert_eq!(backend, "linux-upstream"),
        other => panic!("expected Install, got {:?}", other),
    }
}

#[test]
fn parse_remove() {
    let cli = uman::cli::Cli::try_parse_from(["uman", "remove", "freebsd"]).unwrap();
    match cli.command {
        Some(Commands::Remove { backend }) => assert_eq!(backend, "freebsd"),
        other => panic!("expected Remove, got {:?}", other),
    }
}

#[test]
fn parse_update_all() {
    let cli = uman::cli::Cli::try_parse_from(["uman", "update"]).unwrap();
    match cli.command {
        Some(Commands::Update { backend }) => assert!(backend.is_none()),
        other => panic!("expected Update, got {:?}", other),
    }
}

#[test]
fn parse_update_single() {
    let cli = uman::cli::Cli::try_parse_from(["uman", "update", "linux-upstream"]).unwrap();
    match cli.command {
        Some(Commands::Update { backend }) => assert_eq!(backend.as_deref(), Some("linux-upstream")),
        other => panic!("expected Update, got {:?}", other),
    }
}

#[test]
fn parse_search() {
    let cli = uman::cli::Cli::try_parse_from(["uman", "search", "execve"]).unwrap();
    match cli.command {
        Some(Commands::Search { keyword, topic }) => {
            assert!(!keyword);
            assert_eq!(topic, "execve");
        }
        other => panic!("expected Search, got {:?}", other),
    }
}

#[test]
fn parse_search_with_keyword_flag() {
    let cli = uman::cli::Cli::try_parse_from(["uman", "search", "-k", "execute"]).unwrap();
    match cli.command {
        Some(Commands::Search { keyword, topic }) => {
            assert!(keyword);
            assert_eq!(topic, "execute");
        }
        other => panic!("expected Search, got {:?}", other),
    }
}

#[test]
fn parse_backend_list() {
    let cli = uman::cli::Cli::try_parse_from(["uman", "backend", "list"]).unwrap();
    match cli.command {
        Some(Commands::Backend { action }) => match action {
            BackendAction::List => {}
        },
        other => panic!("expected Backend, got {:?}", other),
    }
}

#[test]
fn parse_no_args() {
    let cli = uman::cli::Cli::try_parse_from(["uman"]).unwrap();
    assert!(cli.command.is_none());
    assert!(cli.backend.is_none());
    assert!(cli.section.is_none());
    assert!(cli.topic.is_none());
}

#[test]
fn parse_partial_args() {
    let cli = uman::cli::Cli::try_parse_from(["uman", "linux-upstream"]).unwrap();
    assert!(cli.command.is_none());
    assert_eq!(cli.backend.as_deref(), Some("linux-upstream"));
    assert!(cli.section.is_none());
}

#[test]
fn parse_two_args() {
    let cli = uman::cli::Cli::try_parse_from(["uman", "linux-upstream", "2"]).unwrap();
    assert!(cli.command.is_none());
    assert_eq!(cli.backend.as_deref(), Some("linux-upstream"));
    assert_eq!(cli.section.as_deref(), Some("2"));
    assert!(cli.topic.is_none());
}

#[test]
fn subcommands_take_priority_over_positional() {
    // "install" should be parsed as a subcommand, not as a backend name
    let cli = uman::cli::Cli::try_parse_from(["uman", "install", "mybackend"]).unwrap();
    assert!(cli.backend.is_none()); // "install" should not be captured as backend
    match cli.command {
        Some(Commands::Install { backend }) => assert_eq!(backend, "mybackend"),
        other => panic!("expected Install, got {:?}", other),
    }
}