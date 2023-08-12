import { BACKEND_PREFIX } from '../consts';
import { graphql } from '../graphql/types';
import { TokenData, useAuthStore } from '../store/auth';
import {
  OAuthApplication,
  useOAuthApplicationStore,
} from '../store/oauth_application';
import { urqlClient } from '../urql';

type OAuthResponse = {
  access_token: string;
  expires_in: number;
  refresh_token: string;
};

async function getApplicationCredentials(): Promise<OAuthApplication> {
  const oauthApplicationStore = useOAuthApplicationStore();
  if (oauthApplicationStore.application) {
    return oauthApplicationStore.application;
  }

  const redirectUri = `${window.location.origin}/oauth-callback`;
  const oauthApplicationMutation = urqlClient.mutation(
    graphql(`
      mutation registerOauthApplication(
        $name: String!
        $redirect_uri: String!
      ) {
        registerOauthApplication(name: $name, redirectUri: $redirect_uri) {
          id
          secret
          redirectUri
        }
      }
    `),
    {
      name: 'Kitsune FE',
      redirect_uri: redirectUri,
    },
  );

  const response = await oauthApplicationMutation.toPromise();
  if (!response.data) {
    throw new Error(
      'Empty response from server on application registration request',
    );
  }

  const { registerOauthApplication: applicationData } = response.data;
  oauthApplicationStore.application = applicationData;

  return oauthApplicationStore.application!;
}

function handleOAuthResponse(oauthResponse: OAuthResponse): TokenData {
  const expiresAt = new Date();
  expiresAt.setUTCSeconds(expiresAt.getUTCSeconds() + oauthResponse.expires_in);

  const tokenData: TokenData = {
    token: oauthResponse.access_token,
    refreshToken: oauthResponse.refresh_token,
    expiresAt: expiresAt.toString(),
  };

  const authStore = useAuthStore();
  authStore.data = tokenData;

  return authStore.data!;
}

export async function authorizationUrl(): Promise<string> {
  const applicationCredentials = await getApplicationCredentials();
  return `${BACKEND_PREFIX}/oauth/authorize?response_type=code&client_id=${applicationCredentials.id}&redirect_uri=${applicationCredentials.redirectUri}&scope=read+write`;
}

export async function obtainAccessToken(
  authorizationCode: string,
): Promise<TokenData> {
  const applicationCredentials = await getApplicationCredentials();
  const basicAuthCredentials = btoa(
    `${applicationCredentials.id}:${applicationCredentials.secret}`,
  );

  const response = await fetch(`${BACKEND_PREFIX}/oauth/token`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded',
      Authorization: `Basic ${basicAuthCredentials}`,
    },
    body: new URLSearchParams({
      grant_type: 'authorization_code',
      code: authorizationCode,
      redirect_uri: applicationCredentials.redirectUri,
    }).toString(),
  });

  if (response.status !== 200) {
    throw new Error('Authorization code flow unsuccessful', {
      cause: await response.text(),
    });
  }

  const oauthResponse: OAuthResponse = await response.json();
  return handleOAuthResponse(oauthResponse);
}

export async function refreshAccessToken(): Promise<TokenData> {
  const applicationCredentials = await getApplicationCredentials();
  const basicAuthCredentials = btoa(
    `${applicationCredentials.id}:${applicationCredentials.secret}`,
  );

  const authStore = useAuthStore();
  if (!authStore.isAuthenticated()) {
    throw new Error('Not authenticated');
  }

  const response = await fetch(`${BACKEND_PREFIX}/oauth/token`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded',
      Authorization: `Basic ${basicAuthCredentials}`,
    },
    body: new URLSearchParams({
      grant_type: 'refresh_token',
      refresh_token: authStore.data!.refreshToken,
      redirect_uri: applicationCredentials.redirectUri,
    }).toString(),
  });

  if (response.status !== 200) {
    throw new Error('Authorization code flow unsuccessful', {
      cause: await response.text(),
    });
  }

  const oauthResponse: OAuthResponse = await response.json();
  return handleOAuthResponse(oauthResponse);
}
