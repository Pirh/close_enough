extern crate close_enough;
extern crate clap;

use std::borrow::Cow;
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use clap::{App, Arg};


fn ce_app<'a, 'b>() -> App<'a, 'b>
{
    App::new("ce")
        .author("Pirh, ***redacted.email@redacted.nope***")
        .version("0.1.0")
        .about("Fuzzy-search the input and return the closest match")
        .arg(
            Arg::with_name("query")
            .help("The string or strings to search for;\nIf multiple strings are given, the closest match of each is returned")
            .multiple(true)
            .required(true)
        )
        .arg(
            Arg::with_name("inputs")
            .long("--inputs")
            .short("-i")
            .help("Lines of input to search")
            .takes_value(true)
            .multiple(true)
        )
        .arg(
            Arg::with_name("sep")
            .long("--sep")
            .help("The seperator to join the results with;\nDefaults to newline")
            .takes_value(true)
            .default_value("\n")
        )
        .arg(
            Arg::with_name("cwd")
            .long("--cwd")
            .help("Use current working directory contents as inputs")
            .conflicts_with("inputs")
        )
        .arg(
            Arg::with_name("files_only")
            .short("-f")
            .help("Used with --cwd: only allow files in results")
            .requires("cwd")
            .conflicts_with("dirs_only")
        )
        .arg(
            Arg::with_name("dirs_only")
            .short("-d")
            .help("Used with --cwd: only allow directories in results")
            .requires("cwd")
            .conflicts_with("files_only")
        )
        .after_help(
r#"Fuzzy-search a list of inputs with one or more query strings.
The closest match to each query string is returned on its own line.
If no inputs are provided, inputs are read from stdin."#)
}

#[derive(Copy, Clone)]
enum CwdSearchStrategy
{
    FilesOnly,
    DirectoriesOnly,
    Anything
}

impl CwdSearchStrategy 
{
    fn create(cwd: bool, files_only: bool, dirs_only: bool) -> Option<CwdSearchStrategy>
    {
        match (cwd, files_only, dirs_only)
        {
            (true, true, _) => Some(CwdSearchStrategy::FilesOnly),
            (true, _, true) => Some(CwdSearchStrategy::DirectoriesOnly),
            (true, _, _) => Some(CwdSearchStrategy::Anything),
            _ => None
        }
    }
}


fn main()
{
    let args = ce_app().get_matches();

    let queries = args.values_of("query").expect("Expected query argument");
    let separator = args.value_of("sep").expect("ce: error: could not find separator");
    let cwd_search_strategy = CwdSearchStrategy::create(args.is_present("cwd"), args.is_present("files_only"), args.is_present("dirs_only"));

    let input_lines = fetch_input_lines(args.values_of("inputs"), cwd_search_strategy);

    input_lines.get(0).expect("ce: error: no valid inputs");

    let inputs: Vec<&str> = input_lines.iter().map(|s| s.as_ref()).collect();

    let output: Vec<&str> = queries.map(
        |q| close_enough::closest_enough(&inputs, q).expect("ce: error: query failed to match any inputs")
    ).collect();

    let output = &output.join(separator);

    io::stdout().write(&output.as_bytes()).expect("ce: error: failed to write results");
}


fn fetch_input_lines<'a, I>(input_args: Option<I>, cwd_search_strategy: Option<CwdSearchStrategy>) -> Vec<Cow<'a, str>>
    where I: Iterator<Item=&'a str>
{
    match (input_args, cwd_search_strategy)
    {
        (Some(inputs), _) => inputs.map(|s| Cow::Borrowed(s)).collect(),
        (None, None) => read_stdin(),
        (None, Some(strategy)) => list_cwd(strategy)
    }
}

fn list_cwd<'a>(strategy: CwdSearchStrategy) -> Vec<Cow<'a, str>>
{
    let here = env::current_dir().expect("ce: error: failed to identify current directory");
    let contents = fs::read_dir(&here).expect("ce: error: failed to read current directory");

    contents.filter_map(move |entry|
        {
            let entry = entry.expect("ce: error: failed to read directory entry");
            let entry = match strategy
            {
                CwdSearchStrategy::FilesOnly => if entry.file_type().expect("ce: error: failed to read file type").is_file() { Some(entry) } else { None },
                CwdSearchStrategy::DirectoriesOnly => if entry.file_type().expect("ce: error: failed to read file type").is_dir() { Some(entry) } else { None },
                _ => Some(entry)
            };
            entry.map(|entry| Cow::Owned(entry.file_name().into_string().expect("ce: error: failed to read directory entry")))
        }
    ).collect()
}

fn read_stdin<'a>() -> Vec<Cow<'a, str>>
{
    let mut s = String::new();
    io::stdin().read_to_string(&mut s).expect("ce: error: failed to read from stdin");

    s.lines().map(|s| Cow::Owned(s.to_owned())).collect()
}