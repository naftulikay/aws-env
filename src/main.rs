use aws_env::{AwsCredentials, AwsCredentialsFile, AwsProfile, AwsProfileLookup};

use log::LevelFilter;

use log4rs::append::console::{ConsoleAppender, Target};

use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Config;
use prettytable::format::{LinePosition, LineSeparator};

use prettytable::{cell, format, row, Table};

use serde::Serialize;

use std::io;
use std::io::Write;
use std::io::{BufWriter, LineWriter};
use std::process::exit;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};

use structopt::StructOpt;

use tokio;

const LIST_FORMATS: &'static [&'static str] = &["table", "plain", "csv", "json"];
const LOG_LEVELS: &'static [&'static str] = &["trace", "debug", "info", "warn", "error"];

#[derive(Debug, StructOpt)]
struct CliArgs {
    /// Set the logging level for the utility.
    #[structopt(long="log-level", default_value="error", possible_values=LOG_LEVELS)]
    log_level: String,
    #[structopt(subcommand)]
    cmd: CliCommand,
}

#[derive(Debug, StructOpt)]
enum CliCommand {
    /// Export the specified profile.
    Export(ExportCommand),
    /// List available profiles.
    List(ListCommand),
}

#[derive(Debug, StructOpt)]
struct ListCommand {
    /// Exclude the header when printing to a TTY.
    #[structopt(long = "no-header")]
    no_header: bool,
    /// The output format.
    #[structopt(short = "F", long = "format", default_value="table", possible_values=LIST_FORMATS)]
    format: ListFormat,
}

#[derive(Debug, StructOpt)]
struct ExportCommand {
    /// The profile name to export. This can be either the bare profile name or a URI. See the 'list' command for URI format.
    #[structopt(name = "profile_name")]
    name: String,
}

/// Output format for listing profiles.
#[derive(Debug)]
enum ListFormat {
    /// Default table output format.
    Table,
    /// Plaintext format, useful for parsing with command-line tools.
    Plain,
    /// CSV format.
    Csv,
    /// JSON format.
    Json,
}

impl Default for ListFormat {
    fn default() -> Self {
        ListFormat::Table
    }
}

impl ToString for ListFormat {
    fn to_string(&self) -> String {
        match *self {
            ListFormat::Table => "table".into(),
            ListFormat::Plain => "plain".into(),
            ListFormat::Csv => "csv".into(),
            ListFormat::Json => "json".into(),
        }
    }
}

impl FromStr for ListFormat {
    type Err = Box<dyn std::error::Error>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "table" => Ok(ListFormat::Table),
            "plain" => Ok(ListFormat::Plain),
            "csv" => Ok(ListFormat::Csv),
            "json" => Ok(ListFormat::Json),
            _ => Err(format!("unknown format {}", s).into()),
        }
    }
}

fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .thread_name_fn(|| {
            static THREAD_ID: AtomicUsize = AtomicUsize::new(0);
            let id = THREAD_ID.fetch_add(1, Ordering::SeqCst);
            format!("tokio-{:02}", id)
        })
        .enable_io()
        .enable_time()
        .build()
        .unwrap()
        .block_on(async_main())
        .unwrap();
}

async fn async_main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::from_args();

    configure_logging(match args.log_level.trim().to_lowercase().as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Error,
    });

    match args.cmd {
        CliCommand::List(c) => list_profiles(c).await?,
        CliCommand::Export(c) => export_profile(c).await?,
    }

    Ok(())
}

async fn list_profiles(args: ListCommand) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Listing all available profiles.");

    let credentials = AwsCredentials::load_all().await?;

    let mut lookup = AwsProfileLookup::new();
    lookup.insert_all(credentials.sources.into_iter());

    match &args.format {
        ListFormat::Table => list_profiles_table(&args, &lookup),
        ListFormat::Plain => list_profiles_plain(&args, &lookup),
        ListFormat::Json => list_profiles_json(&args, &lookup),
        ListFormat::Csv => list_profiles_csv(&args, &lookup),
    };

    Ok(())
}

fn list_profiles_table(args: &ListCommand, lookup: &AwsProfileLookup) {
    let mut table = Table::new();
    let mut format = format::FormatBuilder::new()
        .padding(0, 0)
        .column_separator(' ');

    if !args.no_header {
        format = format.separator(LinePosition::Title, LineSeparator::new('―', ' ', ' ', ' '))
    }

    table.set_format(format.build());

    if !args.no_header {
        // FIXME can space cells via Cell::with_hspan(x)
        table.set_titles(row!["profile", "prefix/profile", "priority", "file"]);
    }

    for (index, profile) in lookup.list().into_iter().enumerate() {
        // FIXME can space cells via Cell::with_hspan(x)

        table.add_row(row![
            Fg->profile.name,
            Fcb->profile.uri(),
            format!("{:02}", index),
            FD->format!("{}", profile.file.home_path().display())
        ]);
    }

    table.printstd();
}

fn list_profiles_plain(_args: &ListCommand, lookup: &AwsProfileLookup) {
    let mut writer = LineWriter::new(io::stdout());

    for (priority, profile) in lookup.list().into_iter().enumerate() {
        write!(
            writer,
            "{name} {uri} {priority} {file}",
            name = profile.name,
            uri = profile.uri(),
            priority = priority,
            file = profile.file.path.display()
        )
        .expect("unable to write to stdout");
    }
}

fn list_profiles_csv(args: &ListCommand, lookup: &AwsProfileLookup) {
    let mut writer = csv::Writer::from_writer(LineWriter::new(io::stdout()));

    if !args.no_header {
        writer
            .write_record(&["name", "uri", "priority", "file"])
            .expect("unable to write header to stdout");
    }

    for (priority, profile) in lookup.list().iter().enumerate() {
        writer
            .write_record(&[
                profile.name.as_str(),
                profile.uri().as_str(),
                format!("{}", priority).as_str(),
                profile.file.path.display().to_string().as_str(),
            ])
            .expect("unable to write row to stdout");
    }
}

fn list_profiles_json(_args: &ListCommand, lookup: &AwsProfileLookup) {
    #[derive(Debug, Serialize)]
    struct Record<'a> {
        name: &'a str,
        uri: String,
        priority: usize,
        file: String,
    }

    let values = lookup.list();
    let mut output = Vec::with_capacity(values.len());

    for (priority, profile) in values.iter().enumerate() {
        output.push(Record {
            name: profile.name.as_str(),
            uri: profile.uri(),
            priority,
            file: profile.file.path.display().to_string(),
        });
    }

    serde_json::to_writer_pretty(BufWriter::new(io::stdout()), output.as_slice())
        .expect("unable to serialize to standard output");
    println!();
}

fn configure_logging(level: LevelFilter) {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "[{d} {h({l:>5})} {t} (({T}))] {m}{n}",
        )))
        .target(Target::Stderr)
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(level))
        .unwrap();

    let _handle = log4rs::init_config(config).unwrap();
}

async fn export_profile(args: ExportCommand) -> Result<(), Box<dyn std::error::Error>> {
    let mut lookup = AwsProfileLookup::new();

    if args.name.as_str().starts_with("/") {
        // if the name starts with a slash, assume it's in ~/.aws/credentials
        log::debug!("Attempting to load profile from ~/.aws/credentials...");

        if let Ok(f) = AwsCredentialsFile::load(
            dirs::home_dir()
                .expect("unable to get home directory")
                .join(".aws")
                .join("credentials"),
        )
        .await
        {
            lookup.insert(f);
        }
    } else if args.name.as_str().contains("/") {
        // if the name contains a slash, attempt to load the specific file-stem(s)
        // FIXME implement file-stem insertion
    }

    // try to look up
    if let Some(p) = lookup.by_uri(args.name.as_str()) {
        log::debug!(
            "Lazy loading successful, found profile {} in {}",
            p.name.as_str(),
            p.file.home_path().display()
        );
        output_profile(p).expect("unable to write profile to stdout");
        return Ok(());
    }

    // if we've made it this far, lazy-loading has failed so load everything
    log::debug!("Unable to find profile so far, loading all credential files.");

    match AwsCredentials::load_all().await {
        Ok(c) => lookup.insert_all(c.sources.into_iter()),
        Err(e) => {
            log::error!("Unable to load credentials: {}", e);
            exit(1);
        }
    };

    if let Some(p) = lookup
        .by_uri(args.name.as_str())
        .or_else(|| lookup.by_name(args.name.as_str()))
    {
        log::debug!(
            "Located profile {} in {}",
            args.name.as_str(),
            p.file.home_path().display()
        );
        output_profile(p).unwrap();
        Ok(())
    } else {
        log::error!("Unable to find profile '{}'", args.name.as_str());
        exit(1);
    }
}

fn output_profile(profile: &AwsProfile) -> io::Result<()> {
    // NOTE we can't output buffer without placing credentials into memory we can't manage, so don't buffer
    let mut writer = io::stdout();

    // access key id
    write!(
        writer,
        "export AWS_ACCESS_KEY_ID={}\n",
        profile.access_key_id
    )?;

    // secret access key
    write!(
        writer,
        "export AWS_SECRET_ACCESS_KEY={}\n",
        profile.secret_access_key.as_str()
    )?;

    if let Some(session_token) = &profile.session_token {
        // optional session token
        write!(
            writer,
            "export AWS_SESSION_TOKEN={}\n",
            session_token.as_str()
        )?;
    }

    Ok(())
}
