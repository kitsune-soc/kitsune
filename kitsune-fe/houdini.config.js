/// <references types="houdini-svelte">

/** @type {import('houdini').ConfigFile} */
const config = {
	watchSchema: {
		url: 'http://localhost:5000/graphql'
	},
	plugins: {
		'houdini-plugin-svelte-global-stores': {
			prefix: 'GQL_',
			generate: 'all'
		},
		'houdini-svelte': {
			static: true
		}
	},
	scalars: {
		DateTime: {
			type: 'Date'
		},
		UUID: {
			type: 'string'
		}
	}
};

export default config;
