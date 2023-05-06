import type { Report, TreeEntry } from "./types"
import { serializePlan } from "./serialize-plan"

export function findChanges(report: Report, target: TreeEntry) {
	return report.positions.flatMap((position, index) =>
		serializePlan(report, report.plan.target, null, index).filter(entry =>
			entry.action.value == target.action.value
			&& entry.parent_index
			&& position.amino_acids[entry.parent_index] != position.amino_acids[entry.index]
			&& position.amino_acids[entry.index] != '-'
		).map(entry => ({
			index: index,
			parent_amino_acid: position.amino_acids[entry.parent_index],
			amino_acid: position.amino_acids[entry.index]
		}))

	)
}


export function findChangesForPosition(report: Report, index: number) {

	let position = report.positions[index]
	return serializePlan(report, report.plan.target, null, index).filter(entry =>
		entry.parent_index
		&& position.amino_acids[entry.parent_index] != position.amino_acids[entry.index]
		&& position.amino_acids[entry.index] != '-'
	).map(entry => ({
		action: entry.action,
		parent_amino_acid: position.amino_acids[entry.parent_index],
		amino_acid: position.amino_acids[entry.index]
	}))
}