import 'phoenix_html'
import App from './App.svelte';
import '../css/app.css';

const app = new App({
  target: document.body,
  props: {
    name: 'world'
  }
});

window.app = app;

export default app;
