/* eslint-disable */
import type { TypedDocumentNode as DocumentNode } from '@graphql-typed-document-node/core';

export type Maybe<T> = T | null;
export type InputMaybe<T> = Maybe<T>;
export type Exact<T extends { [key: string]: unknown }> = {
  [K in keyof T]: T[K];
};
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & {
  [SubKey in K]?: Maybe<T[SubKey]>;
};
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & {
  [SubKey in K]: Maybe<T[SubKey]>;
};
export type MakeEmpty<
  T extends { [key: string]: unknown },
  K extends keyof T,
> = { [_ in K]?: never };
export type Incremental<T> =
  | T
  | {
      [P in keyof T]?: P extends ' $fragmentName' | '__typename' ? T[P] : never;
    };
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: { input: string; output: string };
  String: { input: string; output: string };
  Boolean: { input: boolean; output: boolean };
  Int: { input: number; output: number };
  Float: { input: number; output: number };
  /**
   * A datetime with timezone offset.
   *
   * The input is a string in RFC3339 format, e.g. "2022-01-12T04:00:19.12345Z"
   * or "2022-01-12T04:00:19+03:00". The output is also a string in RFC3339
   * format, but it is always normalized to the UTC (Z) offset, e.g.
   * "2022-01-12T04:00:19.12345Z".
   */
  DateTime: { input: any; output: any };
  /**
   * A UUID is a unique 128-bit number, stored as 16 octets. UUIDs are parsed as
   * Strings within GraphQL. UUIDs are used to assign unique identifiers to
   * entities without requiring a central allocating authority.
   *
   * # References
   *
   * * [Wikipedia: Universally Unique Identifier](http://en.wikipedia.org/wiki/Universally_unique_identifier)
   * * [RFC4122: A Universally Unique IDentifier (UUID) URN Namespace](http://tools.ietf.org/html/rfc4122)
   */
  UUID: { input: any; output: any };
  Upload: { input: any; output: any };
};

export type Account = {
  __typename?: 'Account';
  avatar?: Maybe<MediaAttachment>;
  createdAt: Scalars['DateTime']['output'];
  displayName?: Maybe<Scalars['String']['output']>;
  header?: Maybe<MediaAttachment>;
  id: Scalars['UUID']['output'];
  locked: Scalars['Boolean']['output'];
  note?: Maybe<Scalars['String']['output']>;
  posts: PostConnection;
  updatedAt: Scalars['DateTime']['output'];
  url: Scalars['String']['output'];
  username: Scalars['String']['output'];
};

export type AccountPostsArgs = {
  after?: InputMaybe<Scalars['String']['input']>;
  before?: InputMaybe<Scalars['String']['input']>;
  first?: InputMaybe<Scalars['Int']['input']>;
  last?: InputMaybe<Scalars['Int']['input']>;
};

export enum CaptchaBackend {
  HCaptcha = 'H_CAPTCHA',
  MCaptcha = 'M_CAPTCHA',
}

export type CaptchaInfo = {
  __typename?: 'CaptchaInfo';
  backend: CaptchaBackend;
  key: Scalars['String']['output'];
};

export type Instance = {
  __typename?: 'Instance';
  captcha?: Maybe<CaptchaInfo>;
  description: Scalars['String']['output'];
  domain: Scalars['String']['output'];
  localPostCount: Scalars['Int']['output'];
  name: Scalars['String']['output'];
  registrationsOpen: Scalars['Boolean']['output'];
  userCount: Scalars['Int']['output'];
  version: Scalars['String']['output'];
};

export type MediaAttachment = {
  __typename?: 'MediaAttachment';
  blurhash?: Maybe<Scalars['String']['output']>;
  contentType: Scalars['String']['output'];
  createdAt: Scalars['DateTime']['output'];
  description?: Maybe<Scalars['String']['output']>;
  id: Scalars['UUID']['output'];
  uploader: Account;
  url: Scalars['String']['output'];
};

export type OAuth2Application = {
  __typename?: 'OAuth2Application';
  createdAt: Scalars['DateTime']['output'];
  id: Scalars['UUID']['output'];
  name: Scalars['String']['output'];
  redirectUri: Scalars['String']['output'];
  secret: Scalars['String']['output'];
  updatedAt: Scalars['DateTime']['output'];
};

/** Information about pagination in a connection */
export type PageInfo = {
  __typename?: 'PageInfo';
  /** When paginating forwards, the cursor to continue. */
  endCursor?: Maybe<Scalars['String']['output']>;
  /** When paginating forwards, are there more items? */
  hasNextPage: Scalars['Boolean']['output'];
  /** When paginating backwards, are there more items? */
  hasPreviousPage: Scalars['Boolean']['output'];
  /** When paginating backwards, the cursor to continue. */
  startCursor?: Maybe<Scalars['String']['output']>;
};

export type Post = {
  __typename?: 'Post';
  account: Account;
  attachments: Array<MediaAttachment>;
  content: Scalars['String']['output'];
  createdAt: Scalars['DateTime']['output'];
  id: Scalars['UUID']['output'];
  isSensitive: Scalars['Boolean']['output'];
  subject?: Maybe<Scalars['String']['output']>;
  updatedAt: Scalars['DateTime']['output'];
  url: Scalars['String']['output'];
  visibility: Visibility;
};

export type PostConnection = {
  __typename?: 'PostConnection';
  /** A list of edges. */
  edges: Array<PostEdge>;
  /** A list of nodes. */
  nodes: Array<Post>;
  /** Information to aid in pagination. */
  pageInfo: PageInfo;
};

/** An edge in a connection. */
export type PostEdge = {
  __typename?: 'PostEdge';
  /** A cursor for use in pagination */
  cursor: Scalars['String']['output'];
  /** The item at the end of the edge */
  node: Post;
};

export type RootMutation = {
  __typename?: 'RootMutation';
  createPost: Post;
  deletePost: Scalars['UUID']['output'];
  registerOauthApplication: OAuth2Application;
  registerUser: User;
  updateUser: Account;
};

export type RootMutationCreatePostArgs = {
  content: Scalars['String']['input'];
  isSensitive: Scalars['Boolean']['input'];
  visibility: Visibility;
};

export type RootMutationDeletePostArgs = {
  id: Scalars['UUID']['input'];
};

export type RootMutationRegisterOauthApplicationArgs = {
  name: Scalars['String']['input'];
  redirectUri: Scalars['String']['input'];
};

export type RootMutationRegisterUserArgs = {
  captchaToken?: InputMaybe<Scalars['String']['input']>;
  email: Scalars['String']['input'];
  password: Scalars['String']['input'];
  username: Scalars['String']['input'];
};

export type RootMutationUpdateUserArgs = {
  avatar?: InputMaybe<Scalars['Upload']['input']>;
  displayName?: InputMaybe<Scalars['String']['input']>;
  header?: InputMaybe<Scalars['Upload']['input']>;
  locked?: InputMaybe<Scalars['Boolean']['input']>;
  note?: InputMaybe<Scalars['String']['input']>;
};

export type RootQuery = {
  __typename?: 'RootQuery';
  getAccountById?: Maybe<Account>;
  getPostById: Post;
  homeTimeline: PostConnection;
  instance: Instance;
  myAccount: Account;
  publicTimeline: PostConnection;
};

export type RootQueryGetAccountByIdArgs = {
  id: Scalars['UUID']['input'];
};

export type RootQueryGetPostByIdArgs = {
  id: Scalars['UUID']['input'];
};

export type RootQueryHomeTimelineArgs = {
  after?: InputMaybe<Scalars['String']['input']>;
  before?: InputMaybe<Scalars['String']['input']>;
  first?: InputMaybe<Scalars['Int']['input']>;
  last?: InputMaybe<Scalars['Int']['input']>;
};

export type RootQueryPublicTimelineArgs = {
  after?: InputMaybe<Scalars['String']['input']>;
  before?: InputMaybe<Scalars['String']['input']>;
  first?: InputMaybe<Scalars['Int']['input']>;
  last?: InputMaybe<Scalars['Int']['input']>;
  onlyLocal?: Scalars['Boolean']['input'];
};

export type User = {
  __typename?: 'User';
  account: Account;
  createdAt: Scalars['DateTime']['output'];
  email: Scalars['String']['output'];
  id: Scalars['UUID']['output'];
  updatedAt: Scalars['DateTime']['output'];
  username: Scalars['String']['output'];
};

export enum Visibility {
  FollowerOnly = 'FOLLOWER_ONLY',
  MentionOnly = 'MENTION_ONLY',
  Public = 'PUBLIC',
  Unlisted = 'UNLISTED',
}

export type RegisterUserMutationVariables = Exact<{
  username: Scalars['String']['input'];
  email: Scalars['String']['input'];
  password: Scalars['String']['input'];
  captchaToken?: InputMaybe<Scalars['String']['input']>;
}>;

export type RegisterUserMutation = {
  __typename?: 'RootMutation';
  registerUser: { __typename?: 'User'; id: any };
};

export type GetInstanceInfoQueryVariables = Exact<{ [key: string]: never }>;

export type GetInstanceInfoQuery = {
  __typename?: 'RootQuery';
  instance: {
    __typename?: 'Instance';
    description: string;
    domain: string;
    localPostCount: number;
    registrationsOpen: boolean;
    name: string;
    userCount: number;
    version: string;
    captcha?: {
      __typename?: 'CaptchaInfo';
      backend: CaptchaBackend;
      key: string;
    } | null;
  };
};

export type GetHomeTimelineQueryVariables = Exact<{ [key: string]: never }>;

export type GetHomeTimelineQuery = {
  __typename?: 'RootQuery';
  homeTimeline: {
    __typename?: 'PostConnection';
    nodes: Array<{
      __typename?: 'Post';
      id: any;
      subject?: string | null;
      content: string;
      url: string;
      account: {
        __typename?: 'Account';
        id: any;
        displayName?: string | null;
        username: string;
        url: string;
      };
    }>;
    pageInfo: {
      __typename?: 'PageInfo';
      startCursor?: string | null;
      endCursor?: string | null;
    };
  };
};

export type RegisterOauthApplicationMutationVariables = Exact<{
  name: Scalars['String']['input'];
  redirect_uri: Scalars['String']['input'];
}>;

export type RegisterOauthApplicationMutation = {
  __typename?: 'RootMutation';
  registerOauthApplication: {
    __typename?: 'OAuth2Application';
    id: any;
    secret: string;
    redirectUri: string;
  };
};

export const RegisterUserDocument = {
  kind: 'Document',
  definitions: [
    {
      kind: 'OperationDefinition',
      operation: 'mutation',
      name: { kind: 'Name', value: 'registerUser' },
      variableDefinitions: [
        {
          kind: 'VariableDefinition',
          variable: {
            kind: 'Variable',
            name: { kind: 'Name', value: 'username' },
          },
          type: {
            kind: 'NonNullType',
            type: {
              kind: 'NamedType',
              name: { kind: 'Name', value: 'String' },
            },
          },
        },
        {
          kind: 'VariableDefinition',
          variable: {
            kind: 'Variable',
            name: { kind: 'Name', value: 'email' },
          },
          type: {
            kind: 'NonNullType',
            type: {
              kind: 'NamedType',
              name: { kind: 'Name', value: 'String' },
            },
          },
        },
        {
          kind: 'VariableDefinition',
          variable: {
            kind: 'Variable',
            name: { kind: 'Name', value: 'password' },
          },
          type: {
            kind: 'NonNullType',
            type: {
              kind: 'NamedType',
              name: { kind: 'Name', value: 'String' },
            },
          },
        },
        {
          kind: 'VariableDefinition',
          variable: {
            kind: 'Variable',
            name: { kind: 'Name', value: 'captchaToken' },
          },
          type: { kind: 'NamedType', name: { kind: 'Name', value: 'String' } },
        },
      ],
      selectionSet: {
        kind: 'SelectionSet',
        selections: [
          {
            kind: 'Field',
            name: { kind: 'Name', value: 'registerUser' },
            arguments: [
              {
                kind: 'Argument',
                name: { kind: 'Name', value: 'username' },
                value: {
                  kind: 'Variable',
                  name: { kind: 'Name', value: 'username' },
                },
              },
              {
                kind: 'Argument',
                name: { kind: 'Name', value: 'email' },
                value: {
                  kind: 'Variable',
                  name: { kind: 'Name', value: 'email' },
                },
              },
              {
                kind: 'Argument',
                name: { kind: 'Name', value: 'password' },
                value: {
                  kind: 'Variable',
                  name: { kind: 'Name', value: 'password' },
                },
              },
              {
                kind: 'Argument',
                name: { kind: 'Name', value: 'captchaToken' },
                value: {
                  kind: 'Variable',
                  name: { kind: 'Name', value: 'captchaToken' },
                },
              },
            ],
            selectionSet: {
              kind: 'SelectionSet',
              selections: [
                { kind: 'Field', name: { kind: 'Name', value: 'id' } },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  RegisterUserMutation,
  RegisterUserMutationVariables
>;
export const GetInstanceInfoDocument = {
  kind: 'Document',
  definitions: [
    {
      kind: 'OperationDefinition',
      operation: 'query',
      name: { kind: 'Name', value: 'getInstanceInfo' },
      selectionSet: {
        kind: 'SelectionSet',
        selections: [
          {
            kind: 'Field',
            name: { kind: 'Name', value: 'instance' },
            selectionSet: {
              kind: 'SelectionSet',
              selections: [
                { kind: 'Field', name: { kind: 'Name', value: 'description' } },
                { kind: 'Field', name: { kind: 'Name', value: 'domain' } },
                {
                  kind: 'Field',
                  name: { kind: 'Name', value: 'localPostCount' },
                },
                {
                  kind: 'Field',
                  name: { kind: 'Name', value: 'registrationsOpen' },
                },
                { kind: 'Field', name: { kind: 'Name', value: 'name' } },
                { kind: 'Field', name: { kind: 'Name', value: 'userCount' } },
                { kind: 'Field', name: { kind: 'Name', value: 'version' } },
                {
                  kind: 'Field',
                  name: { kind: 'Name', value: 'captcha' },
                  selectionSet: {
                    kind: 'SelectionSet',
                    selections: [
                      {
                        kind: 'Field',
                        name: { kind: 'Name', value: 'backend' },
                      },
                      { kind: 'Field', name: { kind: 'Name', value: 'key' } },
                    ],
                  },
                },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  GetInstanceInfoQuery,
  GetInstanceInfoQueryVariables
>;
export const GetHomeTimelineDocument = {
  kind: 'Document',
  definitions: [
    {
      kind: 'OperationDefinition',
      operation: 'query',
      name: { kind: 'Name', value: 'getHomeTimeline' },
      selectionSet: {
        kind: 'SelectionSet',
        selections: [
          {
            kind: 'Field',
            name: { kind: 'Name', value: 'homeTimeline' },
            arguments: [
              {
                kind: 'Argument',
                name: { kind: 'Name', value: 'before' },
                value: {
                  kind: 'StringValue',
                  value: '00000000-0000-0000-0000-000000000000',
                  block: false,
                },
              },
            ],
            directives: [
              {
                kind: 'Directive',
                name: { kind: 'Name', value: '_relayPagination' },
                arguments: [
                  {
                    kind: 'Argument',
                    name: { kind: 'Name', value: 'mergeMode' },
                    value: {
                      kind: 'StringValue',
                      value: 'after',
                      block: false,
                    },
                  },
                ],
              },
            ],
            selectionSet: {
              kind: 'SelectionSet',
              selections: [
                {
                  kind: 'Field',
                  name: { kind: 'Name', value: 'nodes' },
                  selectionSet: {
                    kind: 'SelectionSet',
                    selections: [
                      { kind: 'Field', name: { kind: 'Name', value: 'id' } },
                      {
                        kind: 'Field',
                        name: { kind: 'Name', value: 'subject' },
                      },
                      {
                        kind: 'Field',
                        name: { kind: 'Name', value: 'content' },
                      },
                      { kind: 'Field', name: { kind: 'Name', value: 'url' } },
                      {
                        kind: 'Field',
                        name: { kind: 'Name', value: 'account' },
                        selectionSet: {
                          kind: 'SelectionSet',
                          selections: [
                            {
                              kind: 'Field',
                              name: { kind: 'Name', value: 'id' },
                            },
                            {
                              kind: 'Field',
                              name: { kind: 'Name', value: 'displayName' },
                            },
                            {
                              kind: 'Field',
                              name: { kind: 'Name', value: 'username' },
                            },
                            {
                              kind: 'Field',
                              name: { kind: 'Name', value: 'url' },
                            },
                          ],
                        },
                      },
                    ],
                  },
                },
                {
                  kind: 'Field',
                  name: { kind: 'Name', value: 'pageInfo' },
                  selectionSet: {
                    kind: 'SelectionSet',
                    selections: [
                      {
                        kind: 'Field',
                        name: { kind: 'Name', value: 'startCursor' },
                      },
                      {
                        kind: 'Field',
                        name: { kind: 'Name', value: 'endCursor' },
                      },
                    ],
                  },
                },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  GetHomeTimelineQuery,
  GetHomeTimelineQueryVariables
>;
export const RegisterOauthApplicationDocument = {
  kind: 'Document',
  definitions: [
    {
      kind: 'OperationDefinition',
      operation: 'mutation',
      name: { kind: 'Name', value: 'registerOauthApplication' },
      variableDefinitions: [
        {
          kind: 'VariableDefinition',
          variable: { kind: 'Variable', name: { kind: 'Name', value: 'name' } },
          type: {
            kind: 'NonNullType',
            type: {
              kind: 'NamedType',
              name: { kind: 'Name', value: 'String' },
            },
          },
        },
        {
          kind: 'VariableDefinition',
          variable: {
            kind: 'Variable',
            name: { kind: 'Name', value: 'redirect_uri' },
          },
          type: {
            kind: 'NonNullType',
            type: {
              kind: 'NamedType',
              name: { kind: 'Name', value: 'String' },
            },
          },
        },
      ],
      selectionSet: {
        kind: 'SelectionSet',
        selections: [
          {
            kind: 'Field',
            name: { kind: 'Name', value: 'registerOauthApplication' },
            arguments: [
              {
                kind: 'Argument',
                name: { kind: 'Name', value: 'name' },
                value: {
                  kind: 'Variable',
                  name: { kind: 'Name', value: 'name' },
                },
              },
              {
                kind: 'Argument',
                name: { kind: 'Name', value: 'redirectUri' },
                value: {
                  kind: 'Variable',
                  name: { kind: 'Name', value: 'redirect_uri' },
                },
              },
            ],
            selectionSet: {
              kind: 'SelectionSet',
              selections: [
                { kind: 'Field', name: { kind: 'Name', value: 'id' } },
                { kind: 'Field', name: { kind: 'Name', value: 'secret' } },
                { kind: 'Field', name: { kind: 'Name', value: 'redirectUri' } },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  RegisterOauthApplicationMutation,
  RegisterOauthApplicationMutationVariables
>;
