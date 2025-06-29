import { invoke } from "@tauri-apps/api/core";

export const ssr = false;

export const load = async () => {
    const profiles = await invoke("get_profiles", undefined);

    return {
        profiles
    };
};
