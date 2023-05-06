import REPORT from '../report.json'
import METADATA from "../metadata.json"
import dagre from "dagre"
import type {Report, Metadata} from "./types"
import mem from "mem"

export default REPORT as Report
export const ReportMetadata = METADATA as Metadata;


export const nodeDepends = mem((source: string, destination: string) => {
	if (source == destination) {
		return true
	} else {
		for (const parent of REPORT[source].parents) {
			if (nodeDepends(parent, destination)) {
				return true;
			}
		}
		return false;
	}

}, {
	cacheKey: ([source, destination]) => `${source}@${destination}`
})

export const topologicalOrder = mem(() => {
	const order = []
	while (true) {
		let changed = false;
		for (const node_id in REPORT) {
			if (!order.includes(node_id) && REPORT[node_id].parents.every(parent => order.includes(parent))) {
				order.push(node_id)
				changed = true;
			}
		}
		if (!changed) {
			return order
		}
	}
})

export const reverseTopologicalOrder = mem(() => {
	const order = [...topologicalOrder()]
	order.reverse()
	return order
})

export const allDependencies = mem((node_id) => {
	return reverseTopologicalOrder().filter(other => nodeDepends(node_id, other))
})

export const interestingPositions = mem((node_id) => {
	const dependencies = allDependencies(node_id)
	return Array.from(REPORT[node_id].amino_acids).map((_, index) => index).filter(index => dependencies.some(dependency => REPORT[dependency].amino_acids[index] != '-'))
})

export const node_position_inherited = mem((node_id: string, position: number) => {
	let amino_acid = '-'
	let height = 0;
	let source = null;

	for (const parent of REPORT[node_id].parents) {
		const parent_amino_acid = REPORT[parent].amino_acids[position]
		const parent_height = node_position_height(parent, position)
		if (parent_height > height) {
			source = parent;
			height = parent_height;
			amino_acid = REPORT[parent].amino_acids[position]
		} else if (parent_height == height) {
			if (amino_acid === parent_amino_acid) {
				// carry on
			} else {
				amino_acid = 'X'
			}
		}
	}

	return {amino_acid, height, source}

}, {
	cacheKey: ([node_id, position]) => `${node_id}@${position}`
})

export const node_position_height = mem((node_id: string, position: number) => {
	const inherited = node_position_inherited(node_id, position)
	const amino_acid = REPORT[node_id].amino_acids[position]
	if (amino_acid === inherited.amino_acid) {
		return inherited.height
	} else {
		return inherited.height + 1
	}
}, {
	cacheKey: ([node_id, position]) => `${node_id}@${position}`
})

export const changeMode = mem((node_id: string, position: number) => {
	const inherited = node_position_inherited(node_id, position)
	const amino_acid = REPORT[node_id].amino_acids[position]
	if (amino_acid === inherited.amino_acid) {
		return 'none'
	} else if (amino_acid == '-') {
		return 'delete'
	} else if (inherited.amino_acid == '-') {
		return 'insert'
	} else {
		return 'change'
	}
}, {
	cacheKey: ([node_id, position]) => `${node_id}@${position}`
})

export const reportLayers = mem((focus: string | undefined) => {

	const include = (node_id: string) => {
		return focus == undefined || nodeDepends(focus, node_id) || nodeDepends(node_id, focus)
	}
	const graph = new dagre.graphlib.Graph()
	graph.setGraph({ranksep: 200})
	graph.setDefaultEdgeLabel(() => ({}))
	for (const [node_id, node] of Object.entries(REPORT)) {
		if (include(node_id)) {
			if (node.kind == "leaf") {	
				graph.setNode(node_id, {label: node_id, width: 200, height: 50})
			} else {
				graph.setNode(node_id, {label: node_id, width: 40, height: 40})
			}
		}
	}
	for (const [node_id, node] of Object.entries(REPORT)) {
		if (include(node_id)) {
			for (const parent_id of node.parents) {
				if (include(parent_id)) {
					graph.setEdge(parent_id, node_id)
				}
			}
		}
	}
	dagre.layout(graph)
	return graph
})


export const graphForIndex = mem((index: number) => {

	const include = (node_id: string) => {
		return true;
	}
	const graph = new dagre.graphlib.Graph()
	graph.setGraph({ranksep: 200})
	graph.setDefaultEdgeLabel(() => ({}))
	for (const [node_id, node] of Object.entries(REPORT)) {
		if (include(node_id)) {
			if (node.kind == "leaf") {	
				graph.setNode(node_id, {label: node_id, width: 200, height: 50})
			} else {
				graph.setNode(node_id, {label: node_id, width: 40, height: 40})
			}
		}
	}
	for (const [node_id, node] of Object.entries(REPORT)) {
		if (include(node_id)) {
			let inherited = node_position_inherited(node_id, index)
			if (inherited.source !== null) {
				if (include(inherited.source)) {
					graph.setEdge(inherited.source, node_id)
				}
			}
			
		}
	}
	dagre.layout(graph)
	return graph
})

export const sequencePositions = mem(() => {
	return Array.from(Object.values(REPORT)[0].amino_acids).map((_, index) => {
		const counts: {[key: string]: number} = {}
		for (const [node_id, node] of Object.entries(REPORT)) {
			if (node.kind == "leaf") {
				counts[node.amino_acids[index]] = (counts[node.amino_acids[index]] || 0) + 1
			}
		}
		const logo = Object.entries(counts).map(([amino_acid, count]) => ({amino_acid, count}))
		logo.sort((lhs, rhs) => rhs.count - lhs.count)
		return {
			logo
		}
	})
})
