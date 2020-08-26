import { HttpClient } from './HttpClient'
import { createService } from './Service'

export { createService };

const VueHttp = {
  install(vm, options) {
    vm.prototype.$http = new HttpClient(options);
    vm.prototype.$users = createService('/api/v1beta1/users')
    vm.prototype.$containers = createService('/api/oci/v1beta1/repositories')
    vm.prototype.$pats = createService('/api/oauth/v1beta1/pats')
  },
};

export default VueHttp;
export * from './HttpClient';
export * from './Method';
export * from './Page';

