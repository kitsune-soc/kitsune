import { DateTime } from 'luxon';
import { writable } from 'svelte/store';
import { z } from 'zod';

import { loadOAuthApp } from './client';

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

const tokenStore = writable<OAuthTokenStorageTy | undefined>(undefined, (set) => {
	const data = localStorage.getItem(OAUTH_TOKEN_STORAGE_KEY);
	if (!data) {
		return;
	}

	const parseResult = OAUTH_TOKEN_STORAGE_SCHEMA.safeParse(JSON.parse(data));
	if (parseResult.success) {
		set(parseResult.data);
	} else {
		console.error(`Failed to parse OAuth token from local storage`);
		console.error(parseResult.error);
		clearTokenStorage();
	}
});

// register a refresh callback when the token expires
tokenStore.subscribe((newToken) => {
	if (!newToken) return;

	const difference = DateTime.fromJSDate(newToken.expiresAt).diffNow();

	setTimeout(async () => {
		console.log('teto teto beam');

		await refreshOAuthToken(newToken);
	}, difference.toMillis());
});

// store new tokens in local storage
tokenStore.subscribe((newToken) => {
	if (newToken) {
		localStorage.setItem(OAUTH_TOKEN_STORAGE_KEY, JSON.stringify(newToken));
	} else {
		localStorage.removeItem(OAUTH_TOKEN_STORAGE_KEY);
	}
});

async function parseResponseBody(body: unknown): Promise<OAuthTokenStorageTy> {
	const responseBody = await OAUTH_TOKEN_SCHEMA.parseAsync(body);
	const expiresAt = DateTime.now().plus({ seconds: responseBody.expires_in }).toJSDate();

	return {
		accessToken: responseBody.access_token,
		refreshToken: responseBody.refresh_token,
		expiresAt
	};
}

async function fetchOAuthToken(oauthCode: string): Promise<void> {
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

	tokenStore.set(await parseResponseBody(await response.json()));
}

async function refreshOAuthToken(token: OAuthTokenStorageTy): Promise<void> {
	const oauthApp = await loadOAuthApp();

	const body = new URLSearchParams({
		grant_type: 'refresh_token',
		client_id: oauthApp.id,
		client_secret: oauthApp.secret,
		redirect_uri: oauthApp.redirectUri,
		refresh_token: token.refreshToken
	});

	const response = await fetch(`${window.location.origin}/oauth/token`, {
		method: 'POST',
		body: body.toString()
	});

	tokenStore.set(await parseResponseBody(await response.json()));
}

function clearTokenStorage() {
	tokenStore.set(undefined);
}

export { clearTokenStorage, fetchOAuthToken, tokenStore };
