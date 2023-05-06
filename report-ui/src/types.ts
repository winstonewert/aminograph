export type Node = {
	kind: "leaf" | "other" | "root",
	sequence_id: string | null;
	amino_acids: string;
	parents: string[]
}

export type Report = {
	[k: string]: Node
}

export const AMINO_ACIDS = 'ARNDCQEGHILKMFPSTWYV'

export type Metadata = {
	sequences: {
		[k: string]: {
			label: string;
			sublabel: string | null;
			image: string | null
		}
	};
}