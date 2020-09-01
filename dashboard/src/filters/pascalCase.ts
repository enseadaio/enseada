import Vue from 'vue';
import { titleCase } from "./titleCase";

export function pascalCase(value: Object): string {
  if (!value) return '';
  const val = value.toString();
  return val
    .split(' ')
    .map(titleCase)
    .map((s) => s.replace('-', ' ')
      .replace('_', ' ')
      .split(' ')
      .map(titleCase)
      .join('')
    )
    .join(' ');
}

Vue.filter('pascalCase', pascalCase);

