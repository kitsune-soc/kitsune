import { HoudiniClient, subscription } from '$houdini';
import { houdiniPlugin as authPlugin } from '$lib/oauth/auth.svelte';

import { createClient } from 'graphql-ws';

export default new HoudiniClient({
	url: `/graphql`,
	plugins: [
		authPlugin,
		subscription(() =>
			createClient({
				url: `/graphql/ws`
			})
		)
	]
});
