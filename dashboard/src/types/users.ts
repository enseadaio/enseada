import { Permission } from "./roles";

export interface User {
  username: string;
  enabled: boolean;
}

export interface Capabilities {
  permissions: Permission[],
  roles: string[],
}