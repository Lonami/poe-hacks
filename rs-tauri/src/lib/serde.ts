import { BLOCK_CLICK_VARIABLES, BLOCK_SCROLL_VARIABLES, BLOCK_WHEN_CONDITIONS, BLOCK_WHEN_VARIABLES } from "./constants";
import type { BlockDefinition } from "./types";

export const serializeBlockDefinition = (block: BlockDefinition): string => {
    switch (block.kind) {
        case "when":
            return `${block.kind} ${block.variable} ${block.condition} ${block.value}`
        case "press":
            return `${block.kind} ${block.value}`
        case "type":
            return `${block.kind} ${block.value}`
        case "click":
            return `${block.kind} ${block.variable} ${block.value}`
        case "scroll":
            return `${block.kind} ${block.variable} ${block.value}`
        case "cooldown":
            return `${block.kind} ${block.value}`
    }
}

const makeEnumParser = <T>(legalValues: readonly T[]) => (text: unknown): T => {
    if (!legalValues.includes(text as T)) {
        throw new Error(`text is not a legal value: ${JSON.stringify(text)} not in ${JSON.stringify(legalValues)}`)
    }
    return text as T
}

const parseWhenVariable = makeEnumParser(BLOCK_WHEN_VARIABLES)
const parseWhenCondition = makeEnumParser(BLOCK_WHEN_CONDITIONS)
const parseClickVariable = makeEnumParser(BLOCK_CLICK_VARIABLES)
const parseScrollVariable = makeEnumParser(BLOCK_SCROLL_VARIABLES)
const parseString = (text: unknown): string => {
    if (typeof text !== 'string') {
        throw new Error(`text is not a legal value: ${JSON.stringify(text)}`)
    }
    return text
}

export const deserializeBlockDefinition = (text: string): BlockDefinition => {
    const parts = text.split(' ');
    const kind = parts[0] as BlockDefinition['kind'];
    switch (kind) {
        case "when":
            return { kind, variable: parseWhenVariable(parts[1]), condition: parseWhenCondition(parts[2]), value: parseString(parts[3]) }
        case "press":
            return { kind, value: parseString(parts[1]) }
        case "type":
            return { kind, value: text.slice(kind.length + 1) }
        case "click":
            return { kind, variable: parseClickVariable(parts[1]), value: parseString(parts[2]) }
        case "scroll":
            return { kind, variable: parseScrollVariable(parts[1]), value: parseString(parts[2]) }
        case "cooldown":
            return { kind, value: parseString(parts[1]) }
        default:
            const assertNever: never = kind
            throw new Error(`text is not a legal block kind: ${JSON.stringify(assertNever)}`)

    }
}
