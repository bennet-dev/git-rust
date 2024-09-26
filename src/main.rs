use anyhow::Context;
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io;
use std::io::prelude::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init,
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,
        object_hash: String,
    },
}

enum Kind {
    Blob,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialized git directory")
        }
        Command::CatFile {
            pretty_print,
            object_hash,
        } => {
            let contents = fs::read(format!(
                "./.git/objects/{}/{}",
                &object_hash[..2],
                &object_hash[2..]
            ))
            .context("Should have been able to read the file")?;

            fn decode_reader(bytes: Vec<u8>) -> Result<std::string::String, std::io::Error> {
                let mut z = ZlibDecoder::new(&bytes[..]);
                let mut s = String::new();
                z.read_to_string(&mut s)?;
                Ok(s)
            }
            let decompressed = decode_reader(contents).unwrap();
            if let Some((_, content)) = decompressed.split_once("\0") {
                let stdout = io::stdout();
                let mut handle = stdout.lock();

                handle.write_all(content.as_bytes()).unwrap()
            } else {
                println!("Didn't find null byte...")
            }
        }
    };
    Ok(())
}
