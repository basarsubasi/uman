use clap::Parser;
use clap_complete::Shell;
use uniman::cli::Commands;

#[test]
fn parse_read_three_args() {
    let cli = uniman::cli::Cli::try_parse_from(["uniman", "linux-upstream", "2", "execve"]).unwrap();
    assert!(cli.command.is_none());
    assert_eq!(cli.backend.as_deref(), Some("linux-upstream"));
    assert_eq!(cli.section.as_deref(), Some("2"));
    assert_eq!(cli.topic.as_deref(), Some("execve"));
}

#[test]
fn parse_install() {
    let cli = uniman::cli::Cli::try_parse_from(["uniman", "install", "linux-upstream"]).unwrap();
    match cli.command {
        Some(Commands::Install { backend }) => assert_eq!(backend, "linux-upstream"),
        other => panic!("expected Install, got {:?}", other),
    }
}

#[test]
fn parse_remove() {
    let cli = uniman::cli::Cli::try_parse_from(["uniman", "remove", "freebsd"]).unwrap();
    match cli.command {
        Some(Commands::Remove { backend }) => assert_eq!(backend, "freebsd"),
        other => panic!("expected Remove, got {:?}", other),
    }
}

#[test]
fn parse_update_all() {
    let cli = uniman::cli::Cli::try_parse_from(["uniman", "update"]).unwrap();
    match cli.command {
        Some(Commands::Update { backend }) => assert!(backend.is_none()),
        other => panic!("expected Update, got {:?}", other),
    }
}

#[test]
fn parse_list() {
    let cli = uniman::cli::Cli::try_parse_from(["uniman", "list"]).unwrap();
    match cli.command {
        Some(Commands::List { backend }) => assert!(backend.is_none()),
        other => panic!("expected List, got {:?}", other),
    }
}

#[test]
fn parse_list_backend() {
    let cli = uniman::cli::Cli::try_parse_from(["uniman", "list", "linux-upstream"]).unwrap();
    match cli.command {
        Some(Commands::List { backend }) => assert_eq!(backend.as_deref(), Some("linux-upstream")),
        other => panic!("expected List with backend, got {:?}", other),
    }
}

#[test]
fn parse_config() {
    let cli = uniman::cli::Cli::try_parse_from(["uniman", "config"]).unwrap();
    match cli.command {
        Some(Commands::Config) => {}
        other => panic!("expected Config, got {:?}", other),
    }
}

#[test]
fn parse_completion_bash() {
    let cli = uniman::cli::Cli::try_parse_from(["uniman", "completion", "bash"]).unwrap();
    match cli.command {
        Some(Commands::Completion { shell }) => assert_eq!(shell, Shell::Bash),
        other => panic!("expected Completion, got {:?}", other),
    }
}

#[test]
fn parse_update_single() {
    let cli = uniman::cli::Cli::try_parse_from(["uniman", "update", "linux-upstream"]).unwrap();
    match cli.command {
        Some(Commands::Update { backend }) => assert_eq!(backend.as_deref(), Some("linux-upstream")),
        other => panic!("expected Update, got {:?}", other),
    }
}

#[test]
fn parse_search() {
    let cli = uniman::cli::Cli::try_parse_from(["uniman", "search", "execve"]).unwrap();
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
    let cli = uniman::cli::Cli::try_parse_from(["uniman", "search", "-k", "execute"]).unwrap();
    match cli.command {
        Some(Commands::Search { keyword, topic }) => {
            assert!(keyword);
            assert_eq!(topic, "execute");
        }
        other => panic!("expected Search, got {:?}", other),
    }
}

#[test]
fn parse_default_show() {
    let cli = uniman::cli::Cli::try_parse_from(["uniman", "default"]).unwrap();
    match cli.command {
        Some(Commands::Default { name }) => assert!(name.is_none()),
        other => panic!("expected Default, got {:?}", other),
    }
}

#[test]
fn parse_default_set() {
    let cli = uniman::cli::Cli::try_parse_from(["uniman", "default", "linux-upstream"]).unwrap();
    match cli.command {
        Some(Commands::Default { name }) => assert_eq!(name.as_deref(), Some("linux-upstream")),
        other => panic!("expected Default, got {:?}", other),
    }
}

#[test]
fn parse_no_args() {
    let cli = uniman::cli::Cli::try_parse_from(["uniman"]).unwrap();
    assert!(cli.command.is_none());
    assert!(cli.backend.is_none());
    assert!(cli.section.is_none());
    assert!(cli.topic.is_none());
}

#[test]
fn parse_partial_args() {
    let cli = uniman::cli::Cli::try_parse_from(["uniman", "linux-upstream"]).unwrap();
    assert!(cli.command.is_none());
    assert_eq!(cli.backend.as_deref(), Some("linux-upstream"));
    assert!(cli.section.is_none());
}

#[test]
fn parse_two_args() {
    let cli = uniman::cli::Cli::try_parse_from(["uniman", "linux-upstream", "2"]).unwrap();
    assert!(cli.command.is_none());
    assert_eq!(cli.backend.as_deref(), Some("linux-upstream"));
    assert_eq!(cli.section.as_deref(), Some("2"));
    assert!(cli.topic.is_none());
}

#[test]
fn subcommands_take_priority_over_positional() {
    // "install" should be parsed as a subcommand, not as a backend name
    let cli = uniman::cli::Cli::try_parse_from(["uniman", "install", "mybackend"]).unwrap();
    assert!(cli.backend.is_none()); // "install" should not be captured as backend
    match cli.command {
        Some(Commands::Install { backend }) => assert_eq!(backend, "mybackend"),
        other => panic!("expected Install, got {:?}", other),
    }
}