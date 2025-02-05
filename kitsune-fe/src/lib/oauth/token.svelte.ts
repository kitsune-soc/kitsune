import { DateTime } from 'luxon';
import { z } from 'zod';

import { loadOAuthApp } from './client.svelte';

const OAUTH_TOKEN_STORAGE_KEY = 'oauth_token';
const OAUTH_TOKEN_STORAGE_SCHEMA = z.object({
	accessToken: z.string().nonempty(),
	refreshToken: z.string().nonempty(),
	expiresAt: z.coerce.date()
});

const OAUTH_TOKEN_SCHEMA = z.object({
	access_token: z.string().nonempty(),
	token_type: z.enum(['Bearer']),
	refresh_token: z.string().nonempty(),
	expires_in: z.number().positive()
});

type OAuthTokenStorageTy = z.infer<typeof OAUTH_TOKEN_STORAGE_SCHEMA>;

async function fetchOAuthToken(oauthCode: string): Promise<OAuthTokenStorageTy> {
	const oauthApp = await loadOAuthApp();

	const body = new URLSearchParams({
		grant_type: 'authorization_code',
		client_id: oauthApp.id,
		client_secret: oauthApp.secret,
		redirect_uri: oauthApp.redirectUri,
		code: oauthCode
	});

	const response = await fetch(`${window.location.origin}/oauth/token`, {
		method: 'POST',
		body: body.toString()
	});

	const responseBody = await OAUTH_TOKEN_SCHEMA.parseAsync(await response.json());
	const expiresAt = DateTime.now().plus({ seconds: responseBody.expires_in }).toJSDate();

	const stored: OAuthTokenStorageTy = {
		accessToken: responseBody.access_token,
		refreshToken: responseBody.refresh_token,
		expiresAt
	};

	localStorage.setItem(OAUTH_TOKEN_STORAGE_KEY, JSON.stringify(stored));

	return await loadOAuthToken();
}

async function loadOAuthToken(): Promise<OAuthTokenStorageTy> {
	const loaded = localStorage.getItem(OAUTH_TOKEN_STORAGE_KEY);
	return await OAUTH_TOKEN_STORAGE_SCHEMA.parseAsync(loaded);
}

export { fetchOAuthToken, loadOAuthToken };
