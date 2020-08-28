import minimatch, { IOptions } from 'minimatch';
import { identity } from "./utils";
import { Permission, User } from "./types";

const opts: IOptions = {
  nobrace: true,
  nonegate: true,
  noglobstar: true,
  nocase: true,
  noext: true,
};
const match: (obj: string, act: string) => (perm: Permission) => boolean = (obj, act) => ({ object, action }) => minimatch(obj, object, opts) && minimatch(act, action, opts);

export function check(obj: string, act: string, permissions: Permission[] = this.permissions, user: User = this.currentUser) {
  if (!permissions || !user) return false;
  if (user.username === 'root') return true;
  const res = permissions.map(match(obj, act)).some(identity)
  console.debug('Checking object', obj, 'action', act, 'result', res);
  return res
}