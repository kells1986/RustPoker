use rust_poker::table::{TableConfig, TableMode, build_strategies, play_single_hand};
use std::env;

#[derive(Debug, Clone, Copy)]
struct CliConfig {
    table: TableConfig,
    rounds: u32,
}

fn main() {
    if let Err(message) = run() {
        eprintln!("{message}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| String::from("simulate"));
    let command_args: Vec<String> = args.collect();

    match command.as_str() {
        "simulate" => run_mode(TableMode::AgentsOnly, &command_args),
        "play" => run_mode(TableMode::HumanVsAgents, &command_args),
        "-h" | "--help" | "help" => {
            println!("{}", usage());
            Ok(())
        }
        other => Err(format!("Unknown command `{other}`.\n\n{}", usage())),
    }
}

fn run_mode(mode: TableMode, args: &[String]) -> Result<(), String> {
    let cli = parse_cli(args)?;
    let mut strategies = build_strategies(mode, cli.table.seats);
    let mut aggregate_net = vec![0_i64; cli.table.seats];

    for hand_number in 0..cli.rounds {
        let hand = play_single_hand(cli.table, &mut strategies)
            .map_err(|error| format!("hand {} failed: {error:?}", hand_number + 1))?;

        println!("\nHand {}", hand_number + 1);
        println!("Community: {:?}", hand.community_cards);
        for seat in 0..cli.table.seats {
            let net = hand.payouts[seat] as i64 - hand.committed[seat] as i64;
            aggregate_net[seat] += net;
            println!(
                "Seat {seat} ({}) | hole: {:?} {:?} | folded: {} | committed: {} | payout: {} | stack_after: {}",
                strategies[seat].name(),
                hand.hole_cards[seat][0],
                hand.hole_cards[seat][1],
                hand.folded[seat],
                hand.committed[seat],
                hand.payouts[seat],
                hand.stacks_after[seat]
            );
        }
    }

    println!("\nAggregate Net (payout - commitment)");
    for seat in 0..cli.table.seats {
        println!(
            "Seat {seat} ({}) => {}",
            strategies[seat].name(),
            aggregate_net[seat]
        );
    }

    Ok(())
}

fn parse_cli(args: &[String]) -> Result<CliConfig, String> {
    let mut table = TableConfig::default();
    let mut rounds = 1_u32;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--seats" => {
                i += 1;
                table.seats = parse_usize(args.get(i), "--seats")?;
            }
            "--table-stakes" => {
                i += 1;
                table.table_stakes = parse_u32(args.get(i), "--table-stakes")?;
            }
            "--small-blind" => {
                i += 1;
                table.small_blind = parse_u32(args.get(i), "--small-blind")?;
            }
            "--big-blind" => {
                i += 1;
                table.big_blind = parse_u32(args.get(i), "--big-blind")?;
            }
            "--initial-stack" => {
                i += 1;
                table.initial_stack = parse_u32(args.get(i), "--initial-stack")?;
            }
            "--rounds" => {
                i += 1;
                rounds = parse_u32(args.get(i), "--rounds")?;
            }
            "-h" | "--help" => return Err(usage().to_string()),
            unknown => return Err(format!("Unknown argument `{unknown}`.\n\n{}", usage())),
        }
        i += 1;
    }

    if rounds == 0 {
        return Err(String::from("--rounds must be at least 1"));
    }

    Ok(CliConfig { table, rounds })
}

fn parse_u32(value: Option<&String>, flag_name: &str) -> Result<u32, String> {
    let raw = value.ok_or_else(|| format!("Missing value for {flag_name}"))?;
    raw.parse::<u32>()
        .map_err(|_| format!("Invalid integer for {flag_name}: `{raw}`"))
}

fn parse_usize(value: Option<&String>, flag_name: &str) -> Result<usize, String> {
    let raw = value.ok_or_else(|| format!("Missing value for {flag_name}"))?;
    raw.parse::<usize>()
        .map_err(|_| format!("Invalid integer for {flag_name}: `{raw}`"))
}

fn usage() -> &'static str {
    "Usage:
  cargo run -- simulate [options]
  cargo run -- play [options]

Options:
  --seats <n>           Number of table seats (default: 6)
  --table-stakes <n>    Maximum buy-in for table (default: 1000)
  --small-blind <n>     Small blind amount (default: 5)
  --big-blind <n>       Big blind amount (default: 10)
  --initial-stack <n>   Starting stack per player (default: 500)
  --rounds <n>          Number of hands to run (default: 1)

Modes:
  simulate  All seats use simple agent strategy.
  play      Seat 0 is human (console input); other seats are simple agents."
}
