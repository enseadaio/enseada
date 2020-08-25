import Vue, { ComponentOptions } from 'vue';
import { IdMapFn } from "../types";
import { svcGetter } from "../utils";

interface CreateFormData<T> {
  loading: boolean,
  model: any,
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
      }
    },
    methods: {
      async submit() {
        try {
          this.loading = true
          const created = await svc(this).create(this.model)
          this.$buefy.notification.open({
            message: `Created ${name} ${mapId(created)}`,
            type: 'is-success',
            position: 'is-bottom-right',
            duration: 10000
          })
          await this.$router.push({ name: service })
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