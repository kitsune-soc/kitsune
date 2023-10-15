import { useQuery } from '@urql/vue';

import { Ref } from 'vue';

import { graphql } from './types';

function getHome(after: Ref<string>) {
  const { data } = useQuery({
    query: graphql(`
      query getHomeTimeline($after: String!) {
        homeTimeline(after: $after) @_relayPagination(mergeMode: "after") {
          nodes {
            id
            subject
            content
            url
            account {
              id
              avatar {
                url
              }
              displayName
              username
              url
            }
            attachments {
              contentType
              description
              url
            }
          }
          pageInfo {
            startCursor
            endCursor
          }
        }
      }
    `),
    variables: {
      after: after as unknown as string, // Weird cast to allow reactivity
    },
  });

  return data;
}

function getPublic(after: Ref<string>, onlyLocal: boolean) {
  const { data } = useQuery({
    query: graphql(`
      query getPublicTimeline($after: String!, $onlyLocal: Boolean!) {
        publicTimeline(after: $after, onlyLocal: $onlyLocal)
          @_relayPagination(mergeMode: "after") {
          nodes {
            id
            subject
            content
            url
            account {
              id
              avatar {
                url
              }
              displayName
              username
              url
            }
            attachments {
              contentType
              description
              url
            }
          }
          pageInfo {
            startCursor
            endCursor
          }
        }
      }
    `),
    variables: {
      after: after as unknown as string,
      onlyLocal,
    },
  });

  return data;
}

export { getHome, getPublic };
