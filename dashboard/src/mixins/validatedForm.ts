import Vue, { ComponentOptions } from "vue";
import { ValidationFlags } from "vee-validate/dist/types/types";

const form: ComponentOptions<Vue> = {
  methods: {
    fieldType({ touched, invalid }: ValidationFlags) {
      return touched && invalid ? 'is-danger' : '';
    }
  },
};

export default form;