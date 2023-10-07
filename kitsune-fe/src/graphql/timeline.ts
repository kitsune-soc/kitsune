import { useQuery } from '@urql/vue';

import { graphql } from './types';

function getHome() {
  const { data } = useQuery({
    query: graphql(`
      query getHomeTimeline {
        homeTimeline(before: "00000000-0000-0000-0000-000000000000")
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
  });

  return data;
}

export { getHome };
