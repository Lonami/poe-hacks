import type { ProfileDefinition } from "./types"

export const G = $state<{
    profiles: ProfileDefinition[]
}>({
    profiles: []
})
