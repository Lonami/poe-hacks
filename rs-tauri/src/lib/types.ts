import type {
    BLOCK_CLICK_VARIABLES,
    BLOCK_SCROLL_VARIABLES,
    BLOCK_STAT_CONDITIONS,
    BLOCK_STAT_VARIABLES,
} from "./constants"

export type Health = {
    hp: number,
    maxHp: number,
    unreservedHp: number,
    es: number,
    maxEs: number,
}

export type Mana = {
    mana: number,
    maxMana: number,
    unreservedMana: number,
}

export type BlockDefinition =
    // Events
    | {
        kind: 'stat',
        variable: (typeof BLOCK_STAT_VARIABLES)[number],
        condition: (typeof BLOCK_STAT_CONDITIONS)[number],
        value: string,
    }
    | {
        kind: 'key',
        value: string,
    }
    | {
        kind: 'mouse',
        variable: (typeof BLOCK_CLICK_VARIABLES)[number],
    }
    // Actions
    | {
        kind: 'press',
        value: string,
    }
    | {
        kind: 'type',
        value: string,
    }
    | {
        kind: 'disconnect',
    }
    | {
        kind: 'click',
        variable: (typeof BLOCK_CLICK_VARIABLES)[number],
    }
    | {
        kind: 'scroll',
        variable: (typeof BLOCK_SCROLL_VARIABLES)[number],
    }
    // Timing
    | {
        kind: 'cooldown',
        value: string,
    }
    | {
        kind: 'delay',
        value: string,
    }

export type RuleDefinition = {
    name: string
    blocks: BlockDefinition[]
}

export type ProfileDefinition = {
    name: string
    rules: RuleDefinition[]
}
