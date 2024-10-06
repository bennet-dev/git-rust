use anyhow::Context;
use clap::{Parser, Subcommand};
use flate2::read::DeflateDecoder;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
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
    HashObject {
        #[clap(short = 'w')]
        write: bool,
        file: String,
    },
}

// enum Kind {
//     Blob,
// }

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    fn zlib_reader(bytes: Vec<u8>) -> Result<std::string::String, std::io::Error> {
        let mut z = ZlibDecoder::new(&bytes[..]);
        let mut s = String::new();
        z.read_to_string(&mut s)?;
        Ok(s)
    }

    fn deflate_reader(bytes: Vec<u8>) -> io::Result<String> {
        let mut deflater = DeflateDecoder::new(&bytes[..]);
        let mut s = String::new();
        deflater.read_to_string(&mut s)?;
        Ok(s)
    }

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

            let decompressed = zlib_reader(contents).unwrap();
            if let Some((_, content)) = decompressed.split_once("\0") {
                let stdout = io::stdout();
                let mut handle = stdout.lock();

                handle.write_all(content.as_bytes()).unwrap()
            } else {
                println!("Didn't find null byte...")
            }
        }
        Command::HashObject { write, file } => {
            // create a Sha1 object
            let mut hasher = Sha1::new();

            let raw_bytes = fs::read(file)?;
            let size = &raw_bytes.len();
            let contents = String::from_utf8(raw_bytes.clone()).unwrap();

            let blob = format!("blob {}\0{}", size, &contents);

            // // process input message
            hasher.update(&blob);
            let hash_object = hex::encode(hasher.finalize());

            println!("{}", &hash_object);

            if write {
                // //TODO: zlib compress the file
                let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
                e.write_all(&blob.as_bytes())?;
                let compressed = e.finish()?;

                // // Path for the directory based on the first two characters of the hash
                let dir_path = format!(".git/objects/{}", &hash_object[..2]);

                // // Path for the actual file (rest of the hash is used as the file name)
                let file_path = format!("{}/{}", &dir_path, &hash_object[2..]);

                // // Create the directory if it doesn't exist
                fs::create_dir_all(&dir_path)?;

                // //TODO: The input for the SHA hash is the header (blob <size>\0) + the actual contents of the file,
                // //      not just the contents of the file.

                // // Write the compressed bytes to the file
                fs::write(&file_path, compressed)?;
            } else {
                // println!("HASH: {:?}", hash_object)
            }
        }
    };
    Ok(())
}
