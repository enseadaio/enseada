import { Permission } from "./types";

export class ForbiddenError extends Error {
  constructor(readonly permission: Permission) {
    super(`Forbidden action: ${permission.object} ${permission.action}`);
  }
}