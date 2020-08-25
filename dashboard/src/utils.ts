import { Service } from "./http/Service";

export const svcGetter = <T>(svcName: string) => (self: { [svcName]: Service<T> }): Service<T> => self[svcName];
