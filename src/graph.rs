use crate::{
    amino_acids::{AminoAcidModel, ParameterizedAminoAcidModel},
    prelude::*,
};

define_slab_handle!(NodeId);

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum NodeKind {
    Root,
    Leaf(SequenceId),
    Other,
}

impl NodeKind {
    pub fn is_leaf(self) -> bool {
        matches!(self, NodeKind::Leaf(_))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Serialize)]
pub struct Stat {
    pub active: i32,
    pub inactive: i32,
}

impl Stat {
    pub fn record(&mut self, active: bool) {
        if active {
            self.active += 1;
        } else {
            self.inactive += 1;
        }
    }

    fn add(&mut self, stats: &Stat) {
        self.active += stats.active;
        self.inactive += stats.inactive;
    }

    fn subtract(&mut self, stats: &Stat) {
        self.active -= stats.active;
        self.inactive -= stats.inactive;
    }

    pub fn likelihood(&self) -> Log {
        Log::betai(self.active + 1, self.inactive + 1)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Serialize)]
pub struct Stats {
    pub penalty: i32,
    pub initial: Stat,
    pub insert_probability: FixedLog,
    pub deletes: Stat,
    pub inserts: Stat,
    pub transitions:
        nalgebra::Matrix<i32, nalgebra::U20, nalgebra::U20, nalgebra::ArrayStorage<i32, 20, 20>>,
}

impl Stats {
    pub fn record_transition(&mut self, source: AminoAcid, dest: AminoAcid) {
        self.transitions[(source.as_index().unwrap(), dest.as_index().unwrap())] += 1;
    }

    fn add(&mut self, stats: &Stats) {
        self.insert_probability *= stats.insert_probability;
        self.penalty += stats.penalty;
        self.initial.add(&stats.initial);
        self.transitions += stats.transitions;
        self.deletes.add(&stats.deletes);
        self.inserts.add(&stats.inserts);
    }

    fn subtract(&mut self, stats: &Stats) {
        self.insert_probability /= stats.insert_probability;
        self.penalty -= stats.penalty;
        self.initial.subtract(&stats.initial);
        self.transitions -= stats.transitions;
        self.deletes.subtract(&stats.deletes);
        self.inserts.subtract(&stats.inserts);
    }

    fn likelihood(&self, model: &ParameterizedAminoAcidModel) -> Log {
        self.insert_probability.unfix()
            * self.deletes.likelihood()
            * self.inserts.likelihood()
            * self.initial.likelihood()
            * model.likelihood(&self.transitions)
    }

    fn prior(&self) -> Log {
        if self.penalty > 0 {
            Log::zero()
        } else {
            Log::one()
            /*
            self.alternates_factor.unfix()
                * if self.extra_parent.active == 0 {
                    Log::one()
                } else {
                    self.extra_parent.likelihood()
                }
                * self.extra_child.likelihood_given_count()*/
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Inheritance(pub AminoAcid, pub u8);

impl Inheritance {
    fn new() -> Self {
        Inheritance(AminoAcid::Gap, 0)
    }

    pub fn update(self, node_amino_acid: NodeAminoAcid) -> Self {
        if node_amino_acid.height > self.1 {
            Inheritance(node_amino_acid.amino_acid, node_amino_acid.height)
        } else if node_amino_acid.height == self.1 && node_amino_acid.amino_acid != self.0 {
            Inheritance(AminoAcid::Unknown, node_amino_acid.height)
        } else {
            self
        }
    }

    pub fn changes(&self, other: AminoAcid) -> i32 {
        if self.0 == other {
            0
        } else {
            1
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct NodeAminoAcid {
    pub inherited: Option<Inheritance>,
    pub amino_acid: AminoAcid,
    pub pending: bool,
    pub height: u8,
}

#[derive(Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub parents: Vec<NodeId>,
    pub children: Vec<NodeId>,
    pub amino_acids: FixedVec<PositionIndex, NodeAminoAcid>,
    pub stats: Option<Stats>,
    pub parents_dirty: bool,
    pub dirty_positions: Vec<PositionIndex>,
}

impl Node {
    fn compute_stats(&self, alignment: &Alignment, amino_acid_model: &AminoAcidModel) -> Stats {
        let mut stats = match self.kind {
            NodeKind::Root => alignment.root_stats,
            NodeKind::Leaf(index) => alignment.sequence_stats[index],
            NodeKind::Other => alignment.other_stats,
        }
        .clone();

        if self.kind == NodeKind::Root {
            self.amino_acids.values().for_each(|&amino_acid| {
                if amino_acid.amino_acid.is_amino_acid() {
                    stats.initial.record(true);
                    stats.insert_probability *= amino_acid_model.initial(amino_acid.amino_acid);
                }
            });
            stats.initial.record(false);
        } else {
            self.amino_acids.values().for_each(|&amino_acid| {
                let Inheritance(inherited, _) = amino_acid.inherited.unwrap();

                match (inherited, amino_acid.amino_acid) {
                    (_, AminoAcid::Unknown) => {}
                    (AminoAcid::Gap, AminoAcid::Gap) => {}
                    (AminoAcid::Gap, _) => {
                        // insert
                        stats.inserts.record(true);
                        stats.insert_probability *=
                            amino_acid_model.initial(amino_acid.amino_acid);
                    }
                    (_, AminoAcid::Gap) => {
                        // delete
                        stats.deletes.record(true);
                        stats.inserts.record(false);
                    }
                    (AminoAcid::Unknown, _) => {
                        stats.penalty += 1;
                    }
                    (_, _) => {
                        stats.deletes.record(false);
                        stats.inserts.record(false);
                        stats.record_transition(inherited, amino_acid.amino_acid);
                    }
                }
            });

            stats.inserts.record(false);
        }

        if !matches!(self.kind, NodeKind::Root) {
            if self.parents.len() < 1 {
                stats.penalty += 1;
            }
        }

        if !matches!(self.kind, NodeKind::Leaf(_)) {
            if self.children.len() < 2 {
                stats.penalty += 1;
            }
        }

        stats
    }

    fn compute_inherited_for_position(
        &self,
        nodes: &Slab<NodeId, Arc<Node>>,
        position: PositionIndex,
    ) -> Inheritance {
        self.parents
            .iter()
            .fold(Inheritance::new(), |inheritance, &parent| {
                inheritance.update(nodes[parent].amino_acids[position])
            })
    }

    fn compute_height_for_position(&self, position: PositionIndex) -> u8 {
        let Inheritance(inherited, height) = self.amino_acids[position].inherited.unwrap();
        if inherited == self.amino_acids[position].amino_acid {
            height
        } else {
            height + 1
        }
    }
}

#[derive(Clone)]
pub struct TopologicalOrder {
    pub order: Vec<NodeId>,
    pub indexes: SlabMap<NodeId, usize>,
    pub next_index: usize,
}

#[derive(Clone)]
pub struct Graph<'a> {
    alignment: &'a Alignment,
    amino_acid_model: &'a AminoAcidModel,
    nodes: Slab<NodeId, Arc<Node>>,
    edge_count: u32,
    topological_order: Arc<TopologicalOrder>,
    dirty: bool,
    parameterized_model: Arc<ParameterizedAminoAcidModel>,

    pub prior_adjustment: Option<Log>,
    pub stats: Stats,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ExportedNodeKind {
    Leaf,
    Root,
    Other,
}

#[derive(Serialize, Deserialize)]
pub struct ExportedNode {
    kind: ExportedNodeKind,
    sequence_id: Option<String>,
    parents: Vec<String>,
    pub amino_acids: String,
}

impl<'a> Graph<'a> {
    pub fn from_exported(
        amino_acid_model: &'a AminoAcidModel,
        alignment: &'a Alignment,
        parameter: R64,
        exported: &indexmap::IndexMap<String, ExportedNode>,
    ) -> Result<Self> {
        let mapping = exported
            .iter()
            .map(|(key, node)| {
                Ok((
                    NodeId(key[1..].parse()?),
                    Arc::new(Node {
                        kind: match node.kind {
                            ExportedNodeKind::Leaf => NodeKind::Leaf(
                                alignment
                                    .sequence_ids
                                    .iter()
                                    .find(|x| Some(x.1) == node.sequence_id.as_ref())
                                    .ok_or_else(|| {
                                        eyre!("Could not find sequence {:?}", node.sequence_id)
                                    })?
                                    .0,
                            ),
                            ExportedNodeKind::Other => NodeKind::Other,
                            ExportedNodeKind::Root => NodeKind::Root,
                        },
                        parents: Vec::new(),
                        children: Vec::new(),
                        amino_acids: FixedVec::from_raw(
                            node.amino_acids
                                .bytes()
                                .zip(alignment.raw_positions.values())
                                .filter_map(|(b, raw)| {
                                    if raw.is_standard() {
                                        Some(AminoAcid::from_u8(b).map(|amino_acid| {
                                            NodeAminoAcid {
                                                inherited: None,
                                                amino_acid,
                                                pending: true,
                                                height: 0,
                                            }
                                        }))
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Result<Vec<_>>>()?,
                        ),
                        stats: None,
                        parents_dirty: true,
                        dirty_positions: Vec::new(),
                    }),
                ))
            })
            .collect::<Result<FnvHashMap<NodeId, _>>>()?;

        let nodes = Slab::from_map(mapping);

        let mut graph = Graph {
            topological_order: Arc::new(TopologicalOrder {
                order: nodes.ids().collect_vec(),
                indexes: SlabMap::new(),
                next_index: 0,
            }),
            parameterized_model: Arc::new(amino_acid_model.parameterize(parameter)),
            alignment,
            amino_acid_model,
            nodes,
            edge_count: 0,
            stats: Stats::default(),
            prior_adjustment: None,
            dirty: true,
        };

        graph.update_topological_order();

        for (node_label, node) in exported {
            let node_id = NodeId(node_label[1..].parse()?);
            for parent in &node.parents {
                let parent_node_id = NodeId(parent[1..].parse()?);
                graph.add_edge(node_id, parent_node_id);
            }
        }

        Ok(graph)
    }

    pub fn exported(&self) -> indexmap::IndexMap<String, ExportedNode> {
        self.nodes
            .iter()
            .map(|(node_id, node)| {
                let mut amino_acids = String::new();
                let mut node_amino_acids = node.amino_acids.values();
                for raw in self.alignment.raw_positions.values() {
                    let amino_acid = match raw {
                        crate::alignment::RawPosition::Standard(_) => {
                            node_amino_acids.next().unwrap().amino_acid
                        }
                        crate::alignment::RawPosition::Simple(reference, amino_acid) => {
                            match node.kind {
                                NodeKind::Root => *reference,
                                NodeKind::Leaf(index) => amino_acid[index],
                                NodeKind::Other => *reference,
                            }
                        }
                    };
                    amino_acids.push(char::from(amino_acid.as_u8()));
                }

                (
                    format!("N{}", node_id.0),
                    ExportedNode {
                        kind: match node.kind {
                            NodeKind::Leaf(_) => ExportedNodeKind::Leaf,
                            NodeKind::Root => ExportedNodeKind::Root,
                            NodeKind::Other => ExportedNodeKind::Other,
                        },
                        sequence_id: match node.kind {
                            NodeKind::Leaf(sequence) => {
                                Some(self.alignment.sequence_ids[sequence].clone())
                            }
                            _ => None,
                        },
                        parents: node
                            .parents
                            .iter()
                            .sorted()
                            .map(|parent| format!("N{}", parent.0))
                            .collect(),
                        amino_acids,
                    },
                )
            })
            .collect()
    }

    pub fn compact(&mut self) {
        let mut nodes: Slab<NodeId, Arc<Node>> = Slab::new();
        let mut mapping = FnvHashMap::default();

        for (node_id, node) in self.nodes.iter() {
            let new_node_id = nodes.insert(Arc::clone(node));
            mapping.insert(node_id, new_node_id);
        }

        for node in nodes.values_mut() {
            let node = Arc::make_mut(node);
            node.parents = node.parents.iter().map(|x| mapping[x]).collect();
            node.children = node.children.iter().map(|x| mapping[x]).collect();
        }

        self.nodes = nodes;
        let order = Arc::make_mut(&mut self.topological_order);
        order.order = self.nodes.ids().collect();
        self.update_topological_order();
    }

    pub fn new(amino_acid_model: &'a AminoAcidModel, alignment: &'a Alignment) -> Self {
        let mut nodes = Slab::new();

        let root = Arc::new(Node {
            kind: NodeKind::Root,
            parents: Vec::new(),
            children: Vec::new(),
            amino_acids: alignment.positions.make_vec(|_, data| NodeAminoAcid {
                inherited: None,
                pending: true,
                amino_acid: data.counts.iter().max_by_key(|x| x.1).unwrap().0,
                height: 0,
            }),
            stats: None,
            parents_dirty: true,
            dirty_positions: Vec::new(),
        });

        let root_id = nodes.insert(root.clone());

        let child_nodes = alignment
            .sequence_ids
            .ids()
            .map(|sequence_id| {
                let child = Arc::new(Node {
                    kind: NodeKind::Leaf(sequence_id),
                    parents: vec![root_id],
                    children: Vec::new(),
                    amino_acids: alignment.positions.make_vec(|_, data| NodeAminoAcid {
                        inherited: None,
                        amino_acid: data.sequences[sequence_id],
                        height: 0,
                        pending: true,
                    }),
                    stats: None,
                    parents_dirty: true,
                    dirty_positions: Vec::new(),
                });

                let child_id = nodes.insert(child);
                child_id
            })
            .collect();

        Arc::make_mut(&mut nodes[root_id]).children = child_nodes;

        let mut graph = Graph {
            topological_order: Arc::new(TopologicalOrder {
                order: nodes.ids().collect_vec(),
                indexes: SlabMap::new(),
                next_index: 0,
            }),
            alignment,
            amino_acid_model,
            nodes,
            edge_count: u32::try_from(alignment.sequence_ids.len()).unwrap(),
            stats: Stats::default(),
            prior_adjustment: None,
            dirty: true,
            parameterized_model: Arc::new(amino_acid_model.parameterize(r64(1.0))),
        };
        graph.update_topological_order();
        graph
    }

    pub fn alignment(&self) -> &'a Alignment {
        self.alignment
    }

    pub fn amino_acid_model(&self) -> &'a AminoAcidModel {
        self.amino_acid_model
    }

    pub fn parameterized_model(&self) -> &ParameterizedAminoAcidModel {
        &self.parameterized_model
    }

    pub fn set_parameter(&mut self, parameter: R64) {
        self.parameterized_model = Arc::new(self.amino_acid_model.parameterize(parameter));
    }

    pub fn parameter(&self) -> R64 {
        self.parameterized_model.parameter
    }

    fn compute_prior_adjustment(&self) -> Log {
        let other_nodes =
            i32::try_from(self.nodes.len() - self.alignment.sequence_ids.len()).unwrap();
        let extra_edges = self.edge_count + 1 - self.nodes().len() as u32;
        Log::betai(other_nodes, 2)
        * if extra_edges == 0 { Log::one() } else {Log::betai(extra_edges as i32 + 1, self.nodes.len() as i32)}
        * Log::from(other_nodes).powi(-(self.edge_count as i32))
            * Log::gammai(
                other_nodes
            )
            * if other_nodes == 1 {
                Log::pow2(n64(1.0))
            } else {
                Log::one()
            }
    }

    pub fn edge_count(&self) -> u32 {
        self.edge_count
    }

    pub fn probability(&mut self) -> Log {
        self.probability_traced(&mut NullTracer)
    }

    pub fn probability_traced(&mut self, tracer: &mut impl Tracer) -> Log {
        let prior = self.prior();
        let likelihood = self.likelihood();
        tracer.data("prior", prior);
        tracer.data("likelihood", likelihood);
        tracer.close(prior * likelihood)
    }

    fn update_topological_order(&mut self) {
        let mut remaining = self.nodes.len();
        let mut released = SlabSet::new_with_capacity_of(&self.nodes);
        let mut order = Vec::with_capacity(remaining);
        while remaining > 0 {
            self.topological_order.order.iter().for_each(|&node_id| {
                if !released.contains(node_id)
                    && self.nodes[node_id]
                        .parents
                        .iter()
                        .all(|&parent_id| released.contains(parent_id))
                {
                    order.push(node_id);
                    released.insert(node_id);
                    remaining -= 1;
                }
            })
        }

        let mut indexes = SlabMap::new();
        for (index, &node) in order.iter().enumerate() {
            indexes.insert(node, index);
        }

        self.topological_order = Arc::new(TopologicalOrder {
            next_index: order.len(),
            order,
            indexes,
        })
    }

    pub fn topological_order(&self) -> Arc<TopologicalOrder> {
        self.topological_order.clone()
    }

    pub fn ensure_derived(&mut self) {
        if !self.dirty {
            return;
        }

        for &node_id in &self.topological_order().order {
            if !self.nodes[node_id].parents_dirty && self.nodes[node_id].dirty_positions.is_empty()
            {
                continue;
            }
            let positions = if self.nodes[node_id].parents_dirty {
                Arc::make_mut(&mut self.nodes[node_id])
                    .dirty_positions
                    .clear();
                either::Left(self.alignment.positions.ids())
            } else {
                let mut empty = Vec::new();
                std::mem::swap(
                    &mut empty,
                    &mut Arc::make_mut(&mut self.nodes[node_id]).dirty_positions,
                );
                empty.sort();
                empty.dedup();
                either::Right(empty.into_iter())
            };

            let mut position_changed = false;

            positions.for_each(|position| {
                let node_amino_acid = self.nodes[node_id].amino_acids[position];
                let mut incoming_changed = false;
                if self.nodes[node_id].parents_dirty || node_amino_acid.inherited.is_none() {
                    let inherited =
                        self.nodes[node_id].compute_inherited_for_position(&self.nodes, position);
                    let node_amino_acid =
                        &mut Arc::make_mut(&mut self.nodes[node_id]).amino_acids[position];
                    if node_amino_acid.inherited != Some(inherited) {
                        node_amino_acid.inherited = Some(inherited);
                        incoming_changed = true;
                        position_changed = true;
                    }
                }

                let mut height_changed = false;

                if incoming_changed || node_amino_acid.pending {
                    let height = self.nodes[node_id].compute_height_for_position(position);

                    let node_amino_acid =
                        &mut Arc::make_mut(&mut self.nodes[node_id]).amino_acids[position];
                    if node_amino_acid.height != height {
                        node_amino_acid.height = height;
                        height_changed = true;
                    }
                }

                if node_amino_acid.pending {
                    position_changed = true;
                }

                if node_amino_acid.pending || height_changed {
                    for &child in &self.nodes[node_id].clone().children {
                        let child = Arc::make_mut(&mut self.nodes[child]);
                        if !child.parents_dirty {
                            child.dirty_positions.push(position);
                        }
                        child.amino_acids[position].inherited = None;
                    }
                    Arc::make_mut(&mut self.nodes[node_id]).amino_acids[position].pending = false;
                }
            });
            if position_changed {
                self.ensure_node_dirty(node_id);
            }
            if self.nodes[node_id].parents_dirty {
                Arc::make_mut(&mut self.nodes[node_id]).parents_dirty = false;
            }
        }
    }

    pub fn ensure_clean(&mut self) {
        if self.dirty {
            self.ensure_derived();
            let mut current = self.nodes.first_id();
            while let Some(node_id) = current {
                if self.nodes[node_id].stats.is_none() {
                    let stats =
                        self.nodes[node_id].compute_stats(self.alignment, self.amino_acid_model);
                    self.stats.add(&stats);
                    Arc::make_mut(&mut self.nodes[node_id]).stats = Some(stats);
                }
                current = self.nodes.next_id(node_id);
            }
            self.dirty = false;
        }
    }

    pub fn likelihood(&mut self) -> Log {
        self.ensure_clean();
        self.stats.likelihood(&self.parameterized_model)
    }

    pub fn prior(&mut self) -> Log {
        self.ensure_clean();
        if self.prior_adjustment.is_none() {
            self.prior_adjustment = Some(self.compute_prior_adjustment());
        }
        self.stats.prior() * self.prior_adjustment.unwrap()
    }

    pub fn root(&self) -> NodeId {
        self.nodes
            .iter()
            .filter(|x| x.1.kind == NodeKind::Root)
            .map(|x| x.0)
            .next()
            .unwrap()
    }

    pub fn add_edge(&mut self, source: NodeId, destination: NodeId) {
        if self.nodes[source].parents.contains(&destination) {
            // already have this edge
            return;
        }
        assert!(self.nodes[source].kind != NodeKind::Root);
        assert!(!matches!(self.nodes[destination].kind, NodeKind::Leaf(_)));
        self.ensure_node_dirty(source);
        self.ensure_node_dirty(destination);
        self.ensure_prior_adjustment_dirty();

        {
            let node = Arc::make_mut(&mut self.nodes[source]);
            node.parents.push(destination);
            node.parents_dirty = true;
        }
        {
            let node = Arc::make_mut(&mut self.nodes[destination]);
            node.children.push(source);
        }
        self.edge_count += 1;

        if self.topological_order.indexes[source] < self.topological_order.indexes[destination] {
            self.update_topological_order()
        }
        self.dirty = true;
    }

    fn ensure_node_dirty(&mut self, node: NodeId) {
        if let Some(stats) = Arc::make_mut(&mut self.nodes[node]).stats.take() {
            self.stats.subtract(&stats);
        }
    }

    pub fn remove_node(&mut self, source: NodeId) {
        assert!(self.nodes[source].parents.is_empty());
        assert!(self.nodes[source].children.is_empty());
        self.ensure_node_dirty(source);
        self.ensure_prior_adjustment_dirty();

        self.nodes.remove(source);
        let order = Arc::make_mut(&mut self.topological_order);
        order.order.retain(|&x| x != source);
        self.dirty = true;
    }

    pub fn remove_edge(&mut self, source: NodeId, destination: NodeId) {
        if !self.nodes[source].parents.contains(&destination) {
            // edge does not exist, so can't remove it
            return;
        }
        self.ensure_node_dirty(source);
        self.ensure_node_dirty(destination);
        self.ensure_prior_adjustment_dirty();

        {
            let node = Arc::make_mut(&mut self.nodes[source]);
            node.parents.retain(|&x| x != destination);
            node.parents_dirty = true;
        }
        {
            let node = Arc::make_mut(&mut self.nodes[destination]);
            node.children.retain(|&x| x != source);
        }
        self.edge_count -= 1;
        self.dirty = true;
    }

    fn ensure_prior_adjustment_dirty(&mut self) {
        self.prior_adjustment = None;
    }

    pub fn create_node(&mut self, copy: NodeId) -> NodeId {
        let new_node = Node {
            kind: NodeKind::Other,
            parents: Vec::new(),
            children: Vec::new(),
            amino_acids: self.nodes[copy]
                .amino_acids
                .make_vec(|_index, value| NodeAminoAcid {
                    inherited: None,
                    amino_acid: value.amino_acid,
                    pending: true,
                    height: 0,
                }),
            stats: None,
            parents_dirty: true,
            dirty_positions: Vec::new(),
        };

        self.ensure_prior_adjustment_dirty();

        let new_node_id = self.nodes.insert(Arc::new(new_node));

        let order = Arc::make_mut(&mut self.topological_order);
        order.indexes.insert(new_node_id, order.next_index);
        order.next_index += 1;
        order.order.push(new_node_id);
        self.dirty = true;

        new_node_id
    }

    pub fn make_root(&mut self, node_id: NodeId) {
        assert!(self.nodes[node_id].kind == NodeKind::Other);
        Arc::make_mut(&mut self.nodes[node_id]).kind = NodeKind::Root;
    }

    pub fn validate(&mut self) {
        self.ensure_clean();

        let edge_count: u32 = self
            .nodes
            .values()
            .map(|x| u32::try_from(x.parents.len()).unwrap())
            .sum();
        assert_eq!(edge_count, self.edge_count);

        let mut stats = Stats::default();

        for (node_id, node) in self.nodes.iter() {
            assert_eq!(
                node.stats.as_ref().unwrap(),
                &node.compute_stats(self.alignment, self.amino_acid_model)
            );

            for position in self.alignment.positions.ids() {
                let height = node.amino_acids[position].height;
                assert_eq!(
                    height,
                    node.compute_height_for_position(position),
                    "{:?} {:?} Recorded: {:?} Actual: {:?}",
                    node_id,
                    position,
                    height,
                    node.compute_height_for_position(position)
                );

                assert_eq!(
                    node.amino_acids[position].inherited.unwrap(),
                    node.compute_inherited_for_position(&self.nodes, position),
                    "{:?} {:?} Recorded: {:?} Actual: {:?}",
                    node_id,
                    position,
                    node.amino_acids[position].inherited.unwrap(),
                    node.compute_inherited_for_position(&self.nodes, position),
                );
            }

            stats.add(self.nodes[node_id].stats.as_ref().unwrap());
        }

        assert_eq!(stats.penalty, self.stats.penalty);

        assert_eq!(
            stats.insert_probability,
            self.stats.insert_probability
        );
        assert_eq!(stats.transitions, self.stats.transitions);
        assert_eq!(stats.deletes, self.stats.deletes);
        assert_eq!(stats.inserts, self.stats.inserts);

        /*assert_almost_eq(
            "likelihood",
            self.likelihood,
            self.compute_hypothetical_likelihood(self.parameters),
        );*/

        for (node_id, node) in self.nodes.iter() {
            for &parent in &node.parents {
                assert!(self.nodes[parent].children.contains(&node_id));
            }
            for &child in &node.children {
                assert!(self.nodes[child].parents.contains(&node_id));
            }
        }

        let mut released = SlabSet::new();
        let mut order = Vec::new();
        while !self.node_ids().all(|x| released.contains(x)) {
            let mut changed = false;
            for node in self.node_ids() {
                if !released.contains(node)
                    && self[node]
                        .parents
                        .iter()
                        .all(|&parent_id| released.contains(parent_id))
                {
                    order.push(node);
                    released.insert(node);
                    changed = true;
                }
            }

            if !changed {
                panic!("cycle detected");
            }
        }
    }

    pub fn set_amino_acid(&mut self, node: NodeId, index: PositionIndex, amino_acid: AminoAcid) {
        assert!(!matches!(self.nodes[node].kind, NodeKind::Leaf(_)));
        assert!(amino_acid != AminoAcid::Unknown);
        if self.nodes[node].amino_acids[index].amino_acid == amino_acid {
            return;
        }

        {
            let node = Arc::make_mut(&mut self.nodes[node]);
            node.dirty_positions.push(index);

            let node_amino_acid = &mut node.amino_acids[index];
            node_amino_acid.amino_acid = amino_acid;
            node_amino_acid.pending = true;
        }
        self.dirty = true;
    }

    pub fn node_ids(&self) -> impl Iterator<Item = NodeId> + '_ + Clone {
        self.nodes.ids()
    }

    pub fn has_node_id(&self, node: NodeId) -> bool {
        self.nodes.contains(node)
    }

    pub fn inherited_for_position(&self, node: NodeId, position: PositionIndex) -> Inheritance {
        self.nodes[node].amino_acids[position].inherited.unwrap()
    }

    pub fn full_stats(&mut self) -> FullStats {
        FullStats {
            stats: self.stats,
            edge_count: self.edge_count,
            node_count: self.nodes.len(),
            leaf_count: self.alignment.sequence_ids.len(),
            probability: self.probability(),
            prior: self.prior(),
            likelihood: self.likelihood(),
            classification: self.classify(),
        }
    }

    pub fn nodes(&self) -> &Slab<NodeId, Arc<Node>> {
        &self.nodes
    }

    pub fn classify(&mut self) -> &'static str {
        self.ensure_clean();
        let extra_edges = self.edge_count - self.nodes().len() as u32 + 1;
        if self.nodes.len() == self.alignment.sequence_ids.len() + 1 {
            return "star";
        } else if extra_edges == 0 {
            return "tree";
        } else {
            return "dag";
        }
    }
}

impl<'a> std::ops::Index<NodeId> for Graph<'a> {
    type Output = Node;

    fn index(&self, index: NodeId) -> &Self::Output {
        &self.nodes[index]
    }
}

#[derive(Debug, Serialize)]
pub struct FullStats {
    #[serde(flatten)]
    stats: Stats,
    edge_count: u32,
    node_count: usize,
    leaf_count: usize,
    probability: Log,
    prior: Log,
    likelihood: Log,
    classification: &'static str,
}
