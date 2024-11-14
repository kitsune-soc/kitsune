import { HoudiniClient } from '$houdini';
import { houdiniPlugin as authPlugin } from '$lib/oauth/auth.svelte';

export default new HoudiniClient({
	url: `/graphql`,
	plugins: [authPlugin],
	fetchParams() {
		//const session = getClientSession();
		return {
			headers: {
				//...session.headers
			}
		};
	}
});
