import { Location } from "vue-router";

export interface Section {
  name: string;
  to?: Location | string;
  children?: Section[];
}

const sections: Section[] = [
  // <b-navbar-dropdown label="Packages" hoverable class="is-hidden-desktop">
  //   <b-navbar-item tag="router-link" :to="{ name: 'containers'}">
  // Containers
  // </b-navbar-item>
  // <b-navbar-item tag="router-link" :to="{ name: 'maven'}">
  // Maven
  // </b-navbar-item>
  // </b-navbar-dropdown>
  {
    name: 'Packages',
    children: [
      { name: 'Containers', to: {name: 'containers'}},
      { name: 'Maven', to: {name: 'maven'}},
    ],
  },
  {
    name: 'Security',
    children: [
      { name: 'Users', to: {name: 'users'} },
      { name: 'OAuth Clients', to: '#' },
      { name: 'Access Tokens', to: '#' },
    ],
  },
];

export default sections;