import { RegisterOAuthAppStore } from '$houdini';

import { z } from 'zod';

const OAUTH_APP_STORAGE_KEY = 'oauth_app';
const OAUTH_APP_SCHEMA = z.object({
	id: z.string().nonempty(),
	secret: z.string().nonempty(),
	redirectUri: z.string().url().nonempty()
});

const REGISTER_APP = new RegisterOAuthAppStore();

type OAuthApplicationTy = z.infer<typeof OAUTH_APP_SCHEMA>;

async function registerOAuthApp(): Promise<OAuthApplicationTy> {
	const redirectUri = `${window.location.origin}/oauth-callback`;

	try {
		const response = await REGISTER_APP.mutate({
			redirectUri
		});

		if (response.errors) {
			throw new Error(response.errors.map((error) => error.message).join('\n'));
		}

		// If we don't have any errors, we can assume the data is well formed.
		// And if the data isn't well formed, then we're fucked anyways.

		return response.data!.registerOauthApplication;
	} catch (error) {
		console.error(`Failed to register OAuth app: ${error}`);
		throw error;
	}
}

async function registerAndStore(): Promise<void> {
	const oauthApp = await registerOAuthApp();
	localStorage.setItem(OAUTH_APP_STORAGE_KEY, JSON.stringify(oauthApp));
}

async function loadOAuthApp(): Promise<OAuthApplicationTy> {
	const rawApp = localStorage.getItem(OAUTH_APP_STORAGE_KEY);

	if (rawApp) {
		try {
			return OAUTH_APP_SCHEMA.parseAsync(JSON.parse(rawApp));
		} catch (error) {
			console.error(`Failed to load OAuth app. Error: ${error}`);
			console.error(`Registering new..`);

			await registerAndStore();
			return await loadOAuthApp();
		}
	} else {
		await registerAndStore();
		return await loadOAuthApp();
	}
}

export { loadOAuthApp };
