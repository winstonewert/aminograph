import type { Report } from "./types"


export function reportFunction<A extends unknown[], Z>(
	impl: (report: Report, ...a: A) => Z
): (report: Report, ...a: A) => Z {

	const bigCache = new Map()

	return (report, ...a) => {

		let reportCache = bigCache.get(report)
		if (reportCache === undefined) {
			reportCache = new Map()
			bigCache.set(report, reportCache)
		}

		const key = a.map(x => x + '').join('-')

		let value = reportCache.get(key)
		if (value) {
			return value
		} else {
			value = impl(report, ...a)
			reportCache.set(key, value)
			return value
		}
	}
}