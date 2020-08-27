<template>
  <b-table :data="page.items"
           :total="page.count"
           :per-page="page.limit"
           @page-change="$emit('page-change', $event)"
           backend-pagination
           paginated>
    <b-table-column field="subject" label="Subject" v-slot="{ row }">
      {{ row.subject }}
    </b-table-column>
    <b-table-column field="object" label="Object" v-slot="{ row }">
      {{ row.object }}
    </b-table-column>
    <b-table-column field="action" label="Action" v-slot="{ row }">
      {{ row.action }}
    </b-table-column>
    <b-table-column v-slot="{ row }">
      <div class="buttons has-addons">
        <b-button type="is-danger"
                  @click="$emit('remove', row)">Remove
        </b-button>
      </div>
    </b-table-column>
  </b-table>
</template>

<script>
export default {
  name: 'PermissionsTable',
  props: {
    page: Object
  }
}
</script>

<style scoped>

</style>