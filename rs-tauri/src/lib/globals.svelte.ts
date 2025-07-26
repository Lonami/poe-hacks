import type { ProfileDefinition } from "./types"

export const G = $state<{
    profiles: ProfileDefinition[]
    profilesSynced: boolean
}>({
    profiles: [],
    profilesSynced: true
})
