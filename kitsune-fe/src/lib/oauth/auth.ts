import { type ClientPlugin } from '$houdini';

import { merge } from 'lodash';
import { get } from 'svelte/store';

import { tokenStore } from './token';

const houdiniPlugin: ClientPlugin = () => {
	return {
		async beforeNetwork(ctx, { next }): Promise<void> {
			const headers: Record<string, string> = {};

			const token = get(tokenStore);
			if (token) {
				headers['Authorization'] = `Bearer ${token.accessToken}`;
			}

			next(merge({ fetchParams: { headers } }, ctx));
		}
	};
};

export { houdiniPlugin };
