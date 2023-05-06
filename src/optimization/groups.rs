use crate::prelude::*;

pub fn apply_group(graph: &mut Graph, index: PositionIndex, original: AminoAcid, replacement: AminoAcid) {
    graph.ensure_derived();

    let mut nodes= Vec::new();

    for node_id in graph.node_ids() {
        let amino_acid = graph[node_id].amino_acids[index];
        if amino_acid.inherited.unwrap().0 == original && amino_acid.amino_acid == replacement {
            nodes.push(node_id);
        }
    }

    let guide = super::moves::Guide::new(graph);

    let amino_acid = replacement;
    
        

    let hook = guide
        .order
        .iter()
        .rev()
        .copied()
        .find(|&x| {
            nodes
                .iter()
                .all(|&y| y != x && guide.reachable[y].contains(x))
        })
        .unwrap();
    let new_node = graph.create_node(hook);
    graph.add_edge(new_node, hook);
    graph.set_amino_acid(new_node, index, amino_acid);
    for &node in &nodes {
        graph.add_edge(node, new_node);
    }
}

pub fn optimize_groups(graph: &mut Graph) {
    graph.ensure_derived();

    let mut candidates = Vec::new();

    for node_id in graph.node_ids() {
        for (index, amino_acid) in graph[node_id].amino_acids.iter() {
            if amino_acid.inherited.unwrap().0 != amino_acid.amino_acid && amino_acid.amino_acid != AminoAcid::Unknown {
                candidates.push((
                    index,
                    amino_acid.inherited.unwrap().0,
                    amino_acid.amino_acid,
                    node_id,
                ));
            }
        }
    }

    candidates.par_sort();

    let mut guide = super::moves::Guide::new(graph);

    for ((index, _inherited, amino_acid), items) in candidates
        .into_iter()
        .group_by(|x| (x.0, x.1, x.2))
        .into_iter()
    {
        let nodes = items.map(|x| x.3).collect_vec();

        if nodes.len() > 2
            && !nodes.contains(&graph.root())
            && nodes.iter().all(|x| graph.has_node_id(*x))
        {
            let mut new_graph = graph.clone();

            let hook = guide
                .order
                .iter()
                .rev()
                .copied()
                .find(|&x| {
                    nodes
                        .iter()
                        .all(|&y| y != x && guide.reachable[y].contains(x))
                })
                .unwrap();
            let new_node = new_graph.create_node(hook);
            new_graph.add_edge(new_node, hook);
            new_graph.set_amino_acid(new_node, index, amino_acid);
            for &node in &nodes {
                new_graph.add_edge(node, new_node);
            }
            if new_graph.probability() > graph.probability() {
                *graph = new_graph;
                guide = super::moves::Guide::new(graph);
            }
        }
    }
}
