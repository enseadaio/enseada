import Vue, { ComponentOptions } from 'vue';
import { IdMapFn } from "../types";
import { svcGetter } from "../utils";

interface CreateFormData<T> {
  loading: boolean,
  model: any,
  created: T,
}

interface FactoryParams<T> {
  name: string,
  service: string,
  mapId: IdMapFn<T>,
}

function factory<T>({ name, service, mapId }: FactoryParams<T>): ComponentOptions<Vue> {
  const svc = svcGetter(`$${service}`);
  return {
    data(): CreateFormData<T> {
      return {
        loading: false,
        model: {},
        created: null,
      }
    },
    methods: {
      postSubmit() {
        this.$buefy.notification.open({
          message: `Created ${name} ${mapId(this.created)}`,
          type: 'is-success',
          position: 'is-bottom-right',
          duration: 2000
        });
        return this.$router.push({ name: service })
      },
      async submit() {
        try {
          this.loading = true
          this.created = await svc(this).create(this.model)
          return this.postSubmit();
        } catch (err) {
          this.$emit('error', err)
        } finally {
          this.loading = false
        }
      },
      mapId,
    }
  }
}

export default factory;