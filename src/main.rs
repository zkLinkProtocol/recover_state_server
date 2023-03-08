use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "Recover state driver", author = "N Labs", rename_all = "snake_case")]
struct Opt {
    /// Restores data with provided genesis (zero) block
    #[structopt(long)]
    genesis: bool,

    /// Continues data restoring
    #[structopt(long = "continue", name = "continue")]
    continue_mode: bool,

    /// Restore data until the last verified block and exit
    #[structopt(long)]
    finite: bool,

    /// Expected tree root hash after restoring. This argument is ignored if mode is not `finite`
    #[structopt(long)]
    final_hash: Option<String>,
}

fn main() {
    println!("Hello, world!");
}
