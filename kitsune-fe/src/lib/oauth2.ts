import { useMutation } from '@vue/apollo-composable';
import gql from 'graphql-tag';
import {
  OAuthApplication,
  useOAuthApplicationStore,
} from '../store/oauth_application';

export async function getApplicationCredentials(): Promise<OAuthApplication> {
  const oauthApplicationStore = useOAuthApplicationStore();
  if (oauthApplicationStore.application) {
    return oauthApplicationStore.application;
  }

  const { mutate: registerOAuthApplication } = useMutation(gql`
    mutation registerOauthApplication($name: String!, $redirect_uri: String!) {
      registerOauthApplication(name: $name, redirectUri: $redirect_uri) {
        id
        secret
        redirectUri
      }
    }
  `);

  const response = await registerOAuthApplication({
    name: 'Kitsune FE',
    redirect_uri: 'http://example.com',
  });

  const { registerOauthApplication: applicationData } = response?.data;
  oauthApplicationStore.application = applicationData;

  return oauthApplicationStore.application!;
}
