import { type ClientPlugin } from '$houdini';

import { merge } from 'lodash';

import { loadOAuthToken } from './token.svelte';

const houdiniPlugin: ClientPlugin = () => {
	return {
		async beforeNetwork(ctx, { next }): Promise<void> {
			const headers: Record<string, string> = {};

			const token = await loadOAuthToken();
			if (token) {
				headers['Authorization'] = `Bearer ${token.accessToken}`;
			}

			next(merge({ fetchParams: { headers } }, ctx));
		}
	};
};

export { houdiniPlugin };
