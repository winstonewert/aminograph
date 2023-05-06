mod amino_acids;
use std::io::Write;

use jemallocator::Jemalloc;
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[macro_use]
mod fixed;
mod alignment;
mod log;
mod optimization;
mod prelude;
mod trace;
#[allow(dead_code)]
#[macro_use]
mod slab;
mod graph;
mod order_optimize;
use optimization::moves::GraphMove;
use rand::SeedableRng;

use crate::prelude::*;
use std::path::Path;

use structopt::StructOpt;

pub fn cli_init() -> Result<()> {
    color_eyre::config::HookBuilder::default()
        .add_frame_filter(Box::new(|frames| {
            frames.retain(|frame| {
                frame.name.as_ref().map_or(true, |name| {
                    !name.starts_with("<rayon_core")
                        && !name.starts_with("rayon_core")
                        && !name.starts_with("<std::panic")
                        && !name.starts_with("std::panic")
                        && !name.starts_with("rayon")
                        && !name.starts_with("<rayon")
                })
            })
        }))
        .install()?;
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    Ok(())
}

#[derive(Debug, StructOpt)]
enum CommandLine {
    Infer {
        alignment: std::path::PathBuf,
        #[structopt(long = "paml")]
        paml: Option<std::path::PathBuf>,
        output: std::path::PathBuf,
        #[structopt(default_value = "1000", long = "rounds")]
        rounds: u32,
    },
    ExpandSearch {
        target: std::path::PathBuf,
        #[structopt(default_value = "8", long = "rounds")]
        rounds: u32,
        #[structopt(long = "seed")]
        seed: u64,
    },
    BenchMoves {
        target: std::path::PathBuf,
    },
    Reanalyze {
        target: std::path::PathBuf,
    },
    DebugMove {
        source: std::path::PathBuf,
        the_move: optimization::moves::GraphMove,
        #[structopt(default_value)]
        prefix: String,
    },
    ApplyMove {
        source: std::path::PathBuf,
        the_move: Vec<optimization::moves::GraphMove>,
        target: std::path::PathBuf,
    },
    ApplyGroup {
        source: std::path::PathBuf,
        index: usize,
        original: AminoAcid,
        replacement: AminoAcid,
        target: std::path::PathBuf,
    }
}

fn analyze(path: &Path, rounds: u32) -> Result<()> {
    let model = amino_acids::read_paml_matrix(std::fs::File::open(path.join("matrix.paml"))?)?;
    let alignment =
        alignment::read_alignment(std::fs::File::open(path.join("alignment.fasta"))?, &model)?;

    let mut graph = graph::Graph::new(&model, &alignment);

    if alignment.positions.is_empty() {
        eprintln!("All sequences are identical");
        serde_json::to_writer_pretty(
            std::fs::File::create(path.join("graph.json"))?,
            &graph.exported(),
        )?;
        return Ok(());
    }

    let mut star = graph.clone();
    for position in alignment.positions.ids() {
        star.set_amino_acid(NodeId(0), position, AminoAcid::Gap);
    }


    println!("Building initial nearest neighbor tree");
    optimization::nn_join(&mut graph);
    optimization::optimize_parameter(&mut graph);
    optimization::optimize_parameter(&mut star);
    graph.validate();

    dbg!(star.parameter(), graph.parameter());

    if star.probability() > graph.probability() {
        println!("Swapping nearest neighbor tree for preferred star phylogeny");
        std::mem::swap(&mut graph, &mut star);
    }

    let mut moves = std::fs::File::create(path.join("moves.log"))?;
    let mut rounds_log = std::fs::File::create(path.join("rounds.log"))?;

    let mut random = rand::rngs::StdRng::seed_from_u64(1337);

    loop {
        graph.validate();
        println!("Hill Climbing: {:?}", graph.probability());

        let mut new_graph = graph.clone();
        let made_moves = optimization::optimize(&mut new_graph);
        for m in made_moves {
            writeln!(
                &mut moves,
                "{:?}\t{:?}\t{:?}",
                m.the_move, m.probability, m.kind
            )?;
        }

        if new_graph.probability() > graph.probability() {
            graph = new_graph;
        } else {
            break;
        }
    }

    let mut buckets = (0..8)
        .map(|_| {
            let mut new_graph = graph.clone();
            let made_moves = optimization::moves::shuffle(&mut new_graph, &mut random, 7);
            (new_graph, made_moves)
        })
        .collect_vec();
    serde_json::to_writer_pretty(
        std::fs::File::create(path.join("graph.json"))?,
        &graph.exported(),
    )?;
    std::fs::write(path.join("parameter.txt"), format!("{}", graph.parameter().raw()))?;

    /*     while !optimization::moves::find_improvement(&mut graph, &mut random) {

    }*/

    let mut progress = pbr::ProgressBar::new(u64::from(rounds));
    for _ in 0..rounds / 8 {
        serde_json::to_writer_pretty(
            std::fs::File::create(path.join("graph.json"))?,
            &graph.exported(),
        )?;

        std::fs::write(path.join("parameter.txt"), format!("{}", graph.parameter().raw()))?;

        for y in 0..8 {
            writeln!(&mut rounds_log, "{:?}", graph.probability())?;
            let buckets2: Vec<_> = buckets
                .into_par_iter()
                .map(|(mut graph, mut made_moves)| {
                    let baseline = graph.probability();
                    made_moves.extend(optimization::optimize(&mut graph));
                    let probability = graph.probability();

                    (graph, probability > baseline, made_moves)
                })
                .collect();

            buckets = buckets2
                .into_iter()
                .enumerate()
                .map(|(index, (mut new_graph, changed, mut made_moves))| {
                    if new_graph.probability() > graph.probability() {
                        graph = new_graph.clone();
                        for the_move in made_moves.drain(..) {
                            writeln!(
                                &mut moves,
                                "{:?}\t{:?}\t{:?}",
                                the_move.the_move, the_move.probability, the_move.kind
                            )
                            .unwrap();
                        }
                    }
                    if index == y || !changed {
                        new_graph = graph.clone();
                        made_moves = optimization::moves::shuffle(&mut new_graph, &mut random, 7);
                    }
                    (new_graph, made_moves)
                })
                .collect();

            progress.message(&format!("{}: {:?} ", graph.classify(), graph.probability(),));
            progress.inc();
        }
    }
    Ok(())
}

fn expand(path: &Path, rounds: u32, seed: u64) -> Result<()> {
    let model = amino_acids::read_paml_matrix(std::fs::File::open(path.join("matrix.paml"))?)?;
    let alignment =
        alignment::read_alignment(std::fs::File::open(path.join("alignment.fasta"))?, &model)?;
    let parameter = r64(std::fs::read_to_string(path.join("parameter.txt"))?.parse()?);

    let mut moves = std::fs::OpenOptions::new()
        .append(true)
        .open(path.join("moves.log"))?;
    let mut rounds_log = std::fs::OpenOptions::new()
        .append(true)
        .open(path.join("rounds.log"))?;

    let mut random = rand::rngs::StdRng::seed_from_u64(seed);

    let exported = serde_json::from_reader(std::fs::File::open(path.join("graph.json"))?)?;

    let mut graph = graph::Graph::from_exported(&model, &alignment, parameter, &exported)?;

    let mut buckets = (0..8)
        .map(|_| {
            let mut new_graph = graph.clone();
            let made_moves = optimization::moves::shuffle(&mut new_graph, &mut random, 7);
            (new_graph, made_moves)
        })
        .collect_vec();
    serde_json::to_writer_pretty(
        std::fs::File::create(path.join("graph.json"))?,
        &graph.exported(),
    )?;

    let mut progress = pbr::ProgressBar::new(u64::from(rounds));
    for _ in 0..rounds / 8 {
        for y in 0..8 {
            writeln!(&mut rounds_log, "{:?}", graph.probability())?;
            let buckets2: Vec<_> = buckets
                .into_par_iter()
                .map(|(mut graph, mut made_moves)| {
                    let baseline = graph.probability();
                    made_moves.extend(optimization::moves::optimize(&mut graph));
                    let probability = graph.probability();

                    (graph, probability > baseline, made_moves)
                })
                .collect();

            buckets = buckets2
                .into_iter()
                .enumerate()
                .map(|(index, (mut new_graph, changed, mut made_moves))| {
                    if new_graph.probability() > graph.probability() {
                        graph = new_graph.clone();
                        for the_move in made_moves.drain(..) {
                            writeln!(
                                &mut moves,
                                "{:?}\t{:?}\t{:?}",
                                the_move.the_move, the_move.probability, the_move.kind
                            )?;
                        }
                        serde_json::to_writer_pretty(
                            std::fs::File::create(path.join("graph.json"))?,
                            &graph.exported(),
                        )?;
                    }
                    if index == y || !changed {
                        new_graph = graph.clone();
                        made_moves = optimization::moves::shuffle(&mut new_graph, &mut random, 7);
                    }
                    Ok((new_graph, made_moves))
                })
                .collect::<Result<Vec<_>>>()?;

            progress.message(&format!("{:?} ", graph.probability(),));
            progress.inc();
        }
    }
    /*
            for x in 0 .. 100 {
                println!("{} {:?}", x, graph.probability());
                let mut new_graph = (0..8).map(|_| {
                    let mut new_graph = graph.clone();
                    optimization::moves::shuffle(&mut new_graph, &mut random, 7);
                    new_graph
                }).collect_vec().into_par_iter().map(|mut new_graph| {

                    loop {
                        new_graph.validate();
                        eprintln!("{:?}", new_graph.probability());
                        new_graph.force_recompute();

                        let mut new_new_graph = new_graph.clone();
                        optimization::moves::optimize(&mut new_new_graph);
                        new_new_graph.force_recompute();

                        if new_new_graph.probability() > new_graph.probability() {
                            new_graph = new_new_graph;
                        } else {
                            break;
                        }
                    }

                    new_graph.force_recompute();
                    new_graph.validate();

                    let p = new_graph.probability();
                    (new_graph, p )
                }).max_by_key(|x| x.1).unwrap().0;
                graph.force_recompute();
                new_graph.force_recompute();
                if new_graph.probability() > graph.probability() {
                    graph = new_graph;
                }
            }
    */

    /*
    for _ in 0 .. 1000 {
        for t in 0 .. 20 {
            eprintln!("Wander {}",t);
            let mut graph = graph.clone();
            optimization::moves::shuffle(&mut graph, &mut random, t);
            loop {
                graph.validate();
                eprintln!("{:?}", graph.probability());
                graph.force_recompute();

                let mut new_graph = graph.clone();
                optimization::moves::optimize(&mut new_graph);
                new_graph.force_recompute();

                if new_graph.probability() > graph.probability() {
                    graph = new_graph;
                } else {
                    break;
                }
            }
        }
    }*/
    Ok(())
}

const REPORT_TEMPLATE: &[u8] = include_bytes!("./report-template.html");

fn bench_moves(path: &Path) -> Result<()> {
    let model = amino_acids::read_paml_matrix(std::fs::File::open(path.join("matrix.paml"))?)?;
    let alignment =
        alignment::read_alignment(std::fs::File::open(path.join("alignment.fasta"))?, &model)?;
    let parameter = r64(std::fs::read_to_string(path.join("parameter.txt"))?.parse()?);

    let exported = serde_json::from_reader(std::fs::File::open(path.join("graph.json"))?)?;

    let mut graph = graph::Graph::from_exported(&model, &alignment, parameter, &exported)?;

    graph.compact();

    graph.probability();

    optimization::moves::optimize(&mut graph);
    Ok(())
}

fn build_reports(path: &Path) -> Result<()> {
    let model = amino_acids::read_paml_matrix(std::fs::File::open(path.join("matrix.paml"))?)?;
    let alignment =
        alignment::read_alignment(std::fs::File::open(path.join("alignment.fasta"))?, &model)?;
    let parameter = r64(std::fs::read_to_string(path.join("parameter.txt"))?.parse()?);

    let exported = serde_json::from_reader(std::fs::File::open(path.join("graph.json"))?)?;

    let mut graph = graph::Graph::from_exported(&model, &alignment, parameter, &exported)?;

    dbg!(graph.probability());
    dbg!(graph.prior());
    dbg!(graph.likelihood());

    let mut output = std::fs::File::create(path.join("output.dot"))?;
    writeln!(output, "digraph {{")?;
    for node_id in graph.node_ids() {
        let label = match graph[node_id].kind {
            NodeKind::Leaf(leaf) => alignment.sequence_ids[leaf].to_string(),
            NodeKind::Root => "Root".to_string(),
            NodeKind::Other => format!("N{}", node_id.0),
        };
        writeln!(output, "N{} [shape=rectangle,label=<", node_id.0)?;
        writeln!(output, "<b>{}</b><br/>", label)?;
        if !graph[node_id].parents.is_empty() {
            for (position, node_amino_acid) in graph[node_id].amino_acids.iter() {
                let amino_acid = node_amino_acid.amino_acid;
                if amino_acid.is_amino_acid() || amino_acid == AminoAcid::Gap {
                    if graph.inherited_for_position(node_id, position).0 != amino_acid {
                        writeln!(
                            output,
                            "<i>{}{}[{}]</i><br/>",
                            position.0 + 1,
                            char::from(amino_acid.as_u8()),
                            node_amino_acid.height
                        )?;
                    }
                }
            }
        }
        writeln!(output, ">]")?;
        for &child in &graph[node_id].children {
            writeln!(output, "N{} -> N{}", child.0, node_id.0)?;
        }
    }
    writeln!(output, "}}")?;

    let mut output = std::fs::File::create(path.join("stats.json"))?;
    serde_json::to_writer_pretty(&mut output, &graph.full_stats())?;

    let mut output = std::fs::File::create(path.join("node-stats"))?;
    for node_id in graph.node_ids() {
        writeln!(&mut output, "{:?} {:?}", node_id, graph[node_id].stats)?;
    }

    let regex = regex::bytes::Regex::new("import ([A-Za-z0-9_]+) from\"../report.json\"").unwrap();

    let mut output = std::io::BufWriter::new(std::fs::File::create(path.join("report.html"))?);
    output.write_all(
        &regex.replace(REPORT_TEMPLATE, |caps: &regex::bytes::Captures| {
            format!(
                "var {} = {}",
                std::str::from_utf8(caps.get(1).unwrap().as_bytes()).unwrap(),
                serde_json::to_string(&exported).unwrap()
            )
        }),
    )?;

    let stats = graph.stats;

    let mut output = std::io::BufWriter::new(std::fs::File::create(path.join("details.txt"))?);
    writeln!(output, "Likelihood")?;
    writeln!(
        output,
        "\tInsertSeq\t{:?}\t\t{:?}",
        stats.insert_probability.unfix(),
        stats.insert_probability.unfix()
    )?;
    let maintains = stats.transitions.diagonal().sum();
    let changes = stats.transitions.sum() - maintains;
    writeln!(
        output,
        "\tTransitions \t{}\t{}\t{:?}",
        maintains,
        changes,
        graph.parameterized_model().likelihood(&stats.transitions)
    )?;
 
    // TODO transitions
    writeln!(
        output,
        "\tInserts  \t{}\t{}\t{:?}",
        stats.inserts.active,
        stats.inserts.inactive,
        stats.inserts.likelihood()
    )?;
    writeln!(
        output,
        "\tDeletes  \t{}\t{}\t{:?}",
        stats.deletes.active,
        stats.deletes.inactive,
        stats.deletes.likelihood()
    )?;
    writeln!(
        output,
        "\tInitial  \t{}\t{}\t{:?}",
        stats.initial.active,
        stats.initial.inactive,
        stats.initial.likelihood()
    )?;
    writeln!(output, "\tTotal     \t\t\t{:?}", graph.likelihood())?;
    writeln!(output, "Prior")?;
    writeln!(
        output,
        "\tPenalty  \t{:?}\t\t{:?}",
        stats.penalty,
        if stats.penalty > 0 {
            Log::zero()
        } else {
            Log::one()
        }
    )?;
    let other_nodes =
        i32::try_from(graph.node_ids().count() - graph.alignment().sequence_ids.len()).unwrap();
    writeln!(
        output,
        "\t+Nodes    \t{:?}\t{}\t{:?}",
        other_nodes,
        "",
        Log::betai(other_nodes, 2)
    )?;
    writeln!(
        output,
        "\tEdge Orders\t{:?}\t{}\t{:?}",
        graph.edge_count(),
        "",
        Log::one() / Log::gammai(i32::try_from(graph.edge_count()).unwrap() + 1)
    )?;
    writeln!(
        output,
        "\tReordering\t{:?}\t{}\t{:?}",
        other_nodes,
        "",
        Log::gammai(i32::try_from(other_nodes).unwrap() + 1)
    )?;
    if other_nodes == 1 {
        writeln!(
            output,
            "\tStar Adjustment\t{:?}\t{}\t{:?}",
            "",
            "",
            Log::pow2(n64(1.0))
        )?;
    }
    writeln!(output, "\tTotal     \t\t\t{:?}", graph.prior())?;
    writeln!(output, "Total     \t\t\t\t{:?}", graph.probability())?;
    Ok(())
}

fn fix_index(index: usize, alignment: &Alignment) -> PositionIndex {
    let mut mapping = Vec::new();
    let mut current_index = 0;
    for position in alignment.raw_positions.values() {
        if position.is_standard() {
            mapping.push(current_index);
            current_index += 1;
        } else {
            mapping.push(0);
        }
    }
    return PositionIndex(mapping[index]);
}

fn fix_indexes(the_move: &mut GraphMove, alignment: &Alignment) {
    let mut mapping = Vec::new();
    let mut current_index = 0;
    for position in alignment.raw_positions.values() {
        if position.is_standard() {
            mapping.push(current_index);
            current_index += 1;
        } else {
            mapping.push(0);
        }
    }

    match the_move {
        GraphMove::Refactor(_, _) => {}
        GraphMove::Remove(_) => {}
        GraphMove::AddEdge(_, _) => {}
        GraphMove::RemoveEdge(_, _) => {}
        GraphMove::ChangeEdge(_, _, _) => {}
        GraphMove::Reparent(_, _) => {}
        GraphMove::SetAminoAcid(_, index, _) => {
            *index = PositionIndex(mapping[index.0 as usize]);
        }
        GraphMove::FloodFill(_, index, _) => {
            *index = PositionIndex(mapping[index.0 as usize]);
        }
    }
}

fn main() -> Result<()> {
    cli_init()?;

    debug!("Parsing command line arguments");

    let args = CommandLine::from_args();

    debug!("Command line arguments: {:#?}", args);
    match args {
        CommandLine::Infer {
            alignment,
            output,
            paml,
            rounds,
        } => {
            std::fs::create_dir_all(&output)?;

            if let Some(paml) = paml {
                std::fs::copy(paml, output.join("matrix.paml"))?;
            } else {
                std::fs::write(
                    output.join("matrix.paml"),
                    include_bytes!("./BLOSUM62.paml"),
                )?;
            }
            std::fs::copy(&alignment, output.join("alignment.fasta"))?;

            analyze(&output, rounds)?;
            build_reports(&output)?;
        }
        CommandLine::ExpandSearch {
            target,
            rounds,
            seed,
        } => {
            expand(&target, rounds, seed)?;
            build_reports(&target)?;
        }
        CommandLine::BenchMoves { target } => {
            bench_moves(&target)?;
        }
        CommandLine::Reanalyze { target } => {
            build_reports(&target)?;
        }
        CommandLine::DebugMove {
            source,
            mut the_move,
            prefix,
        } => {
            let model =
                amino_acids::read_paml_matrix(std::fs::File::open(source.join("matrix.paml"))?)?;
            let alignment = alignment::read_alignment(
                std::fs::File::open(source.join("alignment.fasta"))?,
                &model,
            )?;
            let parameter = r64(std::fs::read_to_string(source.join("parameter.txt"))?.parse()?);

            fix_indexes(&mut the_move, &alignment);

            let exported =
                serde_json::from_reader(std::fs::File::open(source.join("graph.json"))?)?;

            let mut graph = graph::Graph::from_exported(&model, &alignment, parameter, &exported)?;

            let mut tracer = trace::BasicTracer::new(&prefix);
            optimization::moves::debug_move(&mut graph, the_move, &mut tracer);
        }
        CommandLine::ApplyMove {
            source,
            mut the_move,
            target,
        } => {
            std::fs::create_dir_all(&target)?;
            std::fs::copy(&source.join("matrix.paml"), target.join("matrix.paml"))?;
            std::fs::copy(&source.join("parameter.txt"), target.join("parameter.txt"))?;
            std::fs::copy(
                &source.join("alignment.fasta"),
                target.join("alignment.fasta"),
            )?;

            let model =
                amino_acids::read_paml_matrix(std::fs::File::open(source.join("matrix.paml"))?)?;
            let alignment = alignment::read_alignment(
                std::fs::File::open(source.join("alignment.fasta"))?,
                &model,
            )?;
            let parameter = r64(std::fs::read_to_string(source.join("parameter.txt"))?.parse()?);

            let exported =
                serde_json::from_reader(std::fs::File::open(source.join("graph.json"))?)?;

            let mut graph = graph::Graph::from_exported(&model, &alignment, parameter, &exported)?;

            for m in &mut the_move {
                fix_indexes(m, &alignment);
            }

            optimization::moves::apply_mutation(&mut graph, the_move);

            serde_json::to_writer_pretty(
                std::fs::File::create(target.join("graph.json"))?,
                &graph.exported(),
            )?;

            build_reports(&target)?;
        }
        CommandLine::ApplyGroup {
            source,
            index,
            original,
            replacement,
            target
        } => {
            std::fs::create_dir_all(&target)?;
            std::fs::copy(&source.join("matrix.paml"), target.join("matrix.paml"))?;
            std::fs::copy(&source.join("parameter.txt"), target.join("parameter.txt"))?;
            std::fs::copy(
                &source.join("alignment.fasta"),
                target.join("alignment.fasta"),
            )?;

            let model =
                amino_acids::read_paml_matrix(std::fs::File::open(source.join("matrix.paml"))?)?;
            let alignment = alignment::read_alignment(
                std::fs::File::open(source.join("alignment.fasta"))?,
                &model,
            )?;
            let parameter = r64(std::fs::read_to_string(source.join("parameter.txt"))?.parse()?);

            let exported =
                serde_json::from_reader(std::fs::File::open(source.join("graph.json"))?)?;

            let mut graph = graph::Graph::from_exported(&model, &alignment, parameter, &exported)?;

            let index = fix_index(index, &alignment);



            optimization::groups::apply_group(&mut graph, index, original, replacement);

            serde_json::to_writer_pretty(
                std::fs::File::create(target.join("graph.json"))?,
                &graph.exported(),
            )?;

            build_reports(&target)?;
        }
 
    }

    Ok(())
}
