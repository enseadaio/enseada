import Vue from 'vue';
import { extend, ValidationObserver, ValidationProvider } from "vee-validate";
import { confirmed, required } from 'vee-validate/dist/rules';

Vue.component('ValidationProvider', ValidationProvider);
Vue.component('ValidationObserver', ValidationObserver);

extend('required', {
  ...required,
  message: (field) => (`${field} is required`),
})

extend('confirmed', {
  ...confirmed,
  message: (field, { target }) => (`${field} does not match ${target}`),
})