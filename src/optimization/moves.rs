use crate::prelude::*;
use rand::prelude::*;

use super::amino_acids::analyze_amino_acids;
use reformation::Reformation;

#[derive(Clone)]
pub struct Guide {
    pub order: Vec<NodeId>,
    pub reachable: SlabMap<NodeId, SlabSet<NodeId>>,
}

impl Guide {
    pub fn new(graph: &Graph) -> Self {
        let mut released = SlabSet::new();
        let mut order = Vec::new();
        while !graph.node_ids().all(|x| released.contains(x)) {
            for node in graph.node_ids() {
                if !released.contains(node)
                    && graph[node]
                        .parents
                        .iter()
                        .all(|&parent_id| released.contains(parent_id))
                {
                    order.push(node);
                    released.insert(node);
                }
            }
        }

        let mut reachable: SlabMap<NodeId, SlabSet<NodeId>> = SlabMap::new();
        for &node_id in &order {
            let mut node_reachable = SlabSet::new();
            node_reachable.insert(node_id);
            for &parent in &graph[node_id].parents {
                for other_node_id in reachable[parent].iter() {
                    node_reachable.insert(other_node_id);
                }
            }
            reachable.insert(node_id, node_reachable);
        }

        Guide { order, reachable }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Reformation)]
pub enum GraphMove {
    #[reformation("refactor:{},{}")]
    Refactor(NodeId, NodeId),
    #[reformation("remove:{}")]
    Remove(NodeId),
    #[reformation("add-edge:{}-{}")]
    AddEdge(NodeId, NodeId),
    #[reformation("remove-edge:{}-{}")]
    RemoveEdge(NodeId, NodeId),
    #[reformation("change-edge:{}-{},{}")]
    ChangeEdge(NodeId, NodeId, NodeId),
    #[reformation("reparent:{}-{}")]
    Reparent(NodeId, NodeId),
    #[reformation("set-amino-acid:{}@{}={}")]
    SetAminoAcid(NodeId, PositionIndex, AminoAcid),
    #[reformation("flood:{}@{}={}")]
    FloodFill(NodeId, PositionIndex, AminoAcid),
}

impl std::str::FromStr for GraphMove {
    type Err = reformation::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        GraphMove::parse(s)
    }
}

fn generate_move(graph: &Graph, random: &mut Random) -> GraphMove {
    match random.gen_range(0..7) {
        0 => {
            let items = graph.node_ids().choose_multiple(random, 2);
            GraphMove::Refactor(items[0], items[1])
        }
        1 => GraphMove::Remove(graph.node_ids().choose(random).unwrap()),
        2 => {
            let items = graph.node_ids().choose_multiple(random, 2);
            GraphMove::AddEdge(items[0], items[1])
        }
        3 => {
            let source = graph
                .node_ids()
                .filter(|&x| !graph[x].parents.is_empty())
                .choose(random)
                .unwrap();
            let &destination = graph[source].parents.choose(random).unwrap();
            GraphMove::RemoveEdge(source, destination)
        }
        4 => {
            let source = graph
                .node_ids()
                .filter(|&x| !graph[x].parents.is_empty())
                .choose(random)
                .unwrap();
            let &destination = graph[source].parents.choose(random).unwrap();
            let new_destination = graph.node_ids().choose(random).unwrap();
            GraphMove::ChangeEdge(source, destination, new_destination)
        }
        5 => {
            let items = graph.node_ids().choose_multiple(random, 2);
            GraphMove::Reparent(items[0], items[1])
        }
        6 => {
            let source = graph
                .node_ids()
                .filter(move |&node_id| !graph[node_id].kind.is_leaf())
                .choose(random)
                .unwrap();
            let position = graph.alignment().positions.ids().choose(random).unwrap();

            let amino_acid = graph.alignment().positions[position]
                .candidates
                .choose(random)
                .copied()
                .unwrap_or(AminoAcid::Gap);
            GraphMove::SetAminoAcid(source, position, amino_acid)
        }
        _ => {
            unreachable!()
        }
    }
}

fn generate_moves(graph: &Graph) -> Vec<GraphMove> {
    graph
        .node_ids()
        .tuple_combinations()
        .map(|(lhs, rhs)| GraphMove::Refactor(lhs, rhs))
        .chain(graph.node_ids().map(GraphMove::Remove))
        .chain(
            graph
                .node_ids()
                .tuple_combinations()
                .map(|(lhs, rhs)| GraphMove::AddEdge(lhs, rhs)),
        )
        .chain(graph.node_ids().flat_map(|node| {
            graph[node]
                .parents
                .iter()
                .map(move |&parent| GraphMove::RemoveEdge(node, parent))
        }))
        .chain(graph.node_ids().flat_map(|node| {
            graph[node].parents.iter().flat_map(move |&parent| {
                graph
                    .node_ids()
                    .filter(move |&new_parent| new_parent != parent)
                    .map(move |new_parent| GraphMove::ChangeEdge(node, parent, new_parent))
            })
        }))
        .chain(
            graph
                .node_ids()
                .tuple_combinations()
                .map(|(lhs, rhs)| GraphMove::Reparent(lhs, rhs)),
        )
        .chain(
            graph
                .alignment()
                .positions
                .iter()
                .flat_map(|(index, details)| {
                    details.candidates.iter().flat_map(move |&amino_acid| {
                        graph
                            .node_ids()
                            .filter(move |&node_id| !graph[node_id].kind.is_leaf())
                            .map(move |node_id| GraphMove::SetAminoAcid(node_id, index, amino_acid))
                    })
                }),
        )
        .chain(
            graph
                .alignment()
                .positions
                .iter()
                .flat_map(|(index, details)| {
                    details.candidates.iter().flat_map(move |&amino_acid| {
                        graph
                            .node_ids()
                            .filter(move |&node_id| !graph[node_id].kind.is_leaf())
                            .map(move |node_id| GraphMove::FloodFill(node_id, index, amino_acid))
                    })
                }),
        )
        .collect()
}

impl GraphMove {
    fn valid(self, graph: &Graph, guide: &Guide) -> bool {
        match self {
            GraphMove::Refactor(lhs, rhs) => {
                graph.has_node_id(lhs)
                    && graph.has_node_id(rhs)
                    && graph[lhs].kind != NodeKind::Root
                    && graph[rhs].kind != NodeKind::Root
            }
            GraphMove::Remove(node) => {
                graph.has_node_id(node) && graph[node].kind == NodeKind::Other
            }
            GraphMove::AddEdge(lhs, rhs) => {
                graph.has_node_id(lhs)
                    && graph.has_node_id(rhs)
                    && !graph[lhs].parents.contains(&rhs)
                    && !guide.reachable[rhs].contains(lhs)
                    && graph[lhs].kind != NodeKind::Root
                    && !graph[rhs].kind.is_leaf()
            }
            GraphMove::RemoveEdge(source, destination) => {
                graph.has_node_id(source)
                    && graph.has_node_id(destination)
                    && graph[source].parents.contains(&destination)
                    && graph[source].parents.len() != 1
            }
            GraphMove::ChangeEdge(source, destination, new_destination) => {
                graph.has_node_id(source)
                    && graph.has_node_id(destination)
                    && graph.has_node_id(new_destination)
                    && graph[source].parents.contains(&destination)
                    && !guide.reachable[new_destination].contains(source)
                    && !graph[new_destination].kind.is_leaf()
            }
            GraphMove::Reparent(child, parent) => {
                graph.has_node_id(child)
                    && graph.has_node_id(parent)
                    && !guide.reachable[parent].contains(child)
                    && graph[child].kind != NodeKind::Root
                    && !graph[parent].kind.is_leaf()
            }
            GraphMove::SetAminoAcid(node_id, index, amino_acid) => {
                graph.has_node_id(node_id)
                    && graph[node_id].amino_acids[index].amino_acid != amino_acid
            }
            GraphMove::FloodFill(node_id, index, amino_acid) => {
                graph.has_node_id(node_id)
                    && graph[node_id].amino_acids[index].amino_acid != amino_acid
            }
        }
    }

    fn apply(self, graph: &mut Graph, guide: &Guide, tracer: &mut impl Tracer) -> Vec<NodeId> {
        match self {
            GraphMove::Refactor(lhs, rhs) => {
                let mut common_parents = graph[lhs]
                    .parents
                    .iter()
                    .copied()
                    .filter(|x| graph[rhs].parents.contains(x))
                    .collect_vec();

                if common_parents.is_empty() {
                    for node in guide.order.iter().rev().copied() {
                        if guide.reachable[lhs].contains(node)
                            && guide.reachable[rhs].contains(node)
                            && lhs != node
                            && rhs != node
                        {
                            common_parents.push(node);
                            break;
                        }
                    }
                }
                let new_node_id = graph.create_node(common_parents[0]);
                tracer.data("new-node-id", new_node_id);
                for &parent in &common_parents {
                    graph.remove_edge(lhs, parent);
                    graph.remove_edge(rhs, parent);
                    graph.add_edge(new_node_id, parent);
                }

                let mut to_reconsider = vec![new_node_id, lhs, rhs];
                to_reconsider.extend(common_parents.iter().copied());
                // 1 2

                graph.add_edge(lhs, new_node_id);
                graph.add_edge(rhs, new_node_id);

                common_parents.push(lhs);
                common_parents.push(rhs);
                common_parents.push(new_node_id);
                common_parents
            }
            GraphMove::Remove(node) => {
                analyze_amino_acids(graph, node, -1, tracer);

                let children = graph[node].children.clone();
                let parents = graph[node].parents.clone();
                for &child in &children {
                    graph.remove_edge(child, node);
                    let other = graph[child].parents.iter().copied().filter(|&x| x != node).collect_vec();
                    for &parent in &parents {
                        if !other.iter().any(|&x| guide.reachable[x].contains(parent)) {
                            graph.add_edge(child, parent);
                        }
                    }
                }
                for &parent in &parents {
                    graph.remove_edge(node, parent);
                }

                graph.remove_node(node);

                children.into_iter().chain(parents.into_iter()).collect()
            }
            GraphMove::AddEdge(lhs, rhs) => {
                graph.add_edge(lhs, rhs);
                for child in graph[rhs].children.clone() {
                    if guide.reachable[child].contains(lhs) {
                        graph.remove_edge(child, rhs);
                    }
                }
                vec![lhs, rhs]
            }
            GraphMove::RemoveEdge(source, destination) => {
                graph.remove_edge(source, destination);

                vec![source, destination]
            }
            GraphMove::ChangeEdge(source, destination, new_destination) => {
                graph.remove_edge(source, destination);
                graph.add_edge(source, new_destination);

                vec![source, destination, new_destination]
            }
            GraphMove::Reparent(child, parent) => {
                let old_parents = graph[child].parents.clone();
                for &old_parent in &old_parents {
                    graph.remove_edge(child, old_parent);
                }
                graph.add_edge(child, parent);

                let mut items = vec![child, parent];
                items.extend(old_parents.into_iter());
                items
            }
            GraphMove::SetAminoAcid(node_id, index, amino_acid) => {
                graph.set_amino_acid(node_id, index, amino_acid);
                Vec::new()
            }
            GraphMove::FloodFill(node_id, index, amino_acid) => {
                super::amino_acids::flood_fill(graph, node_id, index, amino_acid, tracer);
                Vec::new()
            }
        }
    }
}

fn quick_cleanup(graph: &mut Graph) {
    let dead = graph
        .node_ids()
        .filter(|&node| graph[node].children.len() < 2 && !graph[node].kind.is_leaf())
        .collect_vec();

    for node in dead {
        let children = graph[node].children.clone();
        let parents = graph[node].parents.clone();
        for &child in &children {
            graph.remove_edge(child, node);
            for &parent in &parents {
                graph.add_edge(child, parent);
            }
        }
        for &parent in &parents {
            graph.remove_edge(node, parent);
        }
        if graph[node].kind == NodeKind::Root {
            graph.make_root(children[0]);
        }
        graph.remove_node(node);
    }

    let dead = graph
        .node_ids()
        .filter(|&node| graph[node].children.len() < 2 && graph[node].kind == NodeKind::Other)
        .collect_vec();
    if !dead.is_empty() {
        quick_cleanup(graph);
    }
}

pub fn apply_mutation(new_graph: &mut Graph, moves: Vec<GraphMove>) {
    let current_guide = Guide::new(new_graph);

    for m in moves {
        let updated_nodes = m.apply(new_graph, &current_guide, &mut NullTracer);
        quick_cleanup(new_graph);

        let mut current = new_graph.probability();
        for &node in &updated_nodes {
            if new_graph.has_node_id(node) {
                let mut sub_graph = new_graph.clone();
                analyze_amino_acids(&mut sub_graph, node, 0, &mut NullTracer);
                if sub_graph.probability() > current {
                    current = sub_graph.probability();
                    *new_graph = sub_graph;
                }
            }
        }
    }
}

fn mutated<'a>(graph: &Graph<'a>, m: GraphMove, current_guide: &Guide) -> Graph<'a> {
    let mut new_graph = graph.clone();
    let updated_nodes = m.apply(&mut new_graph, &current_guide, &mut NullTracer);
    quick_cleanup(&mut new_graph);

    let mut current = new_graph.probability();
    for &node in &updated_nodes {
        if new_graph.has_node_id(node) {
            let mut sub_graph = new_graph.clone();
            analyze_amino_acids(&mut sub_graph, node, 0, &mut NullTracer);
            if sub_graph.probability() > current {
                current = sub_graph.probability();
                new_graph = sub_graph;
            }
        }
    }

    new_graph
}

pub fn shuffle(graph: &mut Graph, random: &mut Random, count: usize) -> Vec<MoveLog> {
    let mut moves = Vec::new();
    for _ in 0..count {
        let current_guide = Guide::new(graph);
        let mut selected_move = generate_move(graph, random);
        while !selected_move.valid(graph, &current_guide) {
            selected_move = generate_move(graph, random)
        }
        let new_graph = mutated(graph, selected_move, &current_guide);
        *graph = new_graph;
        moves.push(MoveLog {
            the_move: selected_move,
            probability: graph.probability(),
            kind: MoveLogKind::Random,
        });
    }
    moves
}

#[derive(Debug)]
pub enum MoveLogKind {
    Climbing,
    Random,
}

pub struct MoveLog {
    pub the_move: GraphMove,
    pub probability: Log,
    pub kind: MoveLogKind,
}

pub fn optimize(graph: &mut Graph) -> Vec<MoveLog> {
    let baseline = graph.probability();
    let mut current_guide = Guide::new(graph);
    let moves = generate_moves(graph);
    let mut logs = Vec::new();
    for (m, _) in moves
        .into_par_iter()
        .filter(|m| m.valid(graph, &current_guide))
        .map(|m| {
            let mut new_graph = mutated(graph, m, &current_guide);
            (m, new_graph.probability())
        })
        .filter(|x| x.1 > baseline)
        .collect::<Vec<_>>()
        .into_iter()
        .sorted_by_key(|x| std::cmp::Reverse(x.1))
    {
        if m.valid(&graph, &current_guide) {
            let mut new_graph = mutated(graph, m, &current_guide);
            if new_graph.probability() > graph.probability() {
                *graph = new_graph;
                current_guide = Guide::new(graph);
                logs.push(MoveLog {
                    the_move: m,
                    probability: graph.probability(),
                    kind: MoveLogKind::Climbing,
                })
            }
        }
    }

    logs
}

pub fn debug_move(graph: &mut Graph, the_move: GraphMove, tracer: &mut impl Tracer) {
    let mut new_graph = graph.clone();
    let guide = Guide::new(&new_graph);

    let updated_nodes = the_move.apply(&mut new_graph, &guide, tracer);
    quick_cleanup(&mut new_graph);

    let mut current = new_graph.probability();
    for &node in &updated_nodes {
        if new_graph.has_node_id(node) {
            let mut sub_graph = new_graph.clone();
            analyze_amino_acids(&mut sub_graph, node, 0, tracer);
            if sub_graph.probability() > current {
                current = sub_graph.probability();
                new_graph = sub_graph;
            }
        }
    }

    tracer.open("graph-probability");
    graph.probability_traced(tracer);
    tracer.open("new-probability");
    new_graph.probability_traced(tracer);

    tracer.open("stats");
    for node_id in graph
        .node_ids()
        .chain(new_graph.node_ids())
        .sorted()
        .dedup()
    {
        tracer.open_ex(|| format!("N{}", node_id.0));
        tracer.close(());
    }
    tracer.close(());

    /*

    println!("P {:?} -> {:?}", graph.probability(), new_graph.probability());

    for node in new_graph.node_ids() {
        for (position, node_amino_acid) in new_graph[node].amino_acids.iter() {
            if node_amino_acid.amino_acid.is_amino_acid() && node_amino_acid.inherited.map(|x| x.0) == Some(AminoAcid::Unknown) {
                println!("{:?} {:?} -> {:?} is incoherent", node, position, node_amino_acid.amino_acid);
                for &parent in &new_graph[node].parents {
                    let parent_amino_acid = new_graph[parent].amino_acids[position];
                    println!("\t{:?} {:?} @ {:?}", parent, parent_amino_acid.amino_acid, parent_amino_acid.height);
                 }

                 if graph.has_node_id(node) {
                     let old_amino_acid = graph[node].amino_acids[position];
                     println!("   was {:?}", old_amino_acid.inherited);
                 }
            }
        }
    }
    */
}
