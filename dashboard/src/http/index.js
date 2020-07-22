import { HttpClient } from './HttpClient'
import { createService } from './Service'

const VueHttp = {
  install(vm, options) {
    vm.prototype.$http = new HttpClient(options);
    vm.prototype.$users = createService('/api/v1beta1/users')
    vm.prototype.$containers = createService('/api/oci/v1beta1/repositories')
  },
};

export default VueHttp;
export * from './HttpClient';
export * from './Method';
export * from './Page';

