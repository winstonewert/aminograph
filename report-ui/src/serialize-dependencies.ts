import type { Report } from "./types"

type IndentPart = 'gap' | 'line' | 'join'
export type SerializedNodeEntry = {
	indent: IndentPart[],
	action: {
		type: "Node",
		value: string
	} | {
		type: "Sequence",
		value: number
	},
	alternates: []
}

const serialCache = new Map()

export function serializeDependenciesImpl(report: Report, node_id: string): SerializedNodeEntry[] {
	const node = report.nodes[node_id]

	return [
		{
			action: {
				type: 'Node',
				value: node_id,
			},
			indent: [],
			alternates: []
		},
		...node.edges.flatMap((value, sourceIndex) => {
			const inner = serializeDependencies(report, value);

			if (sourceIndex == node.edges.length - 1) {
				return inner.map((item, itemIndex) => ({
					...item,
					indent: [itemIndex == 0 ? 'join' as const : 'gap' as const, ...item.indent]
				}))
			} else {
				return inner.map((item, itemIndex) => ({
					...item,
					indent: [itemIndex == 0 ? 'join' as const : 'line' as const, ...item.indent]
				}))
			}
		})
	]
}


export function serializeSequenceDependencies(report: Report, index: number): SerializedNodeEntry[] {
	console.log(report.sequences[index], index)
	return [
		{
			action: {
				type: 'Sequence',
				value: index,
			},
			indent: [],
			alternates: []
		},
		...report.sequences[index].edges.flatMap((value, sourceIndex) => {
			const inner = serializeDependencies(report, value);

			if (sourceIndex == report.sequences[index].edges.length - 1) {
				return inner.map((item, itemIndex) => ({
					...item,
					indent: [itemIndex == 0 ? 'join' as const : 'gap' as const, ...item.indent]
				}))
			} else {
				return inner.map((item, itemIndex) => ({
					...item,
					indent: [itemIndex == 0 ? 'join' as const : 'line' as const, ...item.indent]
				}))
			}
		})
	]
}

export function serializeDependencies(report: Report, node_id: string): SerializedNodeEntry[] {

	let serialCacheForReport = serialCache.get(report)
	if (serialCacheForReport === undefined) {
		serialCacheForReport = new Map()
		serialCache.set(report, serialCacheForReport)
	}

	let value = serialCacheForReport.get(node_id)
	if (value) {
		return value
	} else {
		value = serializeDependenciesImpl(report, node_id)
		serialCacheForReport.set(node_id, value)
		return value
	}
}