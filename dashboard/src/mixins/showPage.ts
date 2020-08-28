import Vue, { ComponentOptions } from 'vue';
import { svcGetter } from "../utils";
import { Permission } from "../types";
import { ForbiddenError } from "../errors";
import { mapGetters } from "vuex";
import { check } from "../auth";

interface ShowData<T> {
  loading: boolean,
  model: T | null,
}

interface FactoryParams<T> {
  name: string,
  service: string,
  permission?: Permission,
}

function factory<T>({ name, service, permission }: FactoryParams<T>): ComponentOptions<Vue> {
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
    computed: {
      ...mapGetters(['currentUser', 'permissions']),
    },
    methods: {
      check,
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
      if (permission && !this.check(`${permission.object}:${this.id}`, permission.action)) {
        return this.$emit('error', new ForbiddenError(permission));
      }
      return this.fetch().catch((err) => this.$emit('error', err))
    },
  }
}

export default factory;