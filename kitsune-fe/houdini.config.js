/// <references types="houdini-svelte">

/** @type {import('houdini').ConfigFile} */
const config = {
	watchSchema: {
		url: 'http://localhost:5000/graphql'
	},
	plugins: {
		'houdini-svelte': {}
	}
};

export default config;
