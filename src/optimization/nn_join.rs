use crate::prelude::*;

use super::amino_acids::analyze_amino_acids;

pub fn nn_join(graph: &mut Graph) {
    let root = graph.root();
    loop {
        let candidates: Vec<_> = graph
            .node_ids()
            .filter(|&node| graph[node].parents.contains(&root))
            .collect();

        if candidates.len() <= 2 {
            break;
        }

        let (lhs, rhs) = candidates
            .into_iter()
            .tuple_combinations()
            .max_by_key(|&(lhs, rhs)| {
                graph[lhs]
                    .amino_acids
                    .values()
                    .zip(graph[rhs].amino_acids.values())
                    .filter(|(x, y)| x.amino_acid == y.amino_acid)
                    .count()
            })
            .unwrap();

        let node = graph.create_node(root);
        graph.remove_edge(lhs, root);
        graph.remove_edge(rhs, root);
        graph.add_edge(lhs, node);
        graph.add_edge(rhs, node);
        graph.add_edge(node, root);
        analyze_amino_acids(graph, lhs, 0, &mut NullTracer);
        analyze_amino_acids(graph, rhs, 0, &mut NullTracer);
        analyze_amino_acids(graph, node, 0, &mut NullTracer);
    }

    for node_id in graph.node_ids().collect_vec() {
        if graph[node_id].kind == NodeKind::Other {
            if graph[node_id]
                .amino_acids
                .values()
                .all(|amino_acid| amino_acid.inherited.unwrap().0 == amino_acid.amino_acid)
            {
                // this node always agrees with its parent
                let children = graph[node_id].children.clone();
                let parents = graph[node_id].parents.clone();
                for &child in &children {
                    graph.remove_edge(child, node_id);
                    for &parent in &parents {
                        graph.add_edge(child, parent);
                    }
                }
                for &parent in &parents {
                    graph.remove_edge(node_id, parent);
                }

                graph.remove_node(node_id);
            }
        }
    }
}
