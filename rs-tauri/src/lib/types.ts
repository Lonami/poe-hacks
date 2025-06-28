import type {
    BLOCK_CLICK_VARIABLES,
    BLOCK_SCROLL_VARIABLES,
    BLOCK_WHEN_CONDITIONS,
    BLOCK_WHEN_VARIABLES,
} from "./constants"

export type Health = {
    hp: number,
    max_hp: number,
    unreserved_hp: number,
    es: number,
    max_es: number,
}

export type Mana = {
    mana: number,
    max_mana: number,
    unreserved_mana: number,
}

export type BlockDefinition = { value: string } & (
    | {
        kind: 'when',
        variable: (typeof BLOCK_WHEN_VARIABLES)[number],
        condition: (typeof BLOCK_WHEN_CONDITIONS)[number],
    }
    | { kind: 'press' }
    | { kind: 'type' }
    | {
        kind: 'click',
        variable: (typeof BLOCK_CLICK_VARIABLES)[number]
    }
    | {
        kind: 'scroll',
        variable: (typeof BLOCK_SCROLL_VARIABLES)[number]
    }
    | { kind: 'cooldown' }
)

export type RuleDefinition = {
    id: number
    name: string
    blocks: BlockDefinition[]
}
