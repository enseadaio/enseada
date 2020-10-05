import Vue from 'vue';

export function truncate(text: string, length: number): string {
  if (!text) return '';
  const val = text.toString();
  console.log(val, 'is', val.length, 'charactes, we want', length)
  if (val.length <= length) return val;
  return val.slice(0, length) + '...';
}

Vue.filter('truncate', truncate);

