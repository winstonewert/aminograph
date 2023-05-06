import { reportFunction } from "./report-function"
import type { Report, Action } from "./types"

type IndentPart = 'gap' | 'line' | 'join'
export type SerializedPlanEntry = {
	parent_index?: number
	indent: IndentPart[],
	index: number,
	action: Action,
	alternates: number[]
}

const hasTarget = reportFunction(function (report: Report, index: number, target: number) {
	if (index == target) {
		return true;
	}
	const instruction = report.plan.instructions[index]
	for (const value of instruction.values) {
		if (hasTarget(report, value, target)) {
			return true
		}
	}
	return false
})

function processValues(report: Report, index: number, values: number[], target: number | null, follow_position: number | null) {
	return values.flatMap((value, sourceIndex) => {
		const inner = serializePlan(report, value, target, follow_position);

		if (sourceIndex == values.length - 1) {
			return inner.map((item, itemIndex) => ({
				...item,
				parent_index: item.parent_index !== undefined ? item.parent_index : index,
				indent: [itemIndex == 0 ? 'join' as const : 'gap' as const, ...item.indent]
			}))
		} else {
			return inner.map((item, itemIndex) => ({
				...item,
				parent_index: item.parent_index !== undefined ? item.parent_index : index,
				indent: [itemIndex == 0 ? 'join' as const : 'line' as const, ...item.indent]
			}))
		}
	})
}


export const serializePlan = reportFunction(function serializePlanImpl(report: Report, index: number, target: number | null, follow_position: number | null): SerializedPlanEntry[] {
	const instruction = report.plan.instructions[index]

	if (instruction.action.type == 'Combine') {

		let followIndex = 0;
		if (target !== null) {
			if (!hasTarget(report, instruction.values[0], target)) {
				for (let current = 1; current < instruction.values.length; current += 2) {
					let module_value = instruction.values[current]
					let leftovers_value = instruction.values[current + 1]
					if (hasTarget(report, module_value, target)
						|| hasTarget(report, leftovers_value, target)
					) {
						followIndex = current;
						break;
					}
				}
			}
		}
		if (follow_position !== null) {
			for (let current = 1; current < instruction.values.length; current += 2) {
				if (report.positions[follow_position].primary[instruction.values[current]] == '1') {
					followIndex = current;
					break;
				}
			}
		}

		let alternates = [instruction.values[0]]
		for (let current = 1; current < instruction.values.length; current += 2) {
			alternates.push(instruction.values[current])
		}

		let inner = []
		if (followIndex == 0) {
			inner = [instruction.values[followIndex]]
		} else {
			inner = [
				instruction.values[followIndex],
				instruction.values[followIndex + 1]
			]
		}
		return [
			{
				index,
				action: instruction.action,
				indent: [],
				alternates
			},
			...processValues(report, index, inner, target, follow_position)
		]
	}

	return [
		{
			index,
			action: instruction.action,
			indent: [],
			alternates: []
		},
		...processValues(report, index, instruction.values, target, follow_position)
	]
})
