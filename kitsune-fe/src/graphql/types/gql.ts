/* eslint-disable */
import type { TypedDocumentNode as DocumentNode } from '@graphql-typed-document-node/core';

import * as types from './graphql';

/**
 * Map of all GraphQL operations in the project.
 *
 * This map has several performance disadvantages:
 * 1. It is not tree-shakeable, so it will include all operations in the project.
 * 2. It is not minifiable, so the string of a GraphQL query will be multiple times inside the bundle.
 * 3. It does not support dead code elimination, so it will add unused operations.
 *
 * Therefore it is highly recommended to use the babel or swc plugin for production.
 */
const documents = {
  '\n      mutation registerUser(\n        $username: String!\n        $email: String!\n        $password: String!\n        $captchaToken: String\n      ) {\n        registerUser(\n          username: $username\n          email: $email\n          password: $password\n          captchaToken: $captchaToken\n        ) {\n          id\n        }\n      }\n    ':
    types.RegisterUserDocument,
  '\n      query getInstanceInfo {\n        instance {\n          description\n          domain\n          localPostCount\n          registrationsOpen\n          name\n          userCount\n          version\n          captcha {\n            backend\n            key\n          }\n        }\n      }\n    ':
    types.GetInstanceInfoDocument,
  '\n      mutation registerOauthApplication(\n        $name: String!\n        $redirect_uri: String!\n      ) {\n        registerOauthApplication(name: $name, redirectUri: $redirect_uri) {\n          id\n          secret\n          redirectUri\n        }\n      }\n    ':
    types.RegisterOauthApplicationDocument,
};

/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 *
 *
 * @example
 * ```ts
 * const query = graphql(`query GetUser($id: ID!) { user(id: $id) { name } }`);
 * ```
 *
 * The query argument is unknown!
 * Please regenerate the types.
 */
export function graphql(source: string): unknown;

/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(
  source: '\n      mutation registerUser(\n        $username: String!\n        $email: String!\n        $password: String!\n        $captchaToken: String\n      ) {\n        registerUser(\n          username: $username\n          email: $email\n          password: $password\n          captchaToken: $captchaToken\n        ) {\n          id\n        }\n      }\n    ',
): (typeof documents)['\n      mutation registerUser(\n        $username: String!\n        $email: String!\n        $password: String!\n        $captchaToken: String\n      ) {\n        registerUser(\n          username: $username\n          email: $email\n          password: $password\n          captchaToken: $captchaToken\n        ) {\n          id\n        }\n      }\n    '];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(
  source: '\n      query getInstanceInfo {\n        instance {\n          description\n          domain\n          localPostCount\n          registrationsOpen\n          name\n          userCount\n          version\n          captcha {\n            backend\n            key\n          }\n        }\n      }\n    ',
): (typeof documents)['\n      query getInstanceInfo {\n        instance {\n          description\n          domain\n          localPostCount\n          registrationsOpen\n          name\n          userCount\n          version\n          captcha {\n            backend\n            key\n          }\n        }\n      }\n    '];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(
  source: '\n      mutation registerOauthApplication(\n        $name: String!\n        $redirect_uri: String!\n      ) {\n        registerOauthApplication(name: $name, redirectUri: $redirect_uri) {\n          id\n          secret\n          redirectUri\n        }\n      }\n    ',
): (typeof documents)['\n      mutation registerOauthApplication(\n        $name: String!\n        $redirect_uri: String!\n      ) {\n        registerOauthApplication(name: $name, redirectUri: $redirect_uri) {\n          id\n          secret\n          redirectUri\n        }\n      }\n    '];

export function graphql(source: string) {
  return (documents as any)[source] ?? {};
}

export type DocumentType<TDocumentNode extends DocumentNode<any, any>> =
  TDocumentNode extends DocumentNode<infer TType, any> ? TType : never;
