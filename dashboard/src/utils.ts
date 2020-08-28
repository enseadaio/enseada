import { Service } from "./http/Service";
import { IdMapFn } from "./types";

export const svcGetter = <T>(svcName: string) => (self: { [svcName]: Service<T> }): Service<T> => self[svcName];
export const mapId: IdMapFn<{ id: string }> = ({ id }) => id;
export const identity: <T>(a: T) => T = (a) => a;