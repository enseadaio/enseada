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
    name: 'sections.packages',
    children: [
      { name: 'sections.containers', to: { name: 'containers' }, permission: { object: 'oci_repos', action: 'read' } },
      { name: 'sections.maven', to: { name: 'maven' }, permission: { object: 'maven_repos', action: 'read' } },
    ],
  },
  {
    name: 'sections.security',
    children: [
      { name: 'sections.users', to: { name: 'users' }, permission: { object: 'users', action: 'read' } },
      { name: 'sections.roles', to: { name: 'roles' }, permission: { object: 'roles', action: 'read' } },
      { name: 'sections.clients', to: '#', permission: { object: 'oauth:clients', action: 'read' } },
      { name: 'sections.pats', to: { name: 'pats' }, permission: { object: 'pats', action: 'read' } },
    ],
  },
];

export default sections;