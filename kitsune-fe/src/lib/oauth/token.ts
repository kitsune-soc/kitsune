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

/**
 * Attempt to load the OAuth token from local storage
 *
 * @returns OAuth token structure if the data was present and well formed, returns `undefined` otherwise
 */
function loadFromStorage(): OAuthTokenStorageTy | undefined {
	const data = localStorage.getItem(OAUTH_TOKEN_STORAGE_KEY);
	if (!data) {
		return undefined;
	}

	const parseResult = OAUTH_TOKEN_STORAGE_SCHEMA.safeParse(JSON.parse(data));
	if (parseResult.success) {
		return parseResult.data;
	} else {
		console.error(`Failed to parse OAuth token from local storage`);
		console.error(parseResult.error);
		return undefined;
	}
}

// initially attempt to load the token from local storage. if that doesn't work, just leave it undefined.
const tokenStore = writable<OAuthTokenStorageTy | undefined>(undefined, (set) => {
	const maybeToken = loadFromStorage();

	if (maybeToken) {
		set(maybeToken);
	} else {
		clearTokenStorage();
	}
});

// register a refresh callback when the token expires
tokenStore.subscribe((newToken) => {
	if (!newToken) return;

	const difference = DateTime.fromJSDate(newToken.expiresAt).diffNow();

	setTimeout(async () => {
		console.log('token expired. refreshing..');
		await refreshOAuthToken(newToken);
	}, difference.toMillis());
});

// persist store changes to local storage
tokenStore.subscribe((newToken) => {
	if (newToken) {
		localStorage.setItem(OAUTH_TOKEN_STORAGE_KEY, JSON.stringify(newToken));
	} else {
		localStorage.removeItem(OAUTH_TOKEN_STORAGE_KEY);
	}
});

/**
 * Helper function for parsing an opaque object into a structure appropriate to be stored internally as an OAuth token
 *
 * @param body Opaque object read from the body of a completed OAuth flow
 * @returns Parsed and constructed OAuth token structure
 */
async function parseResponseBody(body: unknown): Promise<OAuthTokenStorageTy> {
	const responseBody = await OAUTH_TOKEN_SCHEMA.parseAsync(body);
	const expiresAt = DateTime.now().plus({ seconds: responseBody.expires_in }).toJSDate();

	return {
		accessToken: responseBody.access_token,
		refreshToken: responseBody.refresh_token,
		expiresAt
	};
}

/**
 * Fetch an OAuth token from your access code and set the token store to the result.
 * This function has no return value since we modify the store.
 *
 * @param oauthCode Code sent to the token endpoint as part of the authorization code challenge
 */
async function fetchOAuthToken(oauthCode: string): Promise<void> {
	const oauthApp = await loadOAuthApp();

	const body = new URLSearchParams({
		grant_type: 'authorization_code',
		client_id: oauthApp.id,
		client_secret: oauthApp.secret,
		redirect_uri: oauthApp.redirectUri,
		code: oauthCode
	});

	const response = await fetch(`/oauth/token`, {
		method: 'POST',
		body: body.toString()
	});

	tokenStore.set(await parseResponseBody(await response.json()));
}

/**
 * Run the OAuth refresh flow and set the token store to the result
 *
 * @param token Token as stored in the token storage
 */
async function refreshOAuthToken(token: OAuthTokenStorageTy): Promise<void> {
	const oauthApp = await loadOAuthApp();

	const body = new URLSearchParams({
		grant_type: 'refresh_token',
		client_id: oauthApp.id,
		client_secret: oauthApp.secret,
		redirect_uri: oauthApp.redirectUri,
		refresh_token: token.refreshToken
	});

	const response = await fetch(`/oauth/token`, {
		method: 'POST',
		body: body.toString()
	});

	tokenStore.set(await parseResponseBody(await response.json()));
}

/**
 * Remove the token from memory and persistent storage
 */
function clearTokenStorage() {
	tokenStore.set(undefined);
}

export { clearTokenStorage, fetchOAuthToken, tokenStore };
