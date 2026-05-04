use std::path::PathBuf;

use clap::Parser;

mod cache;
mod env;
mod executor;
mod jsonpath;
mod output;
mod parser;

#[derive(Parser)]
#[command(
    name = "rest-runner",
    about = "Execute HTTP requests from .rest/.http files",
    version
)]
struct Args {
    /// Path to the .rest or .http file
    file: PathBuf,

    /// Line number of the request to execute (1-based). Defaults to first request.
    #[arg(long, short)]
    line: Option<usize>,

    /// Execute the request with this name (from ### Name or # @name)
    #[arg(long, short = 'n')]
    name: Option<String>,

    /// Environment name to use from the env file (e.g. "local", "prod")
    #[arg(long, short = 'e')]
    env: Option<String>,

    /// Path to env file (default: .rest-client.env.json next to the .rest file)
    #[arg(long)]
    env_file: Option<PathBuf>,

    /// Request timeout in seconds (default: 30)
    #[arg(long, short, default_value = "30")]
    timeout: u64,

    /// Save the response body to this file
    #[arg(long, short)]
    output: Option<PathBuf>,

    /// Include status line and headers in the output file (requires --output)
    #[arg(long, requires = "output")]
    output_headers: bool,

    /// Print request headers before sending
    #[arg(long, short)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    let content = std::fs::read_to_string(&args.file).unwrap_or_else(|e| {
        eprintln!("error: cannot read {}: {}", args.file.display(), e);
        std::process::exit(1);
    });

    let requests = parser::parse(&content);

    if requests.is_empty() {
        eprintln!("error: no HTTP requests found in {}", args.file.display());
        std::process::exit(1);
    }

    let request = select_request(&requests, args.line, args.name.as_deref());
    let request = match request {
        Some(r) => r.clone(),
        None => {
            eprintln!("error: no request found at the given position");
            std::process::exit(1);
        }
    };

    let env_file = args.env_file.unwrap_or_else(|| {
        args.file
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join(".rest-client.env.json")
    });

    let vars = env::load(&env_file, args.env.as_deref());
    let request = parser::substitute_vars(request, &vars);

    output::print_request_header(&request);

    match executor::execute(&request, args.verbose, args.timeout) {
        Ok(response) => {
            // Cache the response by name so subsequent requests can chain on it.
            if let Some(ref name) = request.name {
                cache::save(
                    name,
                    &cache::CachedResponse {
                        status: response.status,
                        status_text: response.status_text.clone(),
                        headers: response
                            .headers
                            .iter()
                            .map(|(k, v)| (k.to_lowercase(), v.clone()))
                            .collect(),
                        body_raw: response.body.clone(),
                    },
                );
            }
            if let Some(path) = args.output {
                output::save_to_file(&response, &path, args.output_headers);
            }
            output::print_response(&response);
        }
        Err(e) => {
            eprintln!("error: request failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn select_request<'a>(
    requests: &'a [parser::Request],
    line: Option<usize>,
    name: Option<&str>,
) -> Option<&'a parser::Request> {
    if let Some(line) = line {
        return parser::find_at_line(requests, line);
    }
    if let Some(name) = name {
        return requests.iter().find(|r| r.name.as_deref() == Some(name));
    }
    requests.first()
}
