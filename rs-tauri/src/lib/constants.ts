export const ROUTES = {
    "/": "Status",
    "/configure-rules": "Configure rules",
    "/manage-profiles": "Manage profiles",
} as const

export const BLOCK_WHEN_VARIABLES = ['life', 'mana', 'es'] as const
export const BLOCK_WHEN_CONDITIONS = ['<', '>'] as const
export const BLOCK_CLICK_VARIABLES = ['left', 'middle', 'right'] as const
export const BLOCK_SCROLL_VARIABLES = ['up', 'down'] as const
