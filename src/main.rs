mod hash;
use clap::Parser;
use std::fs::metadata;


#[derive(clap::Parser)]
struct Args {
    /// Path do be hashed, file or directory
    #[arg(short = 'p', long, default_value = ".")]
    path: String,

    /// Output file
    #[arg(short = 'o', long)]
    output: String,
}

fn main() {
    let args = Args::parse();
    let meta = metadata(&args.path).unwrap();
    if meta.is_dir() {
        hash::hash(&args.path, &args.output);
        
    } else {
        print!("Hash: \n{}", hash::hash_file(&args.path))
    }
}
