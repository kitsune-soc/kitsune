import { HoudiniClient, getClientSession } from '$houdini';
import { houdiniPlugin as authPlugin } from '$lib/oauth/auth.svelte';

export default new HoudiniClient({
	url: `${import.meta.env.VITE_BACKEND_URL ?? ''}/graphql`,
	plugins: [authPlugin],
	fetchParams() {
		const session = getClientSession();
		return {
			headers: {
				...session.headers
			}
		};
	}
});
