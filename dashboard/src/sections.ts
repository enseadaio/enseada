import { Location } from "vue-router";

export interface Section {
  name: string;
  to?: Location | string;
  children?: Section[];
}

const sections: Section[] = [
  {
    name: 'Packages',
    children: [
      { name: 'Containers', to: { name: 'containers' } },
      { name: 'Maven', to: { name: 'maven' } },
    ],
  },
  {
    name: 'Security',
    children: [
      { name: 'Users', to: { name: 'users' } },
      { name: 'OAuth Clients', to: '#' },
      { name: 'Access Tokens', to: { name: 'pats' } },
    ],
  },
];

export default sections;