use bio::io::fasta;
use gdijkstra::bfs;
use std::{fs::File, io::BufReader, path::PathBuf};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Args {
    graph: PathBuf,
    query: PathBuf,
}

fn main() {
    let args = Args::from_args();
    let parser = gfa::parser::GFAParser::<Vec<u8>, ()>::new();

    let gfa = parser.parse_file(args.graph).unwrap();

    let record = fasta::Reader::new(BufReader::new(File::open(&args.query).unwrap()))
        .records()
        .next()
        .unwrap()
        .unwrap();
    let query = record.seq();

    // Extension alignment of the entire query starting at vertex 0.
    let dist = bfs(&gfa, query, 0);
    println!(
        "DIST: {dist}\tLEN: {}\tREL: {}",
        query.len(),
        dist as f32 / query.len() as f32
    );
}
