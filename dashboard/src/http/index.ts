import { HttpClient } from './HttpClient'
import { createService } from './Service'

export { createService };

const VueHttp = {
  install(vm, options) {
    vm.prototype.$http = new HttpClient(options);
    vm.prototype.$users = createService('/api/v1beta1/users', {
      permissions: (id) => createService(`/api/v1beta1/users/${id}/permissions`),
      roles: (id) => createService(`/api/v1beta1/users/${id}/roles`),
    })
    vm.prototype.$roles = createService('/api/v1beta1/roles', {
      permissions: (id) => createService(`/api/v1beta1/roles/${id}/permissions`)
    })
    vm.prototype.$containers = createService('/api/oci/v1beta1/repositories')
    vm.prototype.$maven = createService('/api/maven/v1beta1/repositories', {
      files: (id) => createService(`/api/maven/v1beta1/repositories/${id}/files`)
    })
    vm.prototype.$pats = createService('/api/oauth/v1beta1/pats')
  },
};

export default VueHttp;
export * from './HttpClient';
export * from './Method';
export * from './Page';

