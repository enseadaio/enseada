import { pageToOffset } from "../http";
import AddPermissionModal from '../components/AddPermissionModal';
import Vue, { ComponentOptions } from "vue";
import { svcGetter } from "../utils";

interface FactoryParams<T> {
  service: string,
}

function factory<T>({ service }: FactoryParams<T>): ComponentOptions<Vue> {
  const svc = svcGetter(`$${service}`);
  return {
    data() {
      return {
        loading: false,
        permissionsPage: {
          limit: 25
        }
      }
    },
    computed: {
      permissionSvc() {
        return svc(this).association('permissions', this.id)
      },
    },
    methods: {
      async fetchPermissions(offset = 0) {
        this.loading = true
        this.permissionsPage = await this.permissionSvc.list({ offset, limit: this.permissionsPage.limit })
        this.loading = false
      },
      openPermissionModal() {
        this.$buefy.modal.open({
          parent: this,
          component: AddPermissionModal,
          hasModalCard: true,
          trapFocus: true,
          props: {
            includeSubject: false
          },
          events: {
            'ok': this.addPermission.bind(this)
          }
        })
      },
      async addPermission(perm) {
        try {
          await this.permissionSvc.create(perm)
          this.$buefy.notification.open({
            message: `Permission added`,
            type: 'is-success',
            position: 'is-bottom-right',
            duration: 2000
          })
          return this.reloadPermissions()
        } catch (err) {
          return this.$emit('error', err)
        }
      },
      reloadPermissions() {
        return this.fetchPermissions(this.permissionsPage.offset).catch((err) => this.$emit('error', err))
      },
      onPermissionsPageChange(page) {
        return this.fetchPermissions(pageToOffset(page, this.permissionsPage.limit)).catch((err) => this.$emit('error', err))
      },
      async removePermission(perm) {
        try {
          await this.permissionSvc.remove(undefined, perm)
          this.$buefy.notification.open({
            message: `Permission removed`,
            type: 'is-warning',
            position: 'is-bottom-right',
            duration: 2000
          })

          return this.reloadPermissions()
        } catch (err) {
          return this.$emit('error', err)
        }
      },
    },
    created() {
      return this.fetchPermissions().catch((err) => this.$emit('error', err))
    }
  }
}

export default factory;