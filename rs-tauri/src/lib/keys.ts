export const mkGetObjectKey = <T extends object = never>() => {
    let curId = 0;
    const ids = new WeakMap<T, number>();
    return (obj: T): number => {
        const id = ids.get(obj);
        if (id) {
            return id;
        }
        curId += 1;
        const newId = curId;
        ids.set(obj, newId);
        return newId;
    };
}
