import { fly, slide } from "svelte/transition";

export const flyIn: typeof fly = (node, props) => {
    const defaultFlight = fly(node);
    const duration = props?.duration ?? defaultFlight.duration ?? 1;
    const delay = props?.delay ?? defaultFlight.delay ?? 0;

    const flight = fly(node, props);
    const totalDuration = duration + delay;
    const start = delay / totalDuration;
    const scale = 1 / (1 - start);
    return {
        duration: totalDuration,
        css: (t: number, u: number) => {
            return t > start
                ? flight.css!((t - start) * scale, u * scale)
                : "display:none";
        },
    };
};
