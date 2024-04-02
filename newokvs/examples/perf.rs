use clap::Parser;
use newokvs::{
    newokvs::OKVS, okvs::{OkvsDecoder, OkvsEncoder},
    block::Block, hash::BufferedRandomGenerator,
    utils::TimerOnce,
    utils::print_communication
};
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
struct Arguments {

    #[arg(short, long, default_value_t = 1048576)]
    n: usize,
    #[arg(short, long, default_value_t = 0.0)]
    epsilon: f64,
    #[arg(short, long, default_value_t = 448)]
    width: usize,
}

fn test_encoder<E>(args: Arguments, encoder: E) where
    E: OkvsEncoder<Block, Block> + OkvsDecoder<Block, Block>
{
    let mut map = Vec::new();
    let mut rng = BufferedRandomGenerator::from_entropy();
    for _ in 0..args.n {
        let key = rng.gen_block();
        let value = rng.gen_block();
        map.push((key, value));
    }

    let s = encoder.encode(&map);

    let keys = map.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>();
    let values = encoder.decode_many(&s, &keys);
    let num_cores = num_cpus::get()/4;
    let start = Instant::now();
    let decoded = encoder.decode_many(&s, &keys);
    let duration = start.elapsed();
    println!("Decode time: {:?}", duration/num_cores as u32);
    //assert_eq!(decoded, values, "decoded = {:?}, values = {:?}", decoded, values);
    print_communication("Encoded length", 0, s.len() * std::mem::size_of::<Block>(), 1);
}

fn test_okvs(mut args: Arguments) {

    println!("[OKVS arguments]");
    if args.epsilon == 0.0 {
        args.epsilon = 0.01;
        println!("  eps   = {} (default)", args.epsilon);
    } else {
        println!("  eps   = {}", args.epsilon);
    }
    println!("  width = {}", args.width);
    
    let encoder = OKVS::new(args.epsilon, args.width);
    test_encoder(args, encoder);
}


fn main() {
    let args = Arguments::parse();
    let mut map = Vec::new();
    let mut rng = BufferedRandomGenerator::from_entropy();
    for _ in 0..args.n {
        let key = rng.gen_block();
        let value = rng.gen_block();
        map.push((key, value));
    }

    println!("[Arguments]");
    println!("  Set size (n)   = {}", args.n);
    println!("    log n        = {:.1}", (args.n as f64).log2());

    test_okvs(args);
}


