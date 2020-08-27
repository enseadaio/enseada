import Vue, { ComponentOptions } from 'vue';
import { svcGetter } from "../utils";

interface ShowData<T> {
  loading: boolean,
  model: T | null,
}

interface FactoryParams<T> {
  name: string,
  service: string,
}

function factory<T>({ name, service }: FactoryParams<T>): ComponentOptions<Vue> {
  const svc = svcGetter(`$${service}`);
  return {
    props: {
      id: String,
    },
    data(): ShowData<T> {
      return {
        loading: false,
        model: null,
      };
    },
    methods: {
      async fetch() {
        this.loading = true
        this.model = await svc(this).get(this.id)
        this.loading = false
      },
      async remove() {
        try {
          await svc(this).remove(this.id);
          this.$buefy.notification.open({
            message: `Deleted ${name} ${this.id}`,
            type: 'is-warning',
            position: 'is-bottom-right',
            duration: 2000
          })

          return this.$router.push({ name: service })
        } catch (err) {
          return this.$emit('error', err)
        }
      },
    },
    async created() {
      return this.fetch().catch((err) => this.$emit('error', err))
    },
  }
}

export default factory;