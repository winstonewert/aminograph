use crate::{graph::Inheritance, prelude::*};

struct FloodContext<'a, T> {
    graph: &'a Graph<'a>,
    index: PositionIndex,
    tracer: &'a mut T,
    seen: SlabSet<NodeId>,
}

impl<'a, T: Tracer> FloodContext<'a, T> {
    fn flood(
        &mut self,
        new_inheritance: Inheritance,
        node_id: NodeId,
        new_amino_acid: AminoAcid,
        changes: &mut Vec<NodeId>,
    ) -> i32 {
        self.tracer.open("flood");
        /*if DEBUG {
            eprintln!(
                "    Flood {:?}  with {:?} {:?}",
                node_id, new_inheritance, new_amino_acid
            );
        }*/
        let current_inherits = self.graph.inherited_for_position(node_id, self.index);
        let current_amino_acid = self.graph[node_id].amino_acids[self.index].amino_acid;
        let current_changes = current_inherits.changes(current_amino_acid);
        let new_changes = new_inheritance.changes(new_amino_acid);

        let downstream = Inheritance(
            new_amino_acid,
            if new_amino_acid == new_inheritance.0 {
                new_inheritance.1
            } else {
                new_inheritance.1 + 1
            },
        );

        let mut delta_with_change = new_changes - current_changes;
        let local_changes = changes.len();

        for &child in &self.graph[node_id].children {
            if !self.seen.insert(child) {
                continue;
            }

            self.tracer.open_ex(|| format!("N{}", child.0));
            let current_inherits = self.graph.inherited_for_position(child, self.index);
            let new_inherits = self.graph[child]
                .parents
                .iter()
                .copied()
                .filter(|&x| x != node_id)
                .fold(downstream, |inheritance, parent| {
                    inheritance.update(self.graph[parent].amino_acids[self.index])
                });

            self.tracer.data("kind", self.graph[child].kind);
            self.tracer.data("current", current_inherits);
            self.tracer.data("new", new_inherits);
            self.tracer.data("downstream", downstream);

            let cost = if self.graph[child].kind.is_leaf()
                || new_inherits != downstream
                || self.graph[child].amino_acids[self.index].amino_acid != current_amino_acid
            {
                /*
                if DEBUG {
                    if self.graph[child].kind.is_leaf() {
                        eprint!("     {:?} LEAF", child);
                    } else if new_inherits != downstream {
                        for &parent in &self.graph[child].parents {
                            eprintln!("      {:?} {:?}", parent, self.graph[parent].amino_acids[self.index]);
                        }
                        eprintln!("      {:?} {:?}", "R", self.graph[NodeId(292)].amino_acids[self.index]);

                        eprint!(
                            "     {:?} Not Controlled {:?} {:?} {:?}",
                            child, new_inherits, downstream, current_inherits
                        );
                   } else if self.graph[child].amino_acids[self.index].amino_acid
                        != current_amino_acid
                    {
                        eprint!("     {:?} Changed", child);
                    }
                }*/
                // we cannot change the amino acids in the leaf
                let amino_acid = self.graph[child].amino_acids[self.index].amino_acid;
                /*
                if DEBUG {
                    eprintln!(
                        " {:} {:}",
                        new_inherits.changes(amino_acid),
                        current_inherits.changes(amino_acid)
                    );
                }*/
                new_inherits.changes(amino_acid) - current_inherits.changes(amino_acid)
            } else {
                self.flood(new_inherits, child, new_amino_acid, changes)
            };

            self.tracer.close(cost);
            delta_with_change += cost;
        }

        let delta_without_change = new_inheritance.changes(current_amino_acid) - current_changes;
        /*if DEBUG {
            //            dbg!(new_inheritance, current_amino_acid, current_changes);
            eprintln!(
                "    {:?} change: {} no-change: {}",
                node_id, delta_with_change, delta_without_change
            );
        }*/

        self.tracer.data("with-change", delta_with_change);
        self.tracer.data("without-change", delta_without_change);

        let result = if delta_with_change <= delta_without_change {
            changes.push(node_id);

            delta_with_change
        } else {
            changes.truncate(local_changes);
            delta_without_change
        };

        self.tracer.close(result)
    }
}

pub fn flood_fill(
    graph: &mut Graph,
    node: NodeId,
    index: PositionIndex,
    amino_acid: AminoAcid,
    tracer: &mut impl Tracer,
) {
    graph.ensure_derived();
    attempt_set(graph, node, index, amino_acid, -1, tracer);
}

fn attempt_set(
    graph: &mut Graph,
    node: NodeId,
    index: PositionIndex,
    amino_acid: AminoAcid,
    bias: i32,
    tracer: &mut impl Tracer,
) -> bool {
    tracer.open_ex(|| format!("N{}-{}-{}", node.0, index.0, amino_acid.as_u8() as char));
    tracer.data("bias", bias);
    let mut context = FloodContext {
        graph,
        index,
        tracer,
        seen: SlabSet::new_with_capacity_of(graph.nodes()),
    };
    let inherits = graph.inherited_for_position(node, index);
    let mut changes = Vec::new();
    let delta = context.flood(inherits, node, amino_acid, &mut changes);
    tracer.data("delta", delta);
    tracer.data("changes", &changes);
    tracer.close(if delta + bias < 0 && !changes.is_empty() {
        for change in changes {
            graph.set_amino_acid(change, index, amino_acid);
        }
        true
    } else {
        false
    })
}

pub fn analyze_amino_acids(graph: &mut Graph, node: NodeId, bias: i32, tracer: &mut impl Tracer) {
    tracer.open_ex(|| format!("analyze-N{:?}", node.0));
    graph.ensure_derived();

    let mut active_count = 0;

    'position: for position in graph.alignment().positions.ids() {
        let inherited = graph.inherited_for_position(node, position);
        let actual = graph[node].amino_acids[position].amino_acid;

        if inherited.0 != actual {
            tracer.open_ex(|| format!("{:?}", position.0));
            tracer.data("inherited", inherited);
            tracer.data("actual", actual);

            if actual != AminoAcid::Unknown {
                for parent in graph[node].parents.clone() {
                    if attempt_set(graph, parent, position, actual, bias, tracer) {
                        active_count += 1;
                        tracer.close('^');
                        continue 'position;
                    }
                }
            }

            if inherited.0 != AminoAcid::Unknown && !graph[node].kind.is_leaf() && (inherited.0 != AminoAcid::Gap || graph[node].kind != NodeKind::Root) {
                if attempt_set(graph, node, position, inherited.0, bias, tracer) {
                    active_count += 1;
                    tracer.close('v');
                    continue 'position;
                }
            }

            tracer.close('-');
        }
    }
    tracer.close(active_count);
}

#[cfg(test)]
mod test {
    use super::*;

    use std::io::Write;

    #[derive(Debug)]
    struct TestNode {
        before: AminoAcid,
        after: AminoAcid,
        analyze: bool,
        id: String,
        parents: Vec<String>,
    }

    fn check(description: &str) {
        lazy_static::lazy_static! {
            static ref PATTERN: regex::Regex = regex::Regex::new(r"^(?P<analyze>\*)?(?P<label>[0-9A-Za-z]+) (?P<before>[A-Z-])(>(?P<after>[A-Z-]))?( (->(?P<parents>( [0-9A-Za-z]+)+)))?$").unwrap();
        }

        let mut nodes = Vec::new();

        for line in description.split('\n') {
            let line = line.trim();
            if !line.is_empty() {
                match PATTERN.captures(line) {
                    Some(captures) => {
                        nodes.push(TestNode {
                            before: AminoAcid::from_u8(
                                captures
                                    .name("before")
                                    .unwrap()
                                    .as_str()
                                    .bytes()
                                    .next()
                                    .unwrap(),
                            )
                            .unwrap(),
                            after: AminoAcid::from_u8(
                                captures
                                    .name("after")
                                    .or(captures.name("before"))
                                    .unwrap()
                                    .as_str()
                                    .bytes()
                                    .next()
                                    .unwrap(),
                            )
                            .unwrap(),
                            analyze: captures.name("analyze").is_some(),
                            id: captures.name("label").unwrap().as_str().to_owned(),
                            parents: captures
                                .name("parents")
                                .map(|x| {
                                    x.as_str()
                                        .split(" ")
                                        .filter(|x| !x.is_empty())
                                        .map(|x| x.to_owned())
                                        .collect_vec()
                                })
                                .unwrap_or_else(Vec::new),
                        });
                    }
                    None => {
                        panic!("{:?} not valid line", line);
                    }
                }
            }
        }

        let mut non_leafs = fnv::FnvHashSet::default();
        for node in &nodes {
            for parent in &node.parents {
                non_leafs.insert(parent);
            }
        }

        let mut alignment = Vec::new();
        for node in &nodes {
            if !non_leafs.contains(&node.id) {
                writeln!(
                    &mut alignment,
                    ">{}\n{}",
                    &node.id,
                    char::from(node.before.as_u8())
                )
                .unwrap();
            }
        }

        let amino_acid_model = crate::amino_acids::read_paml_matrix(std::io::Cursor::new(
            include_bytes!("../BLOSUM62.paml"),
        ))
        .unwrap();
        let alignment =
            crate::alignment::read_alignment(std::io::Cursor::new(alignment), &amino_acid_model)
                .unwrap();

        let mut graph = Graph::new(&amino_acid_model, &alignment);

        let mut node_id_map = FnvHashMap::default();

        for node_id in graph.node_ids().collect_vec() {
            match graph[node_id].kind {
                NodeKind::Leaf(index) => {
                    node_id_map.insert(&alignment.sequence_ids[index], node_id);
                    for parent in graph[node_id].parents.clone() {
                        graph.remove_edge(node_id, parent);
                    }
                }
                NodeKind::Root => {
                    for node in &nodes {
                        if node.parents.is_empty() {
                            node_id_map.insert(&node.id, node_id);
                        }
                    }
                }
                NodeKind::Other => {
                    unreachable!()
                }
            }
        }

        for node in &nodes {
            if !node_id_map.contains_key(&node.id) {
                let node_id = graph.create_node(NodeId(0));
                node_id_map.insert(&node.id, node_id);
            }
        }

        for (key, value) in node_id_map.iter() {
            eprintln!("{:?} => {:?}", key, value);
        }

        for node in &nodes {
            let node_id = node_id_map[&node.id];
            if non_leafs.contains(&node.id) {
                graph.set_amino_acid(node_id, PositionIndex(0), node.before);
            }
            for parent in &node.parents {
                graph.add_edge(node_id, node_id_map[parent]);
            }
        }

        for node in &nodes {
            if node.analyze {
                let node_id = node_id_map[&node.id];
                analyze_amino_acids(&mut graph, node_id, 0, &mut NullTracer);
            }
        }

        for node in &nodes {
            let node_id = node_id_map[&node.id];
            let actual = graph[node_id].amino_acids[PositionIndex(0)].amino_acid;
            if actual != node.after {
                panic!(
                    "While testing:\n{}\nExpected {} to be {:?} but was {:?}",
                    description, &node.id, node.after, actual
                );
            }
        }
    }

    #[test]
    fn test_push_up_minority() {
        check(
            "
        *1 A -> 4
        2 T -> 4
        3 T -> 4
        4 T
        ",
        )
    }

    #[test]
    fn test_push_up_majority() {
        check(
            "
        1 A -> 4
        *2 T -> 4
        3 T -> 4
        4 A>T
        ",
        )
    }

    #[test]
    fn test_push_up_inherit_not() {
        check(
            "
        *1 A -> 3
        2 T -> 3
        3 T -> 5
        4 T -> 5
        5 T
        ",
        )
    }

    #[test]
    fn test_simple_push_up() {
        check(
            "
        *1 A -> 3
        2 A -> 3
        3 T>A
        ",
        )
    }

    #[test]
    fn test_simple_push_up_above() {
        check(
            "
        Dog A -> Carn
        Cat A -> Carn
        Mouse A -> Rodent
        Rat A -> Rodent
        *Carn A -> Mammal
        Rodent A -> Mammal
        Mammal T>A
        ",
        );
    }

    #[test]
    fn test_simple_push_up_above_partial() {
        check(
            "
        Dog A -> Carn
        Cat A -> Carn
        Mouse A -> Rodent
        Rat T -> Rodent
        *Carn A -> Mammal
        Rodent T>A -> Mammal
        Mammal T>A
        ",
        );
    }

    #[test]
    fn test_more_complex() {
        check(
            "
        Dog A -> Carn
        Cat A -> Carn
        Mouse A -> Rodent Secondary
        Rat T -> Rodent
        Carn A -> Mammal Secondary
        Rodent T>A -> Mammal
        Mammal T>A
        *Secondary A -> Mammal
        ",
        );
    }

    #[test]
    fn test_down() {
        check(
            "
        Dog A -> Carn
        Cat A -> Carn
        Mouse A -> Rodent
        Rat T -> Rodent
        Carn A -> Mammal
        Rodent T>A -> Mammal
        *Mammal T>A -> Root
        Bird A -> Root
        Fish A -> Root
        Root A
        ",
        );
    }

    #[test]
    fn test_case() {
        check(
            "
        physeter T -> M159
        M204 T -> M159
        delphin T -> M204
        lipo T -> M204
        M159 T -> M120 M171
        Foo A -> M171
        M120 A -> Root
        M171 A -> Root
        Root A
        M206 T -> M174
        M234 T -> M174
        *M174 T -> Root Common
        Common A>T -> Root
        ",
        )
    }

    #[test]
    fn test_it() {
        check(
            "
        M1 I -> M238
        M2 I -> M238
        M3 I -> M238
        M4 I -> M156
        M5 F -> M240
        M6 I -> M177
        M7 I -> M177
        M8 I -> M177
        M9 I -> M202 M163
        M10 I -> M202
        M11 I -> M226 M218
        M12 I -> M163
        M13 I -> M188
        M14 I -> M188
        M15 I -> M215
        M16 I -> M215
        M17 I -> M242
        M18 I -> M187
        M19 I -> M187
        M20 I -> M187
        M21 I -> M155
        M22 I -> M155
        M23 I -> M187
        M24 I -> M166
        M25 I -> M166
        M26 I -> M166
        M27 I -> M166
        M28 I -> M166 M160
        M29 I -> M166
        M30 I -> M152
        M31 I -> M152 M198
        M32 I -> M189
        M33 I -> M189
        M34 I -> M209
        M35 I -> M209
        M36 I -> M209
        M37 I -> M277 M199
        M38 I -> M179 M277
        M39 X -> M154
        M40 - -> M248
        M41 I -> M277 M241 M180
        M42 - -> M277 M160 M190
        M43 I -> M209
        M44 I -> M252
        M45 I -> M252
        M46 I -> M223 M198
        M47 I -> M252 M148 M198
        M48 I -> M222 M171
        M49 I -> M148
        M50 I -> M212
        M51 I -> M201
        M52 I -> M201
        M53 I -> M201
        M54 I -> M201
        M55 I -> M195
        M56 I -> M153
        M57 I -> M153
        M58 I -> M253 M171
        M59 T -> M185
        M60 T -> M185
        M61 T -> M234
        M62 T -> M234 M180
        M63 T -> M184
        M64 T -> M184
        M65 T -> M184
        M66 T -> M206 M171
        M67 T -> M181
        M68 T -> M181
        M69 T -> M204
        M70 T -> M217
        M71 T -> M204
        M72 T -> M204
        M73 T -> M159
        M74 I -> M196
        M75 I -> M196
        M76 I -> M165
        M77 I -> M165
        M78 I -> M164
        M79 I -> M164
        M80 I -> M151
        M81 I -> M151
        M82 I -> M182
        M83 I -> M182
        M84 I -> M197
        M85 I -> M173
        M86 I -> M173
        M87 I -> M173
        M88 I -> M173
        M89 I -> M173
        M90 I -> M173
        M91 I -> M172
        M92 I -> M172
        M93 I -> M172
        M94 I -> M191
        M95 I -> M192
        M96 I -> M173
        M97 I -> M170
        M98 I -> M170
        M99 I -> M179
        M100 I -> M179
        M101 I -> M179
        M102 I -> M158
        M103 I -> M158
        M104 I -> M194
        M105 I -> M194
        M106 I -> M219
        M107 I -> M157
        M108 I -> M157
        M109 I -> M157
        M110 I -> M167
        M111 I -> M167
        M112 I -> M203
        M113 I -> M169
        M114 I -> M169
        M115 I -> M169
        M116 I -> M169
        M117 I -> M169
        M118 I -> M214
        M119 I -> M214
        M120 I -> M178
        M121 I -> M205
        M122 I -> M183
        M123 I -> M229 M183
        M124 I -> M183
        M125 I -> M220
        M126 I -> M241 M180
        M127 I -> M241
        M128 I -> M150
        M129 I -> M150
        M130 I -> M180
        M131 I -> M287 M154
        M132 I -> M211
        M133 I -> M211
        M134 I -> M211
        M135 I -> M211
        M136 - -> M223
        M137 I -> M211 M248
        M138 I -> M248
        M139 I -> M248
        M140 I -> M193 M212
        M141 I -> M193
        M142 I -> M236
        M143 I -> M236 M224
        M144 I -> M229
        M145 I -> M274 M224
        M146 I -> M220 M171
        M147 I -> M238
        M148 I -> M221
        M149 I -> M250 M218
        M150 I -> M241
        M151 I -> M197
        M152 I -> M189
        M153 I -> M247
        M154 I -> M290
        M155 I -> M187
        M156 I -> M240
        M157 I -> M160 M218
        M158 I -> M194
        M159 T -> M171 M210 Novel
        M160 I -> M161
        M161 I -> M290
        M162 I -> M266
        M163 I -> M227
        M164 I -> M286
        M165 I -> M171
        M166 I -> M189
        M167 I -> M203
        M169 I -> M190
        M170 I -> M192
        M171 I -> M287
        M172 I -> M191
        M173 I -> M192
        *M174 T -> M291 M200 M274 Novel
        M177 I -> M202
        M178 I -> M205
        M179 I -> M199
        M180 I -> M287
        M181 T -> M217
        M182 I -> M197
        M183 I -> M218
        M184 T -> M206
        M185 T -> M234
        M187 I -> M189
        M188 I -> M215
        M189 I -> M149
        M190 I -> M218
        M191 I -> M192
        M192 I -> M237
        M193 I -> M248
        M194 I -> M219
        M195 I -> M247
        M196 I -> M210
        M197 I -> M216
        M198 I -> M287
        M199 I -> M286
        M200 I -> M286
        M201 I -> M195
        M202 I -> M226
        M203 I -> M160
        M204 T -> M159
        M205 I -> M224
        M206 T -> M174
        M209 I -> M149
        M210 I -> M237
        M211 I -> M222 M162
        M212 I -> M221
        M214 I -> M178
        M215 I -> M227
        M216 I -> M287
        M217 T -> M204
        M218 I -> M290
        M219 I -> M286
        M220 I -> M241
        M221 I -> M222
        M222 I -> M285
        M223 I -> M283 M222
        M224 I -> M218
        M226 I -> M227
        M227 I -> M242
        M229 I -> M285
        M234 T -> M174
        M236 I -> M162
        M237 I -> M199
        M238 I -> M156
        M240 I -> M200
        M241 I -> M287
        M242 I -> M283
        M247 I -> M253
        M248 I -> M250
        M250 I -> M221
        M252 I -> M250
        M253 I -> M291
        M266 I -> M285
        M274 I -> M199
        M277 I -> M287
        M283 I -> M266
        M285 I -> M286 M161
        M286 I -> M287
        M287 I -> M290
        M290 I
        M291 I -> M286 M216
        Novel I>T -> M199
        ",
        )
    }
    /*
    assert_push_up("(T,(^A,A)T>A)T;");
    assert_push_up("(T,(A,A)^A)T;");
    assert_push_up("(A,(A,A)^A)T>A;");
    assert_push_up("(T,(A,A)^A)T;");
    assert_push_up("((T,A)T>A,(A,A)^A)T>A;");
    assert_push_up("(S,(A,A)^A)T>A;");
    assert_push_up("(T,(^A,T)T)T;");
    */
}
