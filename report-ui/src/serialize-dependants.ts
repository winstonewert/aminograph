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
	alternates: number[]
}

const serialCache = new Map()

export function serializeDependantsImpl(report: Report, node_id: string): SerializedNodeEntry[] {
	const incoming = Object.entries(report.nodes).map(([node_id, node]) => ({ ...node, node_id })).filter(node => node.edges.indexOf(node_id) != -1)

	return [
		{
			action: {
				type: 'Node',
				value: node_id,
			},
			indent: [],
			alternates: []
		},
		...report.sequences.map((sequence, index) => ({ ...sequence, index })).filter(sequence => sequence.edges.indexOf(node_id) != -1).map(
			sequence => ({
				action: {
					type: 'Sequence' as const,
					value: sequence.index
				},
				indent: ['join' as const],
				alternates: []
			})
		),
		...incoming.flatMap((node, sourceIndex) => {
			const inner = serializeDependants(report, node.node_id);

			if (sourceIndex == incoming.length - 1) {
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

export function serializeDependants(report: Report, node_id: string): SerializedNodeEntry[] {

	let serialCacheForReport = serialCache.get(report)
	if (serialCacheForReport === undefined) {
		serialCacheForReport = new Map()
		serialCache.set(report, serialCacheForReport)
	}

	let value = serialCacheForReport.get(node_id)
	if (value) {
		return value
	} else {
		value = serializeDependantsImpl(report, node_id)
		serialCacheForReport.set(node_id, value)
		return value
	}
}