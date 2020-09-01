import { LocaleMessageObject } from "vue-i18n";

const messages: LocaleMessageObject = {
  home: 'home',
  documentation: 'documentation',
  about: 'about',
  account: 'account',
  logout: 'logout',
  apiDocs: 'API docs',
  issueReport: 'report an issue',
  sections: {
    // Packages
    packages: 'packages',
    containers: 'containers',
    maven: 'maven',

    // Security
    security: 'security',
    users: 'users',
    roles: 'roles',
    clients: 'oauth clients',
    pats: 'access tokens',
  },
}

export default messages;