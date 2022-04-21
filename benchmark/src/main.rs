use ancestry_common::{LargeSignedInteger, SignedInteger};
use ancestry_inline_simplification::Population;
use clap::Parser;
use neutral_evolution::{evolve, Parameters};
use tskit::TableAccess;
use tskit_evolution::EvolvableTableCollection;

#[derive(clap::Parser, Copy, Clone)]
struct Args {
    #[clap(subcommand)]
    simulator: Simulator,
    #[clap(long, short)]
    popsize: SignedInteger,
    #[clap(long, short)]
    rho: f64,
    #[clap(long, short)]
    sequence_length: LargeSignedInteger,
    #[clap(long, short)]
    nsteps: LargeSignedInteger,
    #[clap(long, short)]
    death_probability: f64,
    #[clap(long)]
    seed: u64,
}

#[derive(clap::Subcommand, Clone, Copy)]
enum Simulator {
    Tskit(Tskit),
    Dynamic,
}

#[derive(clap::Parser, Clone, Copy)]
struct Tskit {
    #[clap(long, short)]
    simplification_interval: LargeSignedInteger,
}

fn evolve_wrapper<T: neutral_evolution::EvolveAncestry>(
    parameters: Parameters,
    args: Args,
    population: &mut T,
) {
    evolve(args.seed, parameters, population).unwrap();
}

fn main() {
    let args = Args::parse();

    let c = args.rho / 4.0 / (args.popsize as f64);

    let parameters = Parameters::new(args.death_probability, c, args.nsteps).unwrap();

    match args.simulator {
        Simulator::Tskit(t) => {
            let mut population = EvolvableTableCollection::new(
                args.sequence_length,
                args.popsize,
                t.simplification_interval,
            )
            .unwrap();
            evolve_wrapper(parameters, args, &mut population);
            let tables = tskit::TableCollection::from(population);
            println!(
                "nodes: {}, edges: {}",
                tables.nodes().num_rows(),
                tables.edges().num_rows()
            );
        }
        Simulator::Dynamic => {
            let mut population = Population::new(args.popsize, args.sequence_length).unwrap();
            evolve_wrapper(parameters, args, &mut population);
            println!("num still reachable = {}", population.num_still_reachable());
        }
    }
}
