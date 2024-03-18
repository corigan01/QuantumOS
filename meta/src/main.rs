use clap::Parser;

mod cmdline;

fn main() {
    let args = cmdline::CommandLine::parse();

    match args.option.unwrap_or(cmdline::TaskOption::Run) {
        cmdline::TaskOption::Build => {
            todo!("build")
        }
        cmdline::TaskOption::Run => {
            todo!("run")
        }
        cmdline::TaskOption::Clean => {
            todo!("clean")
        }
    }
}
