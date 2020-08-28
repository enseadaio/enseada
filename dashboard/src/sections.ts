import { Location } from "vue-router";
import { Permission } from "./types/roles";

export interface SectionGroup {
  name: string;
  children?: Section[];
}

export interface Section {
  name: string;
  to?: Location | string;
  permission?: Permission;
}

const sections: SectionGroup[] = [
  {
    name: 'Packages',
    children: [
      { name: 'Containers', to: { name: 'containers' }, permission: { object: 'oci_repos', action: 'read' } },
      { name: 'Maven', to: { name: 'maven' }, permission: { object: 'maven_repos', action: 'read' } },
    ],
  },
  {
    name: 'Security',
    children: [
      { name: 'Users', to: { name: 'users' }, permission: { object: 'users', action: 'read' } },
      { name: 'Roles', to: { name: 'roles' }, permission: { object: 'roles', action: 'read' } },
      { name: 'OAuth Clients', to: '#', permission: { object: 'oauth:clients', action: 'read' } },
      { name: 'Access Tokens', to: { name: 'pats' }, permission: { object: 'pats', action: 'read' } },
    ],
  },
];

export default sections;