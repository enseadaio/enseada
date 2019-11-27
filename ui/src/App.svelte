<script>
    import Router, {link} from 'svelte-spa-router';
    import routes from './routes';

	export let name;
</script>

<style>
	h1 {
		color: purple;
	}
</style>

<h1>Hello {name}!</h1>
<a href="/" use:link>Home</a>
<a href="/test" use:link>Test</a>

<Router {routes}/>
