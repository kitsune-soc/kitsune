import { useMutation } from '@vue/apollo-composable';
import gql from 'graphql-tag';
import {
  OAuthApplication,
  useOAuthApplicationStore,
} from '../store/oauth_application';
import { provideApolloClient } from '../apollo';
import { TokenData, useAuthStore } from '../store/auth';

export async function getApplicationCredentials(): Promise<OAuthApplication> {
  const oauthApplicationStore = useOAuthApplicationStore();
  if (oauthApplicationStore.application) {
    return oauthApplicationStore.application;
  }

  const { mutate: registerOAuthApplication } = provideApolloClient(() => {
    return useMutation(gql`
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
    `);
  });

  const redirectUri = `${window.location.origin}/oauth-callback`;
  const response = await registerOAuthApplication({
    name: 'Kitsune FE',
    redirect_uri: redirectUri,
  });

  const { registerOauthApplication: applicationData } = response?.data;
  oauthApplicationStore.application = applicationData;

  return oauthApplicationStore.application!;
}

type OAuthResponse = {
  access_token: string;
  expires_in: number;
  refresh_token: string;
};

export async function obtainAccessToken(
  authorizationCode: string,
): Promise<TokenData> {
  const applicationCredentials = await getApplicationCredentials();
  const basicAuthCredentials = btoa(
    `${applicationCredentials.id}:${applicationCredentials.secret}`,
  );

  const response = await fetch('/oauth/token', {
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
  const expiresAt = new Date();
  expiresAt.setSeconds(expiresAt.getSeconds() + oauthResponse.expires_in);

  const tokenData: TokenData = {
    token: oauthResponse.access_token,
    refreshToken: oauthResponse.refresh_token,
    expiresAt,
  };

  const authStore = useAuthStore();
  authStore.data = tokenData;

  return authStore.data!;
}
